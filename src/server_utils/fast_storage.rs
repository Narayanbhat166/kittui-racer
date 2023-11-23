use tokio::sync::{
    mpsc::{self},
    RwLock,
};

use crate::models::{self, GameStatus, User};
use std::{
    collections,
    time::{SystemTime, UNIX_EPOCH},
};

/// Currently connected users.
/// Holds a Sender end of the channel to send messages to websocket
#[derive(Clone)]
pub struct UserConnection {
    sender: mpsc::UnboundedSender<models::WSServerMessage>,
    data: models::User,
}

/// Details of users who are currently in a game
#[derive(Clone)]
pub struct UserGameData {
    progress: u16,
    user_id: String,
    sender: mpsc::UnboundedSender<models::WSServerMessage>,
}

impl UserGameData {
    pub fn new(user: &UserConnection) -> Self {
        Self {
            progress: 0,
            user_id: user.data.id.to_owned(),
            sender: user.sender.clone(),
        }
    }
}

#[derive(Clone)]
pub struct GameData {
    pub id: String,
    pub users: Vec<UserGameData>,
    pub status: GameStatus,
    pub prompt_text: String,
    pub starts_at: u64,
}

impl GameData {
    pub fn new(users: Vec<UserGameData>) -> Self {
        // Generate a prompt text, maybe call an api or store all the quotes in a json file
        let prompt_text = "To wear your heart on your sleeve isn't a very good plan; you should wear it inside, where it functions best.".to_string();

        // Start the game after 5 seconds
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let starts_at = current_timestamp + 10;

        let game_id = format!("{}{}", users[0].user_id, users[1].user_id);

        Self {
            id: game_id,
            users,
            status: GameStatus::Init,
            prompt_text,
            starts_at,
        }
    }
}

impl UserConnection {
    pub fn new(user: models::User, sender: mpsc::UnboundedSender<models::WSServerMessage>) -> Self {
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

        eprintln!("boradcasting status {status_message:?}");

        read_lock.values().for_each(|user_connection| {
            user_connection.sender.send(status_message.clone()).ok();
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
        // handle gracefully, the caller should be notified that message was not sent to user
        if let Some(user_connection) = self.get_user_connection_by_id(user_id).await {
            user_connection.sender.send(message).unwrap();
        }
    }

    pub async fn delete_user_connection(&self, user_id: &str) {
        self.users.write().await.remove(user_id);
    }

    pub async fn insert_game(&self, game: GameData) {
        let mut locked_games = self.games.write().await;
        locked_games.insert(game.id.clone(), game);
    }

    pub async fn update_game_progress(&self, game_id: &str, user_id: &str, progress: u16) {
        let mut locked_games = self.games.write().await;
        let current_game = locked_games.get_mut(game_id).unwrap();

        let user_data = current_game
            .users
            .iter_mut()
            .find(|user| user.user_id == user_id)
            .unwrap();

        user_data.progress = progress;
    }

    pub async fn find_game_progress(&self, game_id: &str, user_id: &str) -> u16 {
        let mut locked_games = self.games.write().await;
        let current_game = locked_games.get_mut(game_id).unwrap();

        let user_data = current_game
            .users
            .iter_mut()
            .find(|user| user.user_id == user_id)
            .unwrap();

        user_data.progress
    }

    pub async fn broadcase_game_status(&self, game_id: &str) {
        let locked_games = self.games.read().await;
        let current_game = locked_games.get(game_id).unwrap();

        let user1 = &current_game.users[0];
        let user2 = &current_game.users[1];

        let user1_message = models::WSServerMessage::GameUpdate {
            my_progress: user1.progress,
            opponent_progress: user2.progress,
        };

        let user2_message = models::WSServerMessage::GameUpdate {
            my_progress: user2.progress,
            opponent_progress: user1.progress,
        };

        user1.sender.send(user1_message).unwrap();
        user2.sender.send(user2_message).unwrap();
    }
}
