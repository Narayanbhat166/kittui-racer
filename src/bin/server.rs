use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures_util::StreamExt;
use tokio::sync::mpsc::{self};

use tokio_stream::wrappers::UnboundedReceiverStream;

use kittui_racer::{
    models,
    server_utils::{self, fast_storage},
};

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

use std::env;

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::protocol::Message, WebSocketStream};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // Keep track of all connected users, key is usize,
    // value is a websocket sender.
    let database = Arc::new(fast_storage::BlazinglyFastDb::default());
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:8080".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        let db = database.clone();
        tokio::spawn(async move {
            let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("Error during the websocket handshake occurred");
            println!("WebSocket connection established: {}", addr);

            handle_new_websocket_connection(ws_stream, db).await;
        });
    }
}

// fn broadcast_user_status(my_id: usize, users_db: &UsersDb, )

async fn handle_new_websocket_connection(
    ws: WebSocketStream<TcpStream>,
    db: Arc<fast_storage::BlazinglyFastDb>,
) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed).to_string();

    eprintln!("new chat user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (webs_sender_channel, webs_receiver_channel) = mpsc::unbounded_channel();
    // why is a receiver converted to stream?
    let receiver_stream = UnboundedReceiverStream::new(webs_receiver_channel);

    // Spawn a future to send message to the user, since the sending of messages are async
    // this strategy is used

    tokio::task::spawn(server_utils::message_handlers::bridge_user_websocket(
        receiver_stream,
        user_ws_tx,
    ));

    // Save the sender in our list of connected users.
    let new_user = models::User {
        id: my_id.to_string(),
        status: models::UserStatus::Available,
        display_name: server_utils::generate_name(),
    };

    let successful_connection_message = models::WSServerMessage::SuccessfulConnection {
        user: new_user.clone(),
    };

    let user_connection_details =
        server_utils::fast_storage::UserConnection::new(new_user, webs_sender_channel);

    db.insert_new_user_connection(user_connection_details).await;
    db.send_message_to_user(&my_id, successful_connection_message)
        .await;
    db.boradcast_status().await;

    // Handle the messages sent by the user
    while let Some(result) = user_ws_rx.next().await {
        match result {
            Ok(websocket_user_message) => {
                println!("websocket user message {:?}", websocket_user_message);

                if let Message::Text(text_message) = websocket_user_message {
                    server_utils::message_handlers::handle_client_messages(
                        &text_message,
                        Arc::clone(&db),
                        &my_id.to_string(),
                    )
                    .await;
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
    db.delete_user_connection(&my_id.to_string()).await;
    db.boradcast_status().await;
}
