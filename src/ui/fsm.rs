use crossterm::event::KeyCode;

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
    AcceptChallenge,
}
