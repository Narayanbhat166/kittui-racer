use super::models;
use tui::layout::{Constraint, Direction, Layout, Rect};

// This would have to provide different layouts for the current active window
pub fn divide_frame(main_frame_size: Rect) -> models::Layouts {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Min(1),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(main_frame_size);

    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Min(1),
            Constraint::Percentage(20),
        ])
        .split(main_chunks[1]);

    let progress_bars = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[2]);

    models::Layouts {
        playground: middle_chunks[1],
        progress_bar: progress_bars[0],
    }
}
