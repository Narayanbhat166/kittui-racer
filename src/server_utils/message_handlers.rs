use std::sync::Arc;

use crate::{
    models,
    server_utils::{
        self,
        fast_storage::{self, GameData, UserGameData},
    },
};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use tokio_tungstenite::tungstenite::protocol;

/// Spawn a task to manage the state and use message passing to operate on it
/// Whenever a message to receiver channel is received, it is forwarded to websocket sender
///
/// Why is this done?
/// If we were to store `websocket_sender` in a struct so that any other user can send messages to
/// this, we would have to hold it behind an Arc<Mutex>>, and .await on MutexGuards is not recommended
///
/// For more information
/// https://tokio.rs/tokio/tutorial/shared-state
///
pub async fn bridge_user_websocket(
    // receiver_channel: UnboundedReceiverStream<UnboundedReceiver<protocol::Message>>,
    receiver_stream: impl StreamExt<Item = models::WSServerMessage>,
    // mut websocket_sender: SplitSink<WebSocketStream<tokio::net::TcpStream>, protocol::Message>,
    websocket_sender: impl SinkExt<protocol::Message>,
) {
    // Why do we need to pin?
    tokio::pin!(receiver_stream, websocket_sender);
    // https://stackoverflow.com/questions/62557219/error-on-future-generator-closure-captured-variable-cannot-escape-fnmut-closu
    // receiver_channel
    //     .fold(websocket_sender, |mut websocket_sender, message| async {
    //         websocket_sender.send(message).await.unwrap();
    //         websocket_sender
    //     })
    //     .await;

    while let Some(message) = receiver_stream.next().await {
        eprintln!("Bridge {message:?}");
        let stringified_message = serde_json::to_string(&message).unwrap();
        websocket_sender
            .send(protocol::Message::Text(stringified_message))
            .await
            .ok();
    }
}

pub async fn handle_client_messages(
    text_message: &str,
    db: Arc<fast_storage::BlazinglyFastDb>,
    current_user_id: &str,
) {
    let parsed_message = serde_json::from_str::<models::WSClientMessage>(text_message)
        .expect("Unable to parse websocket message");
    
    let (message_reply, user_ids) = match parsed_message {
        models::WSClientMessage::Challenge { to_user_id } => {
            // Get the user name and send the challenge to `to_user`
            match db.get_user_by_id(current_user_id).await {
                Some(user_details) => {
                    let message = models::WSServerMessage::RequestForChallenge {
                        from_user: user_details,
                    };
                    (Some(message), Some(vec![to_user_id]))
                }
                None => {
                    eprintln!("User not found {to_user_id}");
                    let error_message = models::WSServerMessage::Error {
                        message: "Requested user cannot be found or is disconnected".to_string(),
                    };
                    (Some(error_message), Some(vec![to_user_id]))
                }
            }
        }
        models::WSClientMessage::UpdateProgress { game_id, progress } => {
            let current_progress = db.find_game_progress(&game_id, current_user_id).await;
             db.update_game_progress(&game_id, current_user_id, progress)
                    .await;
            if progress < 70 && progress - current_progress < 5 {
                eprintln!("Skipping update progress of game_id: {game_id}, current_user_id: {current_user_id}, progress: {progress}");
            } else {
                db.broadcase_game_status(&game_id).await;
            }

            (None, None)
        }
        models::WSClientMessage::AcceptChallenge { opponent_user_id } => {
            // Create a game in the database
            // user1 is the person who created the challenge

            // The user can not be present if he is disconnected, what to do in that case?
            let current_user_connection = db.get_user_connection_by_id(current_user_id).await;
            let opponent_user_connection = db.get_user_connection_by_id(&opponent_user_id).await;

            match (current_user_connection, opponent_user_connection) {
                (Some(user1), Some(user2)) => {
                    let user_game_data1 = UserGameData::new(&user1);
                    let user_game_data2 = UserGameData::new(&user2);

                    let game_data = GameData::new(vec![user_game_data1, user_game_data2]);

                    db.insert_game(game_data.clone()).await;

                    // Inform the users about the starting of game
                    let game_init_message = models::WSServerMessage::GameInit {
                        game_id: game_data.id.clone(),
                        prompt_text: game_data.prompt_text.clone(),
                        starts_at: game_data.starts_at,
                    };

                    // Schedule a tokio task to inform the users about the starting of game
                    let game_start_message = models::WSServerMessage::GameStart;
                    let db_clone = db.clone();

                    let cloned_current_user_id = current_user_id.to_string();
                    let cloned_opponent_user_id = opponent_user_id.clone();

                    let timeout_func = || async move {
                        db_clone
                            .send_message_to_user(
                                &cloned_current_user_id,
                                game_start_message.clone(),
                            )
                            .await;
                        db_clone
                            .send_message_to_user(
                                &cloned_opponent_user_id.clone(),
                                game_start_message,
                            )
                            .await;
                    };

                    tokio::spawn(async {
                        server_utils::set_timeout(10, timeout_func).await;
                    });

                    (
                        Some(game_init_message),
                        Some(vec![current_user_id.to_string(), opponent_user_id]),
                    )
                }
                (Some(_), None) => {
                    // The opponent user is disconnected, send the message to current user
                    eprintln!("User not found {opponent_user_id}");
                    let error_message = models::WSServerMessage::Error {
                        message: "Requested user cannot be found or is disconnected".to_string(),
                    };
                    (Some(error_message), Some(vec![current_user_id.to_string()]))
                }
                (None, Some(_)) => {
                    // The opponent user is disconnected, send the message to current user
                    eprintln!("User not found {current_user_id}");
                    let error_message = models::WSServerMessage::Error {
                        message: "Requested user cannot be found or is disconnected".to_string(),
                    };
                    (Some(error_message), Some(vec![opponent_user_id]))
                }
                (None, None) => (None, None),
            }
        }
    };

    if let Some((message, user_ids)) = message_reply.zip(user_ids) {
        let futures_of_messages = user_ids
            .iter()
            .map(|user_id| db.send_message_to_user(user_id, message.clone()))
            .collect::<Vec<_>>();

        futures_util::future::join_all(futures_of_messages).await;
    }
}
