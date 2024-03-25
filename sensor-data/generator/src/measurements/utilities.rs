//! Module that contains some helper functions.
use std::error::Error;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use rand::prelude::ThreadRng;
use rand::Rng;

use environment_config::env;

fn walk_dir_tree_for_assets(file: &str) -> Option<PathBuf> {
    let file = std::path::PathBuf::from(file);
    let mut current_dir = std::env::current_dir().ok()?;

    while !current_dir.join(&file).exists() {
        // Go up a directory.
        current_dir = current_dir.join("../");

        if !current_dir.exists() {
            // We cannot go higher up the tree.
            return None;
        }
    }
    Some(current_dir.join(file))
}

/// Helper function that returns the path to the assets/sensor-data directory relative to the cargo manifest directory.
pub(super) fn navigate_to_assets_sensor_data_directory() -> Result<PathBuf, Box<dyn Error>> {
    Ok(walk_dir_tree_for_assets(env("ASSETS_SENSOR_DATA_PATH")?).ok_or("Directory not found.")?)
}

/// Helper function that converts a unix timestamp to a DateTimeObject.
///
/// # Arguments
///
/// * `timestamp` - Unix timestamp in seconds.
///
/// # Errors
///
/// * Failed to convert timestamp to NaiveDateTime.
pub(super) fn timestamp_to_date_time(timestamp: u64) -> Result<DateTime<Utc>, Box<dyn Error>> {
    let datetime_utc = DateTime::from_timestamp(timestamp as i64, 0)
        .ok_or("Failed to convert timestamp to DateTime")?;
    Ok(datetime_utc)
}

/// Function that randomizes a value based on the variance. Only randomize the value if the value and variance are not equal to zero.
pub(super) fn randomize_value(rng: &mut ThreadRng, value: f64, variance: u64) -> f64 {
    // if variance is greater than 0, randomize
    if variance > 0 && value != 0.0 {
        // determine the range from which a random value should be generated
        let min = value - (value.abs() / variance as f64);
        let max = value + (value.abs() / variance as f64);
        rng.gen_range(min..max)
    } else {
        // if variance is not greater than 0, return the value without randomization
        value
    }
}

#[cfg(test)]
mod tests {
    use super::randomize_value;

    /// Tests that the randomized value is contained in the range defined by the variance parameter.
    #[test]
    fn test_randomize_value_with_variance() {
        let mut rng = rand::thread_rng();
        let value = 10.0;
        let variance = 2;

        // value should be between "value - value.abs() / variance" and "value + value.abs() / variance"
        let randomized_value = randomize_value(&mut rng, value, variance);
        assert!(randomized_value >= value - value.abs() / variance as f64);
        assert!(randomized_value <= value + value.abs() / variance as f64);
    }

    /// Ensures that a variance of 0 does not randomize the value.
    #[test]
    fn test_randomize_value_without_variance() {
        let mut rng = rand::thread_rng();
        let value = 10.0;
        let variance = 0; // variance equal to 0

        // If variance is equal to 0, no randomization should take place.
        let randomized_value = randomize_value(&mut rng, value, variance);
        assert_eq!(randomized_value, value);
    }
}
