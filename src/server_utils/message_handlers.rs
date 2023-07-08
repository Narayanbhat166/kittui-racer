use std::sync::Arc;

use crate::{models, server_utils::fast_storage};
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
    receiver_stream: impl StreamExt<Item = protocol::Message>,
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
        eprintln!("Bridge {message}");
        websocket_sender.send(message).await.ok();
    }
}

pub async fn handle_client_messages(
    text_message: &str,
    db: Arc<fast_storage::BlazinglyFastDb>,
    current_user_id: &str,
) {
    let parsed_message = serde_json::from_str::<models::WSClientMessage>(&text_message)
        .expect("Unable to parse websocket message");

    let (message_reply, user_id) = match parsed_message {
        models::WSClientMessage::Challenge { to_user_id } => {
            // Get the user name and send the challenge to `to_user`
            match db.get_user_by_id(&current_user_id).await {
                Some(user_details) => {
                    let message = models::WSServerMessage::RequestForChallenge {
                        from_user: user_details,
                    };
                    (Some(message), Some(to_user_id))
                }
                None => {
                    eprintln!("User not found {to_user_id}");
                    (None, None)
                }
            }
        }
        models::WSClientMessage::AcceptChallenge {
            opponent_user_id: _,
        } => todo!(),
    };

    match (message_reply, user_id) {
        (Some(message), Some(user_id)) => db.send_message_to_user(&user_id, message).await,
        _ => {}
    }
}
