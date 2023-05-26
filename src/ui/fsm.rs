use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;

use crate::ui::types as ui_models;

// This trait should be applied on a window
pub trait Transition {
    fn get_action(&self, input: TransitionInput) -> TransitionAction;
}

#[derive(PartialEq, Clone)]
pub enum Window {
    // This is where the action happens
    MainArea,
    // This will be used for showing the help text and progress
    MultiPurposeBar,
    // Any information to be displayed to user
    BottomBar,
}

#[derive(PartialEq)]
pub enum TransitionInput {
    Key(KeyCode),
    Init,
    Quit,
}

pub enum TransitionAction {
    MoveDown,
    MoveUp,
    Select,
    Unselect,
    Nop,
    Init,
    Quit,
}

impl Window {
    fn get_action(&self, input: TransitionInput) -> TransitionAction {
        match input {
            TransitionInput::Key(KeyCode::Down | KeyCode::Char('j')) => TransitionAction::MoveDown,
            TransitionInput::Key(KeyCode::Up | KeyCode::Char('k')) => TransitionAction::MoveUp,
            TransitionInput::Key(KeyCode::Right | KeyCode::Enter | KeyCode::Char('l')) => {
                TransitionAction::Select
            }
            TransitionInput::Key(KeyCode::Left | KeyCode::Char('h')) => TransitionAction::Unselect,
            TransitionInput::Init => TransitionAction::Init,
            _ => TransitionAction::Nop,
        }
    }

    async fn execute_action(
        &self,
        app: Arc<Mutex<ui_models::App>>,
        action: TransitionAction,
    ) -> Option<Window> {
        None
    }
}
