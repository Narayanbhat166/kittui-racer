#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(tag = "message_type", content = "message")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    // Progress of the user ( user_id, progress )
    Progress {
        user_id: String,
        progress: u16,
    },
    Challenge {
        current_user_id: String,
        opponent_user_id: String,
    },
    UserStatus {
        connected_users: Vec<User>,
    },
    SuccessfulConnection {
        user_id: usize,
    },
    ChatMessage {
        user_id: String,
        message: String,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum UserStatus {
    Available,
    Busy,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct User {
    pub id: usize,
    pub status: UserStatus,
}

pub enum GameStatus {
    Init,
    Challenge,
    Progress,
    Finished,
}

pub struct GameData {
    users: Vec<User>,
    status: GameStatus,
}
