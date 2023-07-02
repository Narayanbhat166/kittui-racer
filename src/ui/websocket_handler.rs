use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{models as server_models, ui::types};
const WS_URL: &str = "ws://localhost:8080";

fn handle_incoming_websocket_message(
    app: Arc<Mutex<types::App>>,
    websock_message: server_models::WebsocketMessage,
) {
    let mut unlocked_app = app.lock().unwrap();

    match websock_message {
        server_models::WebsocketMessage::Progress {
            user_id: _,
            progress: _,
        } => {
            //TODO:
        }
        server_models::WebsocketMessage::Challenge {
            // Handle a challenge message from a challenger
            // For the receiver, the meaning of words `challenger` and `challenge` are interchanged
            // Because the challenger will challenge with his user id in `challenger_user_id`
            // This is sent by the Master Cat ( server ), to the right challenger
            // If the receiver ( current user ), received this message, then it implies that
            // The other person challenged current user
            challanger_user_id: _opponent_user_id,
            challengee_user_id: _current_user_id,
            challenger_name: opponent_name,
        } => {
            // Show a prompt for the user to accept / reject the challenge
            // This should last only for few seconds, based on the expiry time
            // TODO: add expiry time
            unlocked_app.add_log_event(types::Event::info(&format!(
                "Challenge received from {}. Accept [A/a] | Reject [R/r]",
                opponent_name
            )));
            unlocked_app.state.is_challenged = true;
        }
        server_models::WebsocketMessage::UserStatus { connected_users } => {
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
        server_models::WebsocketMessage::SuccessfulConnection { user } => {
            let name_assign_log_event = types::Event::success(&format!(
                "Master Cat assigned name {} to you",
                user.display_name
            ));
            unlocked_app.add_log_event(name_assign_log_event);
            // User details of the current user
            unlocked_app.current_user = Some(types::Player {
                id: user.id,
                status: types::UserStatus::Available,
                display_name: user.display_name,
            });
        }
        server_models::WebsocketMessage::ChatMessage {
            user_id: _,
            message: _,
        } => {
            //TODO
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
                    types::Event::success("Websocket connection established");
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
                                serde_json::from_str::<server_models::WebsocketMessage>(&message)
                                    .expect("Cannot parse incoming websocket message");

                            handle_incoming_websocket_message(cloned_app.clone(), message)
                        }
                    })
                    .await
            });

            // If blocking channel ( std::sync::mpsc ) is used, it will block the current thread/task
            // If a single threaded runtime is used, no progress can be made by other tasks
            // So, a tokio channel must is used

            while let Some(ui_message) = ui_message_receiver.recv().await {
                let cloned_app = app.clone();
                let mut unlocked_app = cloned_app.lock().unwrap();
                match ui_message {
                    types::UiMessage::ProgressUpdate(_progress) => {}
                    types::UiMessage::Challenge { user_name, user_id } => {
                        let websocket_message = server_models::WebsocketMessage::Challenge {
                            challanger_user_id: unlocked_app
                                .current_user
                                .as_ref()
                                .unwrap()
                                .id
                                .to_string(),
                            challengee_user_id: user_id,
                            challenger_name: user_name.to_owned(),
                        };

                        let websocket_message_string =
                            serde_json::to_string(&websocket_message).unwrap(); // When can this fail?

                        let res = ws_writer
                            .send(Message::Text(websocket_message_string))
                            .await
                            .map_err(|error| {
                                let event = types::Event::error(&format!(
                                    "Could not send challenge because of error {error:?}"
                                ));
                                event
                            })
                            .map(|_| {
                                let event = types::Event::success(&format!(
                                    "Successfully sent the challenge to {user_name}",
                                ));
                                event
                            });

                        let event = match res {
                            Ok(event) => event,
                            Err(event) => event,
                        };

                        unlocked_app.add_log_event(event);
                    }
                };
            }

            //Todo: handle this unwrap
            ws_reader_handler.await.unwrap();
        }

        Err(socket_connect_error) => {
            let mut app = app.lock().unwrap();
            let error_log_event = types::Event::new(
                types::LogType::Error,
                &format!("Could not create websocket connection {socket_connect_error}"),
            );
            app.add_log_event(error_log_event);
        }
    }
}
