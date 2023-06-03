// #![deny(warnings)]
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    RwLock,
};

use tokio_stream::wrappers::UnboundedReceiverStream;

use kittui_racer::{models, server_utils};

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

use std::env;

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::protocol::Message, WebSocketStream};

/// Our state of currently connected users.
///
/// - Key is their id
struct UserConnection {
    sender: mpsc::UnboundedSender<Message>,
    data: models::User,
}

type UserConnections = Arc<RwLock<HashMap<usize, UserConnection>>>;
// type UsersDb = Arc<RwLock<HashMap<usize, models::User>>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let users = UserConnections::default();
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
        tokio::spawn(async move {
            let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("Error during the websocket handshake occurred");
            println!("WebSocket connection established: {}", addr);

            user_connected(ws_stream, cloned_users).await;
        });
    }
}

// fn broadcast_user_status(my_id: usize, users_db: &UsersDb, )

async fn user_connected(ws: WebSocketStream<TcpStream>, users: UserConnections) {
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

    // Save the sender in our list of connected users.
    let new_user = models::User {
        id: my_id,
        status: models::UserStatus::Available,
        display_name: server_utils::generate_name(),
    };

    let successful_connection_message = models::WebsocketMessage::SuccessfulConnection {
        user: new_user.clone(),
    };

    let stringified_message = serde_json::to_string(&successful_connection_message).unwrap();

    // Tell the user about his connection and user id
    send_message(stringified_message, &tx).await;

    let user_connection_details = UserConnection {
        sender: tx.clone(),
        data: new_user,
    };

    users.write().await.insert(my_id, user_connection_details);
    // users_db.write().await.insert(my_id, new_user);

    let all_users = users
        .read()
        .await
        .iter()
        .map(|(_user_id, user_data)| user_data.data.clone())
        .collect::<Vec<_>>();

    broadcast_message(
        my_id,
        models::WebsocketMessage::UserStatus {
            connected_users: all_users,
        },
        &users,
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
    user_disconnected(my_id, &users).await;

    let all_users = users
        .read()
        .await
        .iter()
        .map(|(_user_id, user_data)| user_data.data.clone())
        .collect::<Vec<_>>();

    broadcast_message(
        my_id,
        models::WebsocketMessage::UserStatus {
            connected_users: all_users,
        },
        &users,
        true,
    )
    .await;
}

/// This function will send the message to all users except the one who sent it
async fn broadcast_message(
    my_id: usize,
    msg: models::WebsocketMessage,
    users: &UserConnections,
    send_to_self: bool,
) {
    let stringified_message = serde_json::to_string(&msg).unwrap();
    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        if send_to_self || my_id != uid {
            send_message(stringified_message.clone(), &tx.sender).await
        }
    }
}

async fn send_message(message: String, tx: &UnboundedSender<Message>) {
    println!("{message}");
    if let Err(_disconnected) = tx.send(Message::text(message)) {
        // The tx is disconnected, our `user_disconnected` code
        // should be happening in another task, nothing more to
        // do here.
    }
}

async fn user_disconnected(my_id: usize, users: &UserConnections) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}

static _INDEX_HTML: &str = r#"<!DOCTYPE html>
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
