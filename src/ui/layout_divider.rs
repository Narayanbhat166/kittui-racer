use super::types;
use tui::layout::{Constraint, Direction, Layout, Rect};

pub fn divide_frame(main_frame_size: Rect) -> types::Layouts {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(20), // Used here as a top margin
                Constraint::Min(1),
                Constraint::Length(6),
                Constraint::Length(3),
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

    types::Layouts {
        playground: middle_chunks[1],
        progress_bar: progress_bars[0],
        bottom_bar: main_chunks[3],
    }
}
