use std::{collections::VecDeque, time};

use crossterm::event::KeyCode;
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
};

pub use models::UserStatus;

use crate::{models, ui::stateful_list::StatefulList};

pub struct Layouts {
    pub playground: Rect,
    pub progress_bar: Rect,
    pub bottom_bar: Rect,
}

#[derive(Clone, Copy)]
pub enum TouchState {
    Valid,
    Invalid,
}

#[derive(Default, Clone)]
pub enum CharState {
    #[default]
    /// Either the cursor has not yet reached at the character at this position
    /// or there was a backspace key press.
    Untouched,

    /// This is the current cursor position, the character that has to be pressed next
    CursorPosition,

    /// This is after a key has been pressed, the keypress can either be
    /// `Valid` - Right key was pressed
    /// `Invalid` - Wrong key was pressed
    Touched(TouchState),
}

impl CharState {
    // Get the style to be displayed on terminal
    pub fn get_style(&self) -> Style {
        match self {
            CharState::Untouched => Style::default().add_modifier(Modifier::DIM).fg(Color::Blue),
            CharState::Touched(TouchState::Valid) => Style::default().fg(Color::Yellow),
            CharState::Touched(TouchState::Invalid) => Style::default().fg(Color::LightRed),
            CharState::CursorPosition => Style::default()
                .add_modifier(Modifier::UNDERLINED)
                .add_modifier(Modifier::DIM),
        }
    }
}

// For the client, each user is a player
pub type Player = models::User;

impl Player {
    // Send a challenge message to the player
    pub fn challenge(&self, sender: tokio::sync::mpsc::Sender<UiMessage>) {
        let message = UiMessage::Challenge {
            user_name: self.display_name.clone(),
            user_id: self.id.clone(),
        };
        sender.blocking_send(message).unwrap();
    }
}
pub struct State {
    pub prompt: Vec<PromptKey>,
    // Position of the cursor
    pub cursor_position: u16,
    pub players: StatefulList<Player>,
    pub menu: StatefulList<&'static str>,
    /// Whether the user is currently challenged
    pub challenge: Option<ChallengeData>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            prompt: vec![],
            cursor_position: 0,
            players: StatefulList::with_items(vec![]),
            menu: StatefulList::with_items(vec!["Game", "Practice"]),
            challenge: None,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum Tab {
    // This is where the player can play
    Game,
    // This Tab is where the user can challenge other players
    Arena,
    // This is where the user can choose whether to practice typing or challenge other players
    // This is the default Tab when user initializes the app
    #[default]
    Menu,
}

impl Tab {
    pub fn handle_tab_specific_input(self, app: &mut App, input: KeyCode) -> bool {
        match app.current_tab {
            Tab::Game => super::input_handler::handle_game_input(app, input),
            Tab::Arena => super::input_handler::handle_arena_input(app, input),
            Tab::Menu => super::input_handler::handle_menu_input(app, input),
        }
    }
}

#[derive(Clone)]
pub enum LogType {
    Success,
    Error,
    Info,
}

impl LogType {
    pub fn get_color(&self) -> Color {
        match self {
            LogType::Success => Color::Green,
            LogType::Error => Color::Red,
            LogType::Info => Color::Gray,
        }
    }
}

#[derive(Clone)]
pub struct Event {
    pub log_type: LogType,
    pub message: String,
    pub duration: time::Duration,
    pub created_at: time::Instant,
    pub displayed_at: Option<time::Instant>,
}

impl Event {
    pub fn new(log_type: LogType, message: &str, duration: u8) -> Self {
        Self {
            log_type,
            message: message.to_string(),
            duration: time::Duration::from_secs(u64::from(duration)),
            created_at: time::Instant::now(),
            displayed_at: None,
        }
    }

    /// Update the displayed at time if not previously set
    pub fn check_and_update_display_time(&mut self) -> &mut Self {
        if self.displayed_at.is_none() {
            self.displayed_at = Some(time::Instant::now());
        }
        self
    }

    /// Return true if the event has been displayed on the bottom bar for self.duration time units
    pub fn is_expired(&self) -> bool {
        let current_time = time::Instant::now();
        let event_expiry_time = self.displayed_at.unwrap_or(self.created_at) + self.duration;

        if current_time > event_expiry_time {
            true
        } else {
            false
        }
    }

    /// Returns the modifier with which to display the event
    /// New -> Event is in new state for 75% of it's time since it's display, Display in BOLD
    /// Active -> Display the event without any modifier
    /// Expired -> Display the event in DIM
    pub fn get_display_modifier(&self) -> Modifier {
        let time_elapsed_since_display =
            time::Instant::now() - self.displayed_at.unwrap_or(self.created_at);

        let age_percentage = time_elapsed_since_display.as_secs_f32() / self.duration.as_secs_f32();

        if self.is_expired() {
            Modifier::DIM
        } else if age_percentage <= 0.75 {
            Modifier::BOLD
        } else {
            Modifier::empty()
        }
    }

    /// specify the message to be displayed and duration for which message should be displayed
    pub fn success(message: &str, duration: u8) -> Self {
        Self::new(LogType::Success, message, duration)
    }

    pub fn error(message: &str, duration: u8) -> Self {
        Self::new(LogType::Error, message, duration)
    }

    pub fn info(message: &str, duration: u8) -> Self {
        Self::new(LogType::Info, message, duration)
    }
}

/// App holds the state of the application
pub struct App {
    // The currently active tab. Layout will be same for all the Tabs. Data displayed will be different
    pub current_tab: Tab,
    // The state of application
    pub state: State,
    // User id of the connection
    pub current_user: Option<Player>,
    // A queue of buffered events
    pub events: VecDeque<Event>,
    pub event_sender: tokio::sync::mpsc::Sender<UiMessage>,
}

pub struct PromptKey {
    pub character: char,
    pub state: CharState,
}

impl PromptKey {
    fn new(character: char) -> Self {
        Self {
            character,
            state: CharState::default(),
        }
    }
}

impl App {
    pub fn new(event_sender: tokio::sync::mpsc::Sender<UiMessage>) -> Self {
        let mut transformed_quote_str = String::from("Things we do for love")
            .chars()
            .map(PromptKey::new)
            .collect::<Vec<_>>();

        // Make the first Prompt key underlines to make it appear as cursor
        transformed_quote_str[0].state = CharState::CursorPosition;

        Self {
            current_tab: Tab::default(),
            events: VecDeque::new(),
            current_user: None,
            state: State::default(),
            event_sender,
        }
    }

    pub fn add_log_event(&mut self, event: Event) {
        self.events.push_back(event)
    }

    pub fn accept_current_challenge(&mut self) {
        if let Some(challenge_data) = self.state.challenge.as_ref() {
            let accept_challenge_ui_message = UiMessage::AcceptChallenge {
                user_id: challenge_data.opponent_id.to_owned(),
            };
            self.event_sender
                .blocking_send(accept_challenge_ui_message)
                .unwrap();
        } else {
            let invalid_action_error = Event::error("No active challenges to accept", 1);
            self.add_log_event(invalid_action_error);
        }
    }
}

pub struct ChallengeData {
    pub opponent_id: String,
}

#[derive(Debug)]
pub enum UiMessage {
    ProgressUpdate(usize),
    Challenge {
        /// Username of the current player
        user_name: String,
        /// User id of the player who is challenged
        user_id: String,
    },
    AcceptChallenge {
        /// Username of the opponent
        user_id: String,
    },
}
