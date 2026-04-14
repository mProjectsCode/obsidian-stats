/// Returns the interval at which progress should be logged, based on the total number of items.
/// 
/// Tries to log progress at 10% intervals, but rounds them to 10, 20, 50, 100, 200, 500, 1000 for better readability.
pub fn progress_interval(total: usize) -> usize {
    if total == 0 {
        return 1; // Avoid division by zero, though this case should be handled by should_log_progress
    }

    let mut interval = (total as f64 * 0.1).ceil() as usize;

    // Round to the nearest "nice" number
    let magnitude = 10_usize.pow((interval as f64).log10().floor() as u32);
    let leading_digit = interval / magnitude;

    interval = match leading_digit {
        1 => magnitude,
        2 => 2 * magnitude,
        3..=5 => 5 * magnitude,
        _ => 10 * magnitude,
    };

    interval
}

pub fn should_log_progress(done: usize, total: usize) -> bool {
    if total == 0 {
        return false;
    }

    let interval = progress_interval(total);
    done == total || done.is_multiple_of(interval)
}
