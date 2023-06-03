use futures_util::{join, SinkExt, StreamExt};
use std::sync::{mpsc::Receiver, Arc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    models as server_models,
    ui::{input_handler::handle_arena_input, types},
};
const WS_URL: &'static str = "ws://localhost:8080";

fn handle_incoming_websocket_message(
    app: Arc<Mutex<types::App>>,
    websock_message: server_models::WebsocketMessage,
) {
    let mut unlocked_app = app.lock().unwrap();

    match websock_message {
        server_models::WebsocketMessage::Progress { user_id, progress } => todo!(),
        server_models::WebsocketMessage::Challenge {
            current_user_id,
            opponent_user_id,
        } => todo!(),
        server_models::WebsocketMessage::UserStatus { connected_users } => {
            let players = connected_users.iter().map(|user| )
        }
        server_models::WebsocketMessage::SuccessfulConnection { user_id } => {
            // This is the user id of the client, store it in app state
            unlocked_app.user_id = user_id;
            let log_message = types::Logs::new(
                types::LogType::Info,
                &format!("Assigned user id {}", user_id.to_string()),
            );
            unlocked_app.logs = log_message;
        }
        server_models::WebsocketMessage::ChatMessage { user_id, message } => todo!(),
    }
}

pub async fn event_handler(app: Arc<Mutex<types::App>>, receiver: Receiver<types::UiMessage>) {
    // Handle the ui input in a separate tokio task
    // This is because we do not want the event handler to go down because websocket connection failed
    // The join handlers can then be polled using the join!() macro
    let url = url::Url::parse(WS_URL).expect("Unable to parse the url");

    // This error should be caught and logged
    let connect_socket_result = connect_async(url).await;
    match connect_socket_result {
        Ok((socket, response)) => {
            {
                let connection_success_log =
                    types::Logs::new(types::LogType::Success, "Connection established");
                app.lock().unwrap().logs = connection_success_log;
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

                            handle_incoming_websocket_message(app.clone(), message)
                        }
                    })
                    .await
            });

            let app_message_handler = tokio::spawn(async move {
                while let Ok(ui_message) = receiver.recv() {
                    match ui_message {
                        types::UiMessage::ProgressUpdate(progress) => {}
                        types::UiMessage::Challenge(player_id) => {}
                    }
                }
            });

            _ = join!(ws_reader_handler, app_message_handler)
        }
        Err(socket_connect_error) => {
            let mut app = app.lock().unwrap();
            let error_log =
                types::Logs::new(types::LogType::Error, &socket_connect_error.to_string());
            app.logs = error_log;
        }
    }
}
