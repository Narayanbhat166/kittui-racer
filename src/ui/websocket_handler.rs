use futures_util::{join, SinkExt, StreamExt};
use std::{
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread::JoinHandle,
};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};

use crate::{
    models as server_models,
    ui::{input_handler, types},
};

pub async fn event_handler(app: Arc<Mutex<types::App>>, receiver: Receiver<types::UiMessage>) {
    // Handle the ui input in a separate tokio task
    // This is because we do not want the event handler to go down because websocket connection failed
    // The join handlers can then be polled using the join!() macro
    let app_for_input_handler = app.clone();
    let input_handler = tokio::spawn(async move {
        while let Ok(ui_message) = receiver.recv() {
            match ui_message {
                types::UiMessage::Hello => todo!(),
                types::UiMessage::Input(key_event) => {
                    input_handler::handle_input(app_for_input_handler.clone(), key_event);
                }
            }
        }
    });

    let ws_handler = {
        let url = url::Url::parse("ws://localhost:3030").expect("Unable to parse the url");

        // This error should be caught and logged
        let connect_socket_result = connect_async(url).await;
        match connect_socket_result {
            Ok((mut socket, response)) => {
                let (mut ws_writer, ws_reader) = socket.split();
                let cloned_app = app.clone();

                ws_writer
                    .send(Message::Text("Hola connectored".to_string()))
                    .await
                    .unwrap();

                tokio::spawn(async {
                    ws_reader
                        .for_each(|message| async {
                            let message = message.unwrap();
                            if let Message::Text(message) = message {
                                let message =
                                    serde_json::from_str::<server_models::WebsocketMessage>(
                                        &message,
                                    )
                                    .expect("Cannot parse incoming websocket message");
                            }
                        })
                        .await
                })
            }
            Err(socket_connect_error) => {
                let mut app = app.lock().unwrap();
                app.logs = socket_connect_error.to_string();
                // Things we do for love and to please the rust compiler
                tokio::spawn(async {})
            }
        }
    };

    _ = join!(input_handler, ws_handler);
}
