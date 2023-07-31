/// Handlers are defined to send websocket messages
use futures_util::{SinkExt, StreamExt};
use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{models as server_models, ui::types};
const WS_URL: &str = "ws://localhost:8080";

fn count_down_to_zero(app: Arc<Mutex<types::App>>, event: &str, action: &str, duration: u8) {
    let first_event = types::Event::countdown(event, action, duration, true);
    {
        let mut unlocked_app = app.lock().unwrap();
        unlocked_app.add_log_event(first_event);
    }

    (1..duration)
        .rev()
        .map(|duration| types::Event::countdown(event, action, duration, false))
        .for_each(|event| {
            let mut unlocked_app = app.lock().unwrap();
            unlocked_app.add_log_event(event);
        });
}

fn handle_incoming_websocket_message(
    app: Arc<Mutex<types::App>>,
    websock_message: server_models::WSServerMessage,
) {
    match websock_message {
        server_models::WSServerMessage::RequestForChallenge { from_user } => {
            let mut unlocked_app = app.lock().unwrap();
            // Show a prompt for the user to accept / reject the challenge
            // This should last only for few seconds, based on the expiry time
            // TODO: add expiry time
            unlocked_app.add_log_event(types::Event::info(
                &format!(
                    "Challenge received from {}. Accept [A/a] | Reject [R/r]",
                    from_user.display_name
                ),
                5,
                false,
            ));

            let challenge_data = types::ChallengeData {
                opponent_id: from_user.id.to_string(),
            };
            unlocked_app.state.challenge = Some(challenge_data);
        }
        server_models::WSServerMessage::UserStatus { connected_users } => {
            let mut unlocked_app = app.lock().unwrap();
            // filter out current user
            let users_without_current_user = connected_users
                .into_iter()
                .filter(|user| user.id != unlocked_app.current_user.as_ref().unwrap().id)
                .collect();
            unlocked_app
                .state
                .players
                .clear_and_insert_items(users_without_current_user)
        }
        server_models::WSServerMessage::SuccessfulConnection { user } => {
            let mut unlocked_app = app.lock().unwrap();
            let name_assign_log_event = types::Event::success(
                &format!("Master Cat assigned name {} to you", user.display_name),
                1,
                false,
            );
            unlocked_app.add_log_event(name_assign_log_event);
            // User details of the current user
            unlocked_app.current_user = Some(types::Player {
                id: user.id,
                status: types::UserStatus::Available,
                display_name: user.display_name,
            });
        }
        server_models::WSServerMessage::Error { message } => {
            let mut unlocked_app = app.lock().unwrap();
            let error_event_log = types::Event::error(&message, 1, false);
            unlocked_app.add_log_event(error_event_log);
        }
        server_models::WSServerMessage::GameInit {
            game_id,
            prompt_text,
            starts_at,
        } => {
            let seconds_for_game_start = {
                let mut unlocked_app = app.lock().unwrap();
                let ui_game_data = types::UiGameData::new(game_id, prompt_text, starts_at);
                unlocked_app.state.game = Some(ui_game_data);
                unlocked_app.current_tab = types::Tab::Game;

                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                starts_at.saturating_sub(current_time)
            };

            let cloned_app = app.clone();

            count_down_to_zero(
                cloned_app,
                "game",
                "start",
                u8::try_from(seconds_for_game_start).unwrap(),
            );
        }
        server_models::WSServerMessage::GameStart => {
            let mut unlocked_app = app.lock().unwrap();

            if let Some(game_data) = unlocked_app.state.game.as_mut() {
                game_data.status = server_models::GameStatus::InProgress;
            }
            unlocked_app.add_log_event(types::Event::success("Game Started", 10, true));
        }
        server_models::WSServerMessage::GameUpdate {
            my_progress,
            opponent_progress,
        } => {
            let mut unlocked_app = app.lock().unwrap();
            unlocked_app.state.game.as_mut().unwrap().my_progress = my_progress;
            unlocked_app.state.game.as_mut().unwrap().opponent_progress = opponent_progress;
        }
    }
}

