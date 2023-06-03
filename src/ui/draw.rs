use std::sync::{Arc, Mutex};

use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::ui::types::{App, Layouts, Tab};

fn draw_playground<B: Backend>(app: Arc<Mutex<App>>, playground_area: Rect, frame: &mut Frame<B>) {
    // App locking is required for all the tab variants
    let mut app = app.lock().unwrap();

    match app.current_tab {
        // Draw the Typeracer UI with characters
        Tab::Game => {
            let styles_text = app
                .state
                .prompt
                .iter()
                .map(|prompt_key| {
                    let mut span = Span::from(prompt_key.character.to_string()); //very bad
                    span.style = prompt_key.state.get_style();
                    span
                })
                .collect::<Vec<_>>();

            let text = Text::from(Spans::from(styles_text));

            let drawable = Paragraph::new(text)
                .wrap(Wrap { trim: true })
                .block(Block::default().borders(Borders::ALL))
                .alignment(tui::layout::Alignment::Center);

            frame.render_widget(drawable, playground_area)
        }

        // Draw the Players available for game
        Tab::Arena => {
            let items = app
                .state
                .players
                .items
                .iter()
                .map(|player| ListItem::new(player.id.to_string()))
                .collect::<Vec<_>>();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("List"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");

            frame.render_stateful_widget(list, playground_area, &mut app.state.players.state)
        }

        // Draw the menu, Options are whether to play the game or practice
        // If practice is selected -> Take user to Game page
        // If game is selected -> Take user to Arena
        Tab::Menu => {
            let list_items = app
                .state
                .menu
                .items
                .iter()
                .map(|player_id| ListItem::new(player_id.to_owned()))
                .collect::<Vec<_>>();

            let list = List::new(list_items)
                .block(Block::default().borders(Borders::ALL).title("Menu"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, playground_area, &mut app.state.menu.state)
        }
    }
}

fn draw_bottom_bar<B: Backend>(app: Arc<Mutex<App>>, area: Rect, frame: &mut Frame<B>) {
    let app = app.lock().unwrap();

    let log_color = match app.logs.log_type {
        super::types::LogType::Success => Color::Green,
        super::types::LogType::Error => Color::Red,
        super::types::LogType::Info => Color::Gray,
    };

    let paragraph_widget = Paragraph::new(Text::from(app.logs.message.to_string()))
        .style(Style::default().fg(log_color))
        .block(Block::default().borders(Borders::ALL).title("Logs"));

    frame.render_widget(paragraph_widget, area)
}

/// Draw the UI from layout
/// Based on the current active tab, Data drawn will be different

pub fn draw_ui_from_layout<B: Backend>(
    app: Arc<Mutex<App>>,
    layouts: Layouts,
    frame: &mut Frame<B>,
) {
    draw_playground(app.clone(), layouts.playground, frame);
    draw_bottom_bar(app.clone(), layouts.bottom_bar, frame);

    let app = app.lock().unwrap();
    let progress = app.state.cursor_position as f64 / app.state.prompt.len() as f64;
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

    frame.render_widget(gauge, layouts.progress_bar);
}
