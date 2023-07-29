pub fn calculate_progress(current_position: usize, total_length: usize) -> u16 {
    let progress = current_position as f64 / total_length as f64;
    (progress * 100.0) as u16
}