/// Handle the websocket events
/// No blocking functions should be executed in this function
pub async fn event_handler(
    app: Arc<Mutex<types::App>>,
    mut ui_message_receiver: tokio::sync::mpsc::Receiver<types::UiMessage>,
) {
    // Handle the ui input in a separate tokio task
    // This is because we do not want the event handler to go down because websocket connection failed
    // The join handlers can then be polled using the join!() macro
    let url = url::Url::parse(WS_URL).expect("Unable to parse the url");

    let connect_socket_result = connect_async(url).await;
    let cloned_app = app.clone();
    match connect_socket_result {
        Ok((socket, _response)) => {
            {
                let connection_success_log =
                    types::Event::success("Websocket connection established", 1, false);
                app.clone()
                    .lock()
                    .unwrap()
                    .add_log_event(connection_success_log);
            }

            let (mut ws_writer, ws_reader) = socket.split();

            let ws_reader_handler = tokio::spawn(async move {
                ws_reader
                    .for_each(|message| async {
                        let message = message.unwrap();
                        if let Message::Text(message) = message {
                            let message =
                                serde_json::from_str::<server_models::WSServerMessage>(&message)
                                    .expect("Cannot parse incoming websocket message");

                            handle_incoming_websocket_message(cloned_app.clone(), message)
                        }
                    })
                    .await
            });

            // If blocking channel ( std::sync::mpsc ) is used, it will block the current thread/task
            // If a single threaded runtime is used, no progress can be made by other tasks
            // So, a tokio channel must is used

            // Async code should never spend a long time without reaching an .await
            // https://ryhl.io/blog/async-what-is-blocking/

            while let Some(ui_message) = ui_message_receiver.recv().await {
                match ui_message {
                    types::UiMessage::AcceptChallenge {
                        user_id: opponent_user_id,
                    } => {
                        let websocket_message =
                            server_models::WSClientMessage::AcceptChallenge { opponent_user_id };

                        let websocket_message_string =
                            serde_json::to_string(&websocket_message).unwrap();

                        let res = ws_writer
                            .send(Message::Text(websocket_message_string))
                            .await
                            .map_err(|error| {
                                types::Event::error(
                                    &format!("Could not send challenge because of error {error:?}"),
                                    1,
                                    true,
                                )
                            })
                            .map(|_| types::Event::success("Accepted challenge", 1, true));

                        let event = match res {
                            Ok(event) => event,
                            Err(event) => event,
                        };

                        app.lock().unwrap().add_log_event(event);
                    }
                    types::UiMessage::ProgressUpdate(_progress) => {}
                    types::UiMessage::Challenge { user_name, user_id } => {
                        let websocket_message = server_models::WSClientMessage::Challenge {
                            to_user_id: user_id,
                        };

                        let websocket_message_string =
                            serde_json::to_string(&websocket_message).unwrap(); // When can this fail?

                        let res = ws_writer
                            .send(Message::Text(websocket_message_string))
                            .await
                            .map_err(|error| {
                                types::Event::error(
                                    &format!("Could not send challenge because of error {error:?}"),
                                    1,
                                    false,
                                )
                            })
                            .map(|_| {
                                types::Event::success(
                                    &format!("Successfully sent the challenge to {user_name}",),
                                    2,
                                    false,
                                )
                            });

                        let event = match res {
                            Ok(event) => event,
                            Err(event) => event,
                        };

                        app.lock().unwrap().add_log_event(event);
                    }
                    types::UiMessage::UpdateProgress { game_id, progress } => {
                        let websocket_message =
                            server_models::WSClientMessage::UpdateProgress { game_id, progress };

                        let websocket_message_string =
                            serde_json::to_string(&websocket_message).unwrap();

                        ws_writer
                            .send(Message::Text(websocket_message_string))
                            .await
                            .unwrap();
                    }
                };
            }

            //Todo: handle this unwrap
            ws_reader_handler.await.unwrap();
        }

        Err(socket_connect_error) => {
            let mut app = app.lock().unwrap();
            let error_log_event = types::Event::error(
                &format!("Could not create websocket connection {socket_connect_error}"),
                1,
                true,
            );
            app.add_log_event(error_log_event);
        }
    }
}
