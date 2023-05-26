use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
};

use crate::{
    models::{self as server_models, User},
    ui::stateful_list::StatefulList,
};

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

pub struct State {
    pub prompt: Vec<PromptKey>,
    // Position of the cursor
    pub cursor_position: u16,
    pub players: StatefulList<String>,
    pub menu: StatefulList<&'static str>,
}

impl State {
    pub fn new() -> Self {
        Self {
            prompt: vec![],
            cursor_position: 0,
            players: StatefulList::with_items(vec![]),
            menu: StatefulList::with_items(vec!["Game", "Practice"]),
        }
    }
}

#[derive(Default, Clone, Copy)]
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

/// App holds the state of the application
pub struct App {
    // The currently active tab. Layout will be same for all the Tabs. Data displayed will be different
    pub current_tab: Tab,
    // The state of application
    pub state: State,
    // User id of the connection
    pub user_id: usize,
    pub logs: String,
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
            logs: String::new(),
            user_id: usize::default(),
            state: State::new(),
        }
    }
}

pub enum UiMessage {
    Hello,
    Input(KeyCode),
}
