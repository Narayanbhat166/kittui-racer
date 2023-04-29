use futures_util::{SinkExt, StreamExt};
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};

use crate::ui::{input_handler, models};

pub async fn handler(app: Arc<Mutex<models::App>>, receiver: Receiver<models::UiMessage>) {
    let url = url::Url::parse("ws://localhost:3030").expect("Unable to parse the url");
    let (mut socket, response) = connect_async(url).await.unwrap();

    let (mut write, read) = socket.split();
    let cloned_app = app.clone();

    tokio::spawn(async move {
        while let Ok(ui_message) = receiver.recv() {
            match ui_message {
                models::UiMessage::Hello => todo!(),
                models::UiMessage::Input(key_event) => {
                    input_handler::handle_input(cloned_app.clone(), key_event);
                }
            }
        }
    });

    write
        .send(Message::Text("Hola connectored".to_string()))
        .await
        .unwrap();

    read.for_each(|message| async {
        let message = message.unwrap();
        if let Message::Text(message) = message {
            // app.lock().unwrap().help_text = message;
        }
    })
    .await
}
