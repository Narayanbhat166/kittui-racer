#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(tag = "message_type", content = "message")]
#[serde(rename_all = "snake_case")]
pub enum WebsocketMessage {
    // Progress of the user ( user_id, progress )
    Progress {
        user_id: String,
        progress: u16,
    },
    Challenge {
        // The current player, who is prompting other user for a game
        challanger_user_id: String,
        // One to whom a challenge is made
        challengee_user_id: String,
        // The name of challenger
        challenger_name: String,
    },
    UserStatus {
        connected_users: Vec<User>,
    },
    SuccessfulConnection {
        user: User,
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
    pub display_name: String,
}

pub enum GameStatus {
    Init,
    Challenge,
    Progress,
    Finished,
}

pub struct GameData {
    _users: Vec<User>,
    _status: GameStatus,
}
