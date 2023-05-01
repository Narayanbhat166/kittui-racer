use crossterm::event::KeyCode;
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
};

use crate::models as server_models;

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
    Untouched,
    CursorPosition,
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

#[derive(Default)]
pub struct State {
    pub prompt: Vec<PromptKey>,
    pub position: u16,
    pub players: Vec<server_models::User>,
}

#[derive(Default)]
pub enum Tab {
    Game,
    #[default]
    Arena,
}

/// App holds the state of the application
pub struct App {
    pub current_tab: Tab,
    pub state: State,
    /// User id of the connection
    pub user_id: usize,
    pub help_text: String,
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
    pub fn new(quote: String) -> Self {
        let mut transformed_quote_str = quote
            .chars()
            .into_iter()
            .map(PromptKey::new)
            .collect::<Vec<_>>();

        // Make the first Prompt key underlines to make it appear as cursor
        transformed_quote_str[0].state = CharState::CursorPosition;

        Self {
            current_tab: Tab::default(),
            help_text: String::new(),
            user_id: usize::default(),
            state: State::default(),
        }
    }
}

impl Default for App {
    fn default() -> App {
        App {
            current_tab: Tab::default(),
            help_text: String::new(),
            user_id: usize::default(),
            state: State::default(),
        }
    }
}

pub enum UiMessage {
    Hello,
    Input(KeyCode),
}
