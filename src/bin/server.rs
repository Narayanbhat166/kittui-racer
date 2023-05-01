// #![deny(warnings)]
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures_util::{sink::Close, SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    RwLock,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tui::text;
// use warp::ws::{Message, WebSocket};
use warp::Filter;

use kittui_racer::models;

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

use std::{env, io::Error as IoError, net::SocketAddr, sync::Mutex};

use futures_util::{future, pin_mut, stream::TryStreamExt};

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::protocol::Message, WebSocketStream};

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type UserConnections = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;
type UsersDb = Arc<RwLock<HashMap<usize, models::User>>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let users = UserConnections::default();
    let users_db = UsersDb::default();
    // Turn our "state" into a new Filter...
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        let cloned_users = users.clone();
        let cloned_users_db = users_db.clone();
        tokio::spawn(async move {
            let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("Error during the websocket handshake occurred");
            println!("WebSocket connection established: {}", addr);

            user_connected(ws_stream, cloned_users, cloned_users_db).await;
        });
    }
}

// fn broadcast_user_status(my_id: usize, users_db: &UsersDb, )

async fn user_connected(ws: WebSocketStream<TcpStream>, users: UserConnections, users_db: UsersDb) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    let successful_connection_message =
        models::WebsocketMessage::SuccessfulConnection { user_id: my_id };
    let stringified_message = serde_json::to_string(&successful_connection_message).unwrap();

    // Tell the user about his connection and user id
    send_message(stringified_message, &tx).await;

    // Save the sender in our list of connected users.
    let new_user = models::User {
        id: my_id,
        status: models::UserStatus::Available,
    };

    users.write().await.insert(my_id, tx.clone());
    users_db.write().await.insert(my_id, new_user);

    let all_users = users_db
        .read()
        .await
        .iter()
        .map(|(_user_id, user_data)| user_data.clone())
        .collect::<Vec<_>>();

    broadcast_message(
        my_id,
        models::WebsocketMessage::UserStatus {
            connected_users: all_users,
        },
        &users,
        &users_db,
        true,
    )
    .await;

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = user_ws_rx.next().await {
        match result {
            Ok(websocket_user_message) => {
                println!("websocket user message {:?}", websocket_user_message);

                match websocket_user_message {
                    Message::Text(text_message) => {
                        broadcast_message(
                            my_id,
                            models::WebsocketMessage::ChatMessage {
                                user_id: my_id.to_string(),
                                message: text_message,
                            },
                            &users,
                            &users_db,
                            false,
                        )
                        .await;
                    }
                    Message::Close(_close_frame) => {}
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &users, &users_db).await;

    let all_users = users_db
        .read()
        .await
        .iter()
        .map(|(_user_id, user_data)| user_data.clone())
        .collect::<Vec<_>>();

    broadcast_message(
        my_id,
        models::WebsocketMessage::UserStatus {
            connected_users: all_users,
        },
        &users,
        &users_db,
        true,
    )
    .await;
}

/// This function will send the message to all users except the one who sent it
async fn broadcast_message(
    my_id: usize,
    msg: models::WebsocketMessage,
    users: &UserConnections,
    _users_db: &UsersDb,
    send_to_self: bool,
) {
    let stringified_message = serde_json::to_string(&msg).unwrap();
    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        if send_to_self || my_id != uid {
            send_message(stringified_message.clone(), tx).await
        }
    }
}

async fn send_message(message: String, tx: &UnboundedSender<Message>) {
    if let Err(_disconnected) = tx.send(Message::text(message)) {
        // The tx is disconnected, our `user_disconnected` code
        // should be happening in another task, nothing more to
        // do here.
    }
}

// async fn user_message(my_id: usize, msg: models::Message, users: &UserConnections) {
//     // Skip any non-Text messages...
// }

async fn user_disconnected(my_id: usize, users: &UserConnections, users_db: &UsersDb) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
    users_db.write().await.remove(&my_id);
}

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>Warp Chat</title>
    </head>
    <body>
        <h1>Warp chat</h1>
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>
        <input type="text" id="text" />
        <button type="button" id="send">Send</button>
        <script type="text/javascript">
        const chat = document.getElementById('chat');
        const text = document.getElementById('text');
        const uri = 'ws://' + location.host + '/';
        const ws = new WebSocket(uri);

        function message(data) {
            const line = document.createElement('p');
            line.innerText = data;
            chat.appendChild(line);
        }

        ws.onopen = function() {
            chat.innerHTML = '<p><em>Connected!</em></p>';
        };

        ws.onmessage = function(msg) {
            message(msg.data);
        };

        ws.onclose = function() {
            chat.getElementsByTagName('em')[0].innerText = 'Disconnected!';
        };

        send.onclick = function() {
            const msg = text.value;
            ws.send(msg);
            text.value = '';

            message('<You>: ' + msg);
        };
        </script>
    </body>
</html>
"#;
