pub fn calculate_progress() -> f64 {
    let progress = app.state.cursor_position as f64 / prompt_text.len() as f64;
    let progress = (progress * 100.0) as u16;
}
