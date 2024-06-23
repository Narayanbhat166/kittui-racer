/// These are the messages that can be sent by server to client.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(tag = "message_type", content = "message")]
#[serde(rename_all = "snake_case")]
pub enum WSServerMessage {
    // Progress of the game ( user_id, progress )
    UserStatus {
        connected_users: Vec<User>,
    },
    SuccessfulConnection {
        user: User,
    },
    RequestForChallenge {
        // Inform the user that a challenge has been raised against him
        from_user: User,
    },
    Error {
        message: String,
    },
    GameInit {
        game_id: String,
        prompt_text: String,
        // Unix timestamp
        starts_at: u64,
    },
    GameStart,
    GameUpdate {
        my_progress: u16,
        opponent_progress: u16,
    },
}

/// These are the messages that are sent by client to server
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(tag = "message_type", content = "message")]
#[serde(rename_all = "snake_case")]
pub enum WSClientMessage {
    Challenge {
        // Raise a challenge to user id
        to_user_id: String,
    },
    AcceptChallenge {
        // Accept the challenge from opponent_user_id
        opponent_user_id: String,
    },
    UpdateProgress {
        game_id: String,
        progress: u16,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum UserStatus {
    Available,
    Busy,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct User {
    pub id: String,
    pub status: UserStatus,
    pub display_name: String,
}

#[derive(Copy, Clone, PartialEq)]
pub enum GameStatus {
    Init,
    InProgress,
    Finished,
}
