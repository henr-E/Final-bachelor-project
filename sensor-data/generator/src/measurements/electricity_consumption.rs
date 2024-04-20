//! Module for generating fake measurements related to electricity consumption.

use chrono::{Datelike, Timelike};
use sqlx::types::BigDecimal;

use sensor_store::quantity::Quantity;
use sensor_store::unit::Unit;

use crate::measurements::read_historic_data::HistoricDataReader;
use crate::measurements::utilities::timestamp_to_date_time;
use crate::measurements::Measurement;

/// A structure for generating fake measurements related to electricity consumption.
pub(super) struct ElectricityConsumptionGenerator {}

impl ElectricityConsumptionGenerator {
    /// Constructor for [ElectricityConsumptionGenerator].
    pub(super) fn new() -> Self {
        Self {}
    }

    /// Returns a fake electricity consumption measurement for the specified timestamp.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Unix timestamp in seconds. The time at which the fake measurement is taken.
    /// * `variance` - Used to control te randomness of the measurements.
    /// * `historic_data_reader` - Used to read historic data values from assets
    pub(super) fn get_measurement(
        &mut self,
        timestamp: u64,
        variance: u64,
        historic_data_reader: &mut HistoricDataReader,
    ) -> Option<Measurement> {
        // convert timestamp to DateTime
        let date_timestamp = timestamp_to_date_time(timestamp).ok()?;
        // retrieve historic value (datetime starts counting months from 1)
        let value = historic_data_reader.sample(
            date_timestamp.month(),
            date_timestamp.day(),
            date_timestamp.hour(),
            variance,
            Quantity::Power,
        )?;
        // create the measurement (in kiloWatt)
        let measurement: Measurement = Measurement {
            unit: Unit::Watt,
            quantity: Quantity::Power,
            value,
            prefix: BigDecimal::from(1000),
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

    use crate::measurements::electricity_consumption::ElectricityConsumptionGenerator;
    use crate::measurements::read_historic_data::HistoricDataReader;
    use crate::measurements::Measurement;

    /// Tests to ensure that 'get_measurement' selects the corresponding historic value for the measurement.
    #[test]
    fn test_get_measurement() {
        // specify a time range for which electricity consumption data should be generated
        let date_time: NaiveDateTime = NaiveDate::from_ymd_opt(2024, 3, 2)
            .unwrap()
            .and_hms_opt(20, 44, 44)
            .unwrap();

        let timestamp = date_time.and_utc().timestamp();

        let mut historic_data_reader = HistoricDataReader::new();

        let mut electricity_consumption_data = ElectricityConsumptionGenerator::new();

        // retrieve measurements for specified time range
        let actual_measurement = electricity_consumption_data.get_measurement(
            timestamp as u64,
            0,
            &mut historic_data_reader,
        );

        assert!(
            actual_measurement.is_some(),
            "Failed to retrieve the measurement."
        );

        // value comes from the parquet file assets/sensor-data/historic-data/2013-03-02.parquet at 20:00:00 first house
        let expected_measurement = Measurement {
            unit: Unit::Watt,
            quantity: Quantity::Power,
            value: 0.964,
            prefix: BigDecimal::from(1000),
        };

        // ensure that the historic measurement value is used
        assert_eq!(
            actual_measurement.unwrap(),
            expected_measurement,
            "Measurement does not match the expected value."
        );

        // retrieve measurements for specified time range
        let actual_measurement = electricity_consumption_data.get_measurement(
            timestamp as u64,
            0,
            &mut historic_data_reader,
        );

        assert!(
            actual_measurement.is_some(),
            "Failed to retrieve the measurement."
        );

        // value comes from the parquet file assets/sensor-data/historic-data/2013-03-02.parquet at 20:00:00 second house
        let expected_measurement = Measurement {
            unit: Unit::Watt,
            quantity: Quantity::Power,
            value: 0.484,
            prefix: BigDecimal::from(1000),
        };

        // ensure that the historic measurement value is used
        assert_eq!(
            actual_measurement.unwrap(),
            expected_measurement,
            "Measurement does not match the expected value."
        );
    }
}
