use crossterm::event::KeyCode;
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
};

use crate::ui::models;

pub struct Layouts {
    pub playground: Rect,
    pub progress_bar: Rect,
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

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    pub prompt: Vec<PromptKey>,
    /// The current position of the cursor
    pub position: u16,
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
            prompt: transformed_quote_str,
            position: 0,
            help_text: String::new(),
        }
    }
}

impl Default for App {
    fn default() -> App {
        App {
            prompt: vec![],
            position: 0,
            help_text: String::new(),
        }
    }
}

pub enum UiMessage {
    Hello,
    Input(KeyCode),
}
