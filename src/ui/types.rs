use std::{sync::mpsc, time};

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
    pub fn challenge(&self, current_user: &Player, sender: mpsc::Sender<UiMessage>) {
        let message = UiMessage::Challenge {
            user_name: current_user.display_name.clone(),
            user_id: self.id.to_string(),
        };
        sender.send(message).unwrap();
    }
}
pub struct State {
    pub prompt: Vec<PromptKey>,
    // Position of the cursor
    pub cursor_position: u16,
    pub players: StatefulList<Player>,
    pub menu: StatefulList<&'static str>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            prompt: vec![],
            cursor_position: 0,
            players: StatefulList::with_items(vec![]),
            menu: StatefulList::with_items(vec!["Game", "Practice"]),
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

// #[derive(Default)]
pub struct Event {
    pub log_type: LogType,
    pub message: String,
    pub duration: time::Duration,
    pub created_at: time::Instant,
}

impl Event {
    pub fn new(log_type: LogType, message: &str) -> Self {
        Self {
            log_type,
            message: message.to_string(),
            duration: time::Duration::from_secs(1),
            created_at: time::Instant::now(),
        }
    }

    pub fn success(message: &str) -> Self {
        Self::new(LogType::Success, message)
    }

    pub fn error(message: &str) -> Self {
        Self::new(LogType::Error, message)
    }

    pub fn info(message: &str) -> Self {
        Self::new(LogType::Info, message)
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
    pub events: Option<Event>,
    pub event_sender: mpsc::Sender<UiMessage>,
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
    pub fn new(event_sender: mpsc::Sender<UiMessage>) -> Self {
        let mut transformed_quote_str = String::from("Things we do for love")
            .chars()
            .map(PromptKey::new)
            .collect::<Vec<_>>();

        // Make the first Prompt key underlines to make it appear as cursor
        transformed_quote_str[0].state = CharState::CursorPosition;

        Self {
            current_tab: Tab::default(),
            events: None,
            current_user: None,
            state: State::default(),
            event_sender,
        }
    }

    pub fn add_log_event(&mut self, event: Event) {
        self.events = Some(event)
    }
}

pub enum UiMessage {
    ProgressUpdate(usize),
    Challenge {
        /// Username of the current player
        user_name: String,
        /// User id of the player who is challenged
        user_id: String,
    },
}
