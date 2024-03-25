//! Module for generating fake measurements related to electricity consumption.
use std::error::Error;
use std::fs::File;

use chrono::{Datelike, Timelike};
use rand::prelude::ThreadRng;
use rand::thread_rng;
use serde::Deserialize;
use sqlx::types::BigDecimal;

use sensor_store::quantity::Quantity;
use sensor_store::unit::Unit;

use crate::measurements::utilities::{
    navigate_to_assets_sensor_data_directory, randomize_value, timestamp_to_date_time,
};
use crate::measurements::{CsvFile, Measurement};

/// Represents a record in "electricity_consumption_monthly_hourly.csv" ([CsvFile::ElectricityConsumptionMonthlyHourly]).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ElectricityConsumptionRecord {
    month: u32,
    hour: u32,
    average_global_active_power: f64,
}

/// A structure for generating fake measurements related to electricity consumption.
pub(super) struct ElectricityConsumptionGenerator {
    /// Energy consumption data, contains 24 entries for each hour of the day for each month.
    data: Vec<f64>,
    /// Used for randomizing the data.
    rng: ThreadRng,
    /// Used to remember if the csv has been loaded into memory.
    data_loaded: bool,
}

impl ElectricityConsumptionGenerator {
    /// Constructor for [ElectricityConsumptionGenerator].
    pub(super) fn new() -> Self {
        Self {
            data: Vec::with_capacity(24 * 12), // for performance: pre-allocate memory
            rng: thread_rng(),
            data_loaded: false,
        }
    }
    /// Load the historic data into self.data.
    fn load_csv(&mut self) -> Result<(), Box<dyn Error>> {
        // make sure the data vector is empty
        self.data.clear();
        // create a path to assets/sensor-data/electricity_consumption_monthly_hourly.csv
        let mut file_path = navigate_to_assets_sensor_data_directory()?;
        file_path.push(CsvFile::ElectricityConsumptionMonthlyHourly.file_name());

        // open the file and create a reader for parsing
        let file = File::open(file_path)?;
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(CsvFile::ElectricityConsumptionMonthlyHourly.delimiter())
            .from_reader(file);

        // Parse the CSV file
        for record_result in reader.deserialize() {
            let record: Result<ElectricityConsumptionRecord, csv::Error> = record_result;
            let record = match record {
                Ok(record) => record,
                Err(err) => {
                    self.data.clear(); // Clear the vector if an error occurs while parsing the record
                    return Err(err.into());
                }
            };
            // ensure that the index at which the data is inserted matches the month and hour in the csv
            let len_vec = self.data.len() as u32;
            let expected_month = len_vec / 24 + 1;
            let expected_hour = len_vec % 24;
            if expected_month != (record.month) || expected_hour != (record.hour) {
                return Err(format!(
                    "Record for month {} and hour {} is missing.",
                    expected_month, expected_hour
                )
                .into());
            }
            // add the data
            self.data.push(record.average_global_active_power);
        }
        // everything was ok! we can set the data_loaded flag to true
        self.data_loaded = true;
        Ok(())
    }

    /// Returns a (fake) electricity consumption value for the specified month and hour,
    /// based on the historic data.
    ///
    /// # Arguments
    ///
    /// * `month` - Value between 0 (January) and 11 (December). Represents the different months.
    /// * `hour` - Value between 0 (00:00) and 23 (23:00). Represents the different hours of the day.
    /// * `variance` - Used to control te randomness of the electricity consumption measurement.
    fn sample(&mut self, month: u32, hour: u32, variance: u64) -> Option<f64> {
        // first load the historic data if not already loaded
        if !self.data_loaded {
            self.load_csv().ok()?;
        }
        // retrieve the historic data value
        let value = self.data[month as usize * 24 + hour as usize] * 60.0;
        // if variance is greater than 0, randomize
        let randomized_value = randomize_value(&mut self.rng, value, variance);
        Some(randomized_value)
    }

    /// Returns a fake electricity consumption measurement for the specified timestamp.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Unix timestamp in seconds. The time at which the fake measurement is taken.
    /// * `variance` - Used to control te randomness of the measurements.
    pub(super) fn get_measurement(&mut self, timestamp: u64, variance: u64) -> Option<Measurement> {
        // convert timestamp to DateTime
        let date_timestamp = timestamp_to_date_time(timestamp).ok()?;
        // retrieve historic value (datetime starts counting months from 1)
        let value = self.sample(date_timestamp.month() - 1, date_timestamp.hour(), variance)?;
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
    use crate::measurements::Measurement;

    /// Tests if the csv concerning energy consumption can be parsed correctly.
    #[test]
    fn test_load_csv() {
        let mut generator = ElectricityConsumptionGenerator::new();
        // Load CSV data
        let result = generator.load_csv();
        // Assert no errors occurred
        assert!(
            result.is_ok(),
            "Error occurred: {:?}",
            result.err().unwrap()
        );
        // Assert vector length is as expected (12 months * 24 hours per month)
        assert_eq!(generator.data.len(), 12 * 24);
    }

    /// Tests to ensure that 'get_measurement' selects the corresponding historic value for the measurement.
    #[test]
    fn test_get_measurement() {
        // specify a time range for which electricity consumption data should be generated
        let date_time: NaiveDateTime = NaiveDate::from_ymd_opt(2024, 3, 2)
            .unwrap()
            .and_hms_opt(20, 44, 44)
            .unwrap();

        let timestamp = date_time.and_utc().timestamp();

        let mut electricity_consumption_data = ElectricityConsumptionGenerator::new();

        // retrieve measurements for specified time range
        let actual_measurement = electricity_consumption_data.get_measurement(timestamp as u64, 0);

        assert!(
            actual_measurement.is_some(),
            "Failed to retrieve the measurement."
        );

        // value comes from the csv directly (*60 for hour)
        let expected_measurement = Measurement {
            unit: Unit::Watt,
            quantity: Quantity::Power,
            value: 2.9169 * 60.0,
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
