use futures_util::{SinkExt, StreamExt};
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};

use crate::{
    models as server_models,
    ui::{input_handler, types},
};

pub async fn event_handler(app: Arc<Mutex<types::App>>, receiver: Receiver<types::UiMessage>) {
    let url = url::Url::parse("ws://localhost:3030").expect("Unable to parse the url");

    // This error should be caught and logged
    let (mut socket, response) = connect_async(url).await.unwrap();

    let (mut ws_writer, ws_reader) = socket.split();
    let cloned_app = app.clone();

    tokio::spawn(async move {
        while let Ok(ui_message) = receiver.recv() {
            match ui_message {
                types::UiMessage::Hello => todo!(),
                types::UiMessage::Input(key_event) => {
                    input_handler::handle_input(cloned_app.clone(), key_event);
                }
            }
        }
    });

    ws_writer
        .send(Message::Text("Hola connectored".to_string()))
        .await
        .unwrap();

    ws_reader
        .for_each(|message| async {
            let message = message.unwrap();
            if let Message::Text(message) = message {
                let message = serde_json::from_str::<server_models::WebsocketMessage>(&message)
                    .expect("Cannot parse incoming websocket message");
            }
        })
        .await
}
