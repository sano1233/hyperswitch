//! Utility functions

use time::OffsetDateTime;

/// Format timestamp as ISO 8601 string
pub fn format_timestamp(ts: &OffsetDateTime) -> String {
    ts.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "Invalid timestamp".to_string())
}

/// Calculate percentage
pub fn percentage(part: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    (part as f64 / total as f64) * 100.0
}

/// Calculate average
pub fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Calculate standard deviation
pub fn std_deviation(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let avg = average(values);
    let variance = values.iter()
        .map(|v| (v - avg).powi(2))
        .sum::<f64>() / values.len() as f64;

    variance.sqrt()
}

/// Calculate moving average
pub fn moving_average(values: &[f64], window_size: usize) -> Vec<f64> {
    if values.is_empty() || window_size == 0 {
        return Vec::new();
    }

    let mut result = Vec::new();

    for i in 0..values.len() {
        let start = if i >= window_size { i - window_size + 1 } else { 0 };
        let window = &values[start..=i];
        result.push(average(window));
    }

    result
}

/// Calculate exponential moving average
pub fn exponential_moving_average(values: &[f64], alpha: f64) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(values.len());
    result.push(values[0]);

    for i in 1..values.len() {
        let ema = alpha * values[i] + (1.0 - alpha) * result[i - 1];
        result.push(ema);
    }

    result
}

/// Calculate z-score
pub fn z_score(value: f64, mean: f64, std_dev: f64) -> f64 {
    if std_dev == 0.0 {
        return 0.0;
    }
    (value - mean) / std_dev
}

/// Normalize value to 0-1 range
pub fn normalize(value: f64, min: f64, max: f64) -> f64 {
    if max == min {
        return 0.5;
    }
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

/// Calculate percentile
pub fn percentile(sorted_values: &[f64], percentile: f64) -> Option<f64> {
    if sorted_values.is_empty() || percentile < 0.0 || percentile > 100.0 {
        return None;
    }

    let index = (percentile / 100.0 * (sorted_values.len() - 1) as f64).round() as usize;
    sorted_values.get(index).copied()
}

/// Format duration in human-readable form
pub fn format_duration(seconds: i64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Format bytes in human-readable form
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Generate random ID
pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Retry with exponential backoff
pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_attempts: u32,
    initial_delay_ms: u64,
    multiplier: f64,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut delay = initial_delay_ms;

    for attempt in 1..=max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == max_attempts {
                    return Err(e);
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                delay = (delay as f64 * multiplier) as u64;
            }
        }
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentage() {
        assert_eq!(percentage(50, 100), 50.0);
        assert_eq!(percentage(0, 100), 0.0);
        assert_eq!(percentage(100, 100), 100.0);
        assert_eq!(percentage(1, 0), 0.0); // Division by zero
    }

    #[test]
    fn test_average() {
        assert_eq!(average(&[1.0, 2.0, 3.0]), 2.0);
        assert_eq!(average(&[]), 0.0);
        assert_eq!(average(&[5.0]), 5.0);
    }

    #[test]
    fn test_normalize() {
        assert_eq!(normalize(5.0, 0.0, 10.0), 0.5);
        assert_eq!(normalize(0.0, 0.0, 10.0), 0.0);
        assert_eq!(normalize(10.0, 0.0, 10.0), 1.0);
        assert_eq!(normalize(15.0, 0.0, 10.0), 1.0); // Clamped
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(59), "59s");
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(3661), "1h 1m");
        assert_eq!(format_duration(86400), "1d 0h 0m");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0.00 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
    }
}
