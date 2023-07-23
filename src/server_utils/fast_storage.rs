use tokio::sync::{
    mpsc::{self},
    RwLock,
};

use tokio_tungstenite::tungstenite::protocol;

use crate::models::{self, GameStatus, User};
use std::{
    collections,
    time::{SystemTime, UNIX_EPOCH},
};

/// Currently connected users.
/// Holds a Sender end of the channel to send messages to websocket
#[derive(Clone)]
pub struct UserConnection {
    sender: mpsc::UnboundedSender<protocol::Message>,
    data: models::User,
}

/// Details of users who are currently in a game
#[derive(Clone)]
pub struct UserGameData {
    progress: f32,
    user_id: String,
    sender: mpsc::UnboundedSender<protocol::Message>,
}

impl UserGameData {
    pub fn new(user: &UserConnection) -> Self {
        Self {
            progress: 0.0,
            user_id: user.data.id.to_owned(),
            sender: user.sender.clone(),
        }
    }
}

#[derive(Clone)]
pub struct GameData {
    pub id: String,
    pub user1: UserGameData,
    pub user2: UserGameData,
    pub status: GameStatus,
    pub prompt_text: String,
    pub starts_at: u64,
}

impl GameData {
    pub fn new(user1: UserGameData, user2: UserGameData) -> Self {
        // Generate a prompt text, maybe call an api or store all the quotes in a json file
        let prompt_text = "To wear your heart on your sleeve isn't a very good plan; you should wear it inside, where it functions best.".to_string();

        // Start the game after 5 seconds
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let starts_at = current_timestamp + 10;

        let game_id = format!("{}{}", user1.user_id, user2.user_id);

        Self {
            id: game_id,
            user1,
            user2,
            status: GameStatus::Init,
            prompt_text,
            starts_at,
        }
    }
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
    games: GameDetails,
}

type UserConnections = RwLock<collections::HashMap<String, UserConnection>>;
type GameDetails = RwLock<collections::HashMap<String, GameData>>;

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
        let users = &self.users;
        let read_lock = users.read().await;

        read_lock
            .get(user_id)
            .map(|user_connection| user_connection.data.clone())
    }

    pub async fn get_user_connection_by_id(&self, user_id: &str) -> Option<UserConnection> {
        let users = &self.users;
        let read_lock = users.read().await;

        read_lock.get(user_id).cloned()
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

    pub async fn insert_game(&self, game: GameData) {
        let mut locked_games = self.games.write().await;
        locked_games.insert(game.id.clone(), game);
    }
}
