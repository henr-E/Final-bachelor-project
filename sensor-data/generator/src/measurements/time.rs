//! Module for generating measurements related to time.
use sqlx::types::BigDecimal;

use sensor_store::quantity::Quantity;
use sensor_store::unit::Unit;

use crate::measurements::Measurement;

/// Structure used for generating measurements related to time.
pub(super) struct TimestampGenerator {}

impl TimestampGenerator {
    /// Returns a measurement with a value corresponding to the provided timestamp.
    pub(super) fn get_measurement(&self, timestamp: u64) -> Option<Measurement> {
        let measurement: Measurement = Measurement {
            unit: Unit::Utc,
            quantity: Quantity::Timestamp,
            value: timestamp as f64,
            prefix: BigDecimal::from(1),
        };
        Some(measurement)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveDateTime};
    use sqlx::types::BigDecimal;

    use sensor_store::quantity::Quantity;
    use sensor_store::unit::Unit;

    use crate::measurements::time::TimestampGenerator;
    use crate::measurements::Measurement;

    /// This test ensures that the returned measurement of 'get_measurement()' corresponds to the unit and the provided timestamp.
    #[test]
    fn test_get_measurement() {
        let date_time: NaiveDateTime = NaiveDate::from_ymd_opt(2024, 3, 2)
            .unwrap()
            .and_hms_opt(20, 44, 44)
            .unwrap();

        let timestamp = date_time.and_utc().timestamp();

        let timestamp_generator = TimestampGenerator {};

        // retrieve measurements for specified time range
        let act_timestamp = timestamp_generator.get_measurement(timestamp as u64);

        let exp_timestamp = Measurement {
            unit: Unit::Utc,
            quantity: Quantity::Timestamp,
            value: 1709412284.0,
            prefix: BigDecimal::from(1),
        };

        assert!(
            act_timestamp.is_some(),
            "Failed to retrieve the measurement for quantity Time."
        );
        assert_eq!(
            act_timestamp.unwrap(),
            exp_timestamp,
            "Timestamp measurement does not match the expected value."
        );
    }
}
