use std::sync::{Arc, Mutex};

use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

use crate::ui::types::{App, Layouts};

pub fn draw_ui_from_layout<B: Backend>(
    app: Arc<Mutex<App>>,
    layouts: Layouts,
    frame: &mut Frame<B>,
) {
    let app = app.lock().unwrap();
    let progress = app.position as f64 / app.prompt.len() as f64;
    let progress = (progress * 100.0) as u16;

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC),
        )
        .percent(progress);

    let styles_text = app
        .prompt
        .iter()
        .map(|prompt_key| {
            let mut span = Span::from(prompt_key.character.to_string()); //very bad
            span.style = prompt_key.state.get_style();
            span
        })
        .collect::<Vec<_>>();

    let text = Text::from(Spans::from(styles_text));

    let prompt = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL))
        .alignment(tui::layout::Alignment::Center);

    frame.render_widget(prompt, layouts.playground);

    frame.render_widget(gauge, layouts.progress_bar);
}
