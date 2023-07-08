use tokio::sync::{
    mpsc::{self},
    RwLock,
};

use tokio_tungstenite::tungstenite::protocol;

use crate::models::{self, User};
use std::collections;

/// Currently connected users.
/// Holds a Sender end of the channel to send messages to websocket
pub struct UserConnection {
    sender: mpsc::UnboundedSender<protocol::Message>,
    data: models::User,
}

impl UserConnection {
    pub fn new(user: models::User, sender: mpsc::UnboundedSender<protocol::Message>) -> Self {
        Self { sender, data: user }
    }
}

/// A simple storage service ( not S3 )
/// This holds the user connections and user data
#[derive(Default)]
pub struct BlazinglyFastDb {
    users: UserConnections,
}

type UserConnections = RwLock<collections::HashMap<String, UserConnection>>;

impl BlazinglyFastDb {
    pub async fn insert_new_user_connection(&self, user_connection: UserConnection) {
        self.users
            .write()
            .await
            .insert(user_connection.data.id.clone(), user_connection);
    }

    /// Boradcast the current user status to all connected users
    pub async fn boradcast_status(&self) {
        let read_lock = self.users.read().await;
        let all_users = read_lock
            .values()
            .map(|user_connection| user_connection.data.to_owned())
            .collect::<Vec<models::User>>();

        let status_message = models::WSServerMessage::UserStatus {
            connected_users: all_users,
        };

        let stringified_message = serde_json::to_string(&status_message).unwrap();
        eprintln!("boradcasting status {stringified_message}");

        read_lock.values().for_each(|user_connection| {
            user_connection
                .sender
                .send(protocol::Message::Text(stringified_message.clone()))
                .ok();
        });
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Option<User> {
        // self.users..read().await.get(&user_id)
        let users = &self.users;
        let read_lock = users.read().await;

        read_lock
            .get(user_id)
            .map(|user_connection| user_connection.data.clone())
    }

    pub async fn send_message_to_user(&self, user_id: &str, message: models::WSServerMessage) {
        let stringified_message = serde_json::to_string(&message).unwrap();

        // handle gracefully, the caller should be notified that message was not sent to user
        self.users.read().await.get(user_id).map(|user_connection| {
            user_connection
                .sender
                .send(protocol::Message::Text(stringified_message))
                .unwrap();
        });
    }

    pub async fn delete_user_connection(&self, user_id: &str) {
        self.users.write().await.remove(user_id);
    }
}
