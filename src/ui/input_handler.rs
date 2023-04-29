use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;

use crate::ui::models::{self, CharState, TouchState};

pub fn handle_input(app: Arc<Mutex<models::App>>, input: KeyCode) -> bool {
    let mut app = app.lock().unwrap();
    let position = app.position.clone() as usize;
    match input {
        KeyCode::Char(character) => {
            if app.prompt.get(position).unwrap().character.eq(&character) {
                app.prompt.get_mut(position).unwrap().state = CharState::Touched(TouchState::Valid);
            } else {
                app.prompt.get_mut(position).unwrap().state =
                    CharState::Touched(TouchState::Invalid);
            }
            if position + 1 != app.prompt.len() {
                app.position += 1;
                app.prompt.get_mut(position + 1).unwrap().state = CharState::CursorPosition;
            }
            false
        }
        KeyCode::Backspace => {
            if app.position > 0 {
                // Make current character as next character
                app.prompt.get_mut(position).unwrap().state = CharState::Untouched;
                app.position -= 1;
                app.prompt.get_mut(position - 1).unwrap().state = CharState::CursorPosition;

                false
            } else {
                true
            }
        }
        KeyCode::Esc => return true,
        _ => false,
    }
}
