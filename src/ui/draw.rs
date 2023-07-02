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

            frame.render_widget(drawable, playground_area);
        }

        // Draw the Players available for game
        // Do not display the current user
        Tab::Arena => {
            let items = app
                .state
                .players
                .items
                .iter()
                .filter(|player| player.id != app.current_user.as_ref().unwrap().id) // user_id will have been set
                .map(|player| ListItem::new(player.display_name.to_string()))
                .collect::<Vec<_>>();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("List"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("█ ");

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
                .highlight_symbol("█ ");

            frame.render_stateful_widget(list, playground_area, &mut app.state.menu.state)
        }
    }
}

fn draw_bottom_bar<B: Backend>(app: Arc<Mutex<App>>, area: Rect, frame: &mut Frame<B>) {
    let app = app.lock().unwrap();

    let log_color = app
        .events
        .as_ref()
        .map(|event| event.log_type.get_color())
        .unwrap_or(Color::Gray);

    let event_message = app
        .events
        .as_ref()
        .map(|event| event.message.to_owned())
        .unwrap_or("No new events to display".to_string());

    let paragraph_widget = Paragraph::new(Text::from(event_message))
        .style(Style::default().fg(log_color))
        .block(Block::default().borders(Borders::ALL).title("Events"));

    frame.render_widget(paragraph_widget, area)
}

fn draw_progress_bar<B: Backend>(app: Arc<Mutex<App>>, area: Rect, frame: &mut Frame<B>) {
    let app = app.lock().unwrap();

    // Draw progress bar only in game mode
    if app.current_tab == Tab::Game {
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

        frame.render_widget(gauge, area);
    }
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

    // This will be drawn only in case of game mode
    draw_progress_bar(app, layouts.progress_bar, frame);
}
