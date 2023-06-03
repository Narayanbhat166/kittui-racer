use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;

use crate::ui::{
    fsm::TransitionAction,
    types::{self, CharState, TouchState},
};

/// Handle input if current tab is game tab
/// Returns a bool which indicates whether to quit the app or not
///
/// Check whether the entered key is same as expected
/// Update the state of characters based on this
pub fn handle_game_input(app: &mut types::App, input: KeyCode) -> bool {
    let position = app.state.cursor_position.clone() as usize;
    match input {
        KeyCode::Char(character) => {
            if app
                .state
                .prompt
                .get(position)
                .unwrap()
                .character
                .eq(&character)
            {
                app.state.prompt.get_mut(position).unwrap().state =
                    CharState::Touched(TouchState::Valid);
            } else {
                app.state.prompt.get_mut(position).unwrap().state =
                    CharState::Touched(TouchState::Invalid);
            }
            if position + 1 != app.state.prompt.len() {
                app.state.cursor_position += 1;
                app.state.prompt.get_mut(position + 1).unwrap().state = CharState::CursorPosition;
            }
            false
        }
        KeyCode::Backspace => {
            if app.state.cursor_position > 0 {
                // Make current character as next character
                app.state.prompt.get_mut(position).unwrap().state = CharState::Untouched;
                app.state.cursor_position -= 1;
                app.state.prompt.get_mut(position - 1).unwrap().state = CharState::CursorPosition;

                false
            } else {
                true
            }
        }
        KeyCode::Esc => return true,
        _ => false,
    }
}

/// Handle challenging of players, and switching selection using arrow keys
/// Returns a bool which indicates whether to quit the app or not
pub fn handle_arena_input(app: &mut types::App, input: KeyCode) -> bool {
    let action = match input {
        KeyCode::Down | KeyCode::Char('j') => TransitionAction::MoveDown,
        KeyCode::Up | KeyCode::Char('k') => TransitionAction::MoveUp,
        KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => TransitionAction::Select,
        KeyCode::Esc => TransitionAction::Quit,
        _ => TransitionAction::Nop,
    };

    match action {
        TransitionAction::MoveDown => {
            app.state.players.next();
            false
        }
        TransitionAction::MoveUp => {
            app.state.players.previous();
            false
        }
        TransitionAction::Select => {
            // Challenge the player
            // Steps to be taken
            // Send Challenge(player_id), message to be handled by the websocket
            app.state
                .players
                .get_selected_item()
                .map(|selected_player| selected_player.challenge(app.event_sender.clone()));
            false
        }
        TransitionAction::Quit => true,
        _ => false,
    }
}

/// Handle switching between menu options
/// Returns a bool which indicates whether to quit the app or not
pub fn handle_menu_input(app: &mut types::App, input: KeyCode) -> bool {
    let action = match input {
        KeyCode::Down | KeyCode::Char('j') => TransitionAction::MoveDown,
        KeyCode::Up | KeyCode::Char('k') => TransitionAction::MoveUp,
        KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => TransitionAction::Select,
        KeyCode::Esc => TransitionAction::Quit,
        _ => TransitionAction::Nop,
    };

    match action {
        TransitionAction::MoveDown => {
            app.state.menu.next();
            false
        }
        TransitionAction::MoveUp => {
            app.state.menu.previous();
            false
        }
        TransitionAction::Select => app
            .state
            .menu
            .state
            .selected()
            .and_then(|index| {
                match index {
                    0 => app.current_tab = types::Tab::Arena,
                    _ => app.current_tab = types::Tab::Game,
                }
                Some(false)
            })
            .unwrap_or(false),
        TransitionAction::Quit => true,
        _ => false,
    }
}

/// Handle the input for a key event
/// Returns a bool which indicates whether to quit the app or not
pub fn handle_input(app: Arc<Mutex<types::App>>, input: KeyCode) -> bool {
    // Handling of input is dependent on the current tab the user is in.

    match input {
        // On pressing the `Esc` key, the app should quit no matter what tab he is in
        // This logic can be handled for all tabs at a single place
        KeyCode::Esc => true,
        _ => {
            // How other keys behave will be dependent on the current tab
            let mut unlocked_app = app.lock().unwrap();
            unlocked_app
                .current_tab
                .handle_tab_specific_input(&mut unlocked_app, input)
        }
    }
}
