//! Module that provides functionality to retrieve historic data values (assets/sensor-data/historic-data)
use std::error::Error;
use std::fs::File;

use chrono::Timelike;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::RowAccessor;
use rand::prelude::ThreadRng;
use rand::thread_rng;
use serde::Deserialize;

use sensor_store::quantity::Quantity;

use crate::measurements::utilities::{
    navigate_to_assets_sensor_data_directory, randomize_value, timestamp_to_date_time,
};

/// The data from the record which we will store.
#[derive(Default, Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct HistoricDataRecord {
    energy_consumption: f64,
    temperature: f64,
    wind_direction: f64,
    wind_speed: f64,
    precipitation: f64,
    irradiance: f64,
}

impl HistoricDataRecord {
    /// Maps a [Quantity] to a field in [HistoricDataRecord].
    pub fn get_value(&self, quantity: Quantity) -> Result<f64, Box<dyn Error>> {
        let value = match quantity {
            Quantity::Power => self.energy_consumption,
            Quantity::Temperature => self.temperature,
            Quantity::Rainfall => self.precipitation,
            Quantity::WindSpeed => self.wind_speed,
            Quantity::WindDirection => self.wind_direction,
            Quantity::Irradiance => self.irradiance,
            _ => {
                return Err(format!(
                    "HistoricDataRecord does not contain info for parameter type {:?}",
                    quantity
                )
                .into());
            }
        };
        Ok(value)
    }
}

/// A structure for retrieving historical values from assets/sensor-data/historic-data.
pub(super) struct HistoricDataReader {
    /// Historic data for one or more houses every hour. Contains weather data and energy consumption data.
    historic_data: Vec<Vec<HistoricDataRecord>>,
    /// Used for randomizing the data.
    rng: ThreadRng,
    /// Used to keep track of the last file loaded into memory, to prevent unnecessary reloading.
    last_loaded: String,
    /// The minimum nr of records (one for every house) present for an hour.
    min_nr_houses_per_hour: u64,
    /// The current house for which records are being used.
    current_house: u64,
}

impl HistoricDataReader {
    /// Constructor for [HistoricDataReader].
    pub(super) fn new() -> Self {
        Self {
            historic_data: vec![Vec::new(); 24],
            rng: thread_rng(),
            last_loaded: String::new(),
            min_nr_houses_per_hour: 1,
            current_house: 0,
        }
    }

    /// Determine if self.historic_data contains at least one entry for every hour.
    fn historic_data_is_complete(&self) -> bool {
        // Check if historic_data contains at least 24 inner vectors (one or every hour)
        if self.historic_data.len() < 24 {
            return false;
        }
        // Check if at least one record is present for each hour
        for historic_data_hourly in &self.historic_data {
            if historic_data_hourly.is_empty() {
                return false;
            }
        }
        true
    }

    /// Load the historic data into self.historic_data
    fn load_parquet(&mut self, month: u32, day: u32) -> Result<(), Box<dyn Error>> {
        // Determine file name corresponding to the provided month and day
        let file_name = format!("2013-{:02}-{:02}.parquet", month, day);
        // If the file is already loaded into memory, don't load it again.
        if self.last_loaded == file_name {
            return Ok(());
        }

        // Clear all inner vectors
        self.historic_data.clear();
        self.historic_data = vec![Vec::new(); 24];

        // Construct file path
        let mut file_path = match navigate_to_assets_sensor_data_directory() {
            Ok(f) => f,
            Err(e) => {
                tracing::error!("could not find sensor data directory");
                return Err(e);
            }
        };
        file_path.push("historic-data");
        file_path.push(file_name.clone());

        let file = File::open(file_path)?;
        let reader = SerializedFileReader::new(file)?;
        let row_iter = reader.get_row_iter(None).unwrap();

        // Loop over records
        for record_result in row_iter {
            match record_result {
                Ok(record) => {
                    // Determine which hour is represented by the timestamp
                    let timestamp_ns = record.get_long(7)? as u64;
                    let timestamp_s = timestamp_ns / 1_000_000_000;
                    let date_time = timestamp_to_date_time(timestamp_s)?;
                    let hour = date_time.hour();

                    // Read the required fields from the record in the right format
                    let energy_consumption = record.get_double(1)?;
                    let temperature = record.get_double(2)?;
                    let wind_direction = record.get_double(3)?;
                    let wind_speed = record.get_double(4)?;
                    let precipitation = record.get_double(5)?;
                    let irradiance = record.get_double(6)?;

                    let weather_data_record = HistoricDataRecord {
                        energy_consumption,
                        temperature,
                        wind_direction,
                        wind_speed,
                        precipitation,
                        irradiance,
                    };
                    self.historic_data[hour as usize].push(weather_data_record);
                }
                Err(err) => {
                    // The row could not be parsed.
                    return Err(err.into());
                }
            }
        }
        // Determine if at least one record was present for every hour.
        if !self.historic_data_is_complete() {
            return Err(format!(
                "File {} does not contain at least one record for every hour.",
                file_name
            )
            .into());
        }
        // Loading was successful, update last_loaded
        self.last_loaded = file_name;
        // initialize the min_nr_houses and current_house (later used for round-robin)
        self.min_nr_houses_per_hour = self
            .historic_data
            .iter()
            .map(|inner_vec| inner_vec.len())
            .min()
            .ok_or(".historic_data has no elements.")? as u64;
        self.current_house = 0;
        Ok(())
    }

    /// Returns a (randomized) value for the specified month (1-12), month day (starting from 1), hour (0-23) and [Quantity], based on the historic data.
    /// Returns None if the required historic data could not be loaded or if the quantity is not present in the data.
    ///
    /// # Arguments
    ///
    /// * `month` - Value between 1 and 12. Represents the different months.
    /// * `day` - Value greater than 0. Represents the day of the month..
    /// * `hour` - Value between 0 (00:00) and 23 (23:00). Represents the different hours of the day.
    /// * `variance` - Used to control te randomness of the data value. A value of zero returns the historic data value.
    /// * `quantity` - The quantity for which the value should be sampled.
    pub(super) fn sample(
        &mut self,
        month: u32,
        day: u32,
        hour: u32,
        variance: u64,
        quantity: Quantity,
    ) -> Option<f64> {
        let target_day = if month == 2 && day == 29 {
            28 // leap year adjustment
        } else {
            day
        };
        // Load the parquet file into memory (if not already)
        match self.load_parquet(month, target_day) {
            Ok(_) => {}
            Err(_) => {
                // if the parquet file is incomplete or can not be loaded, clear the data and return None
                self.historic_data.clear();
                self.historic_data = vec![Vec::new(); 24];
                return None;
            }
        }
        // Round-robin, subsequent calls should yield values from different households
        let next_house = self.current_house;
        self.current_house = (self.current_house + 1) % self.min_nr_houses_per_hour;

        // retrieve the corresponding historic value.
        let value = self.historic_data[hour as usize][next_house as usize]
            .get_value(quantity)
            .ok()?;
        // randomize (if variance greater than 0).
        let randomized_value = randomize_value(&mut self.rng, value, variance);
        Some(randomized_value)
    }
}

#[cfg(test)]
mod tests {
    use super::HistoricDataReader;

    /// Tests if the historic data (parquets) can be correctly parsed.
    #[test]
    fn test_load_parquet() {
        // Load parquet
        let mut historic_data_reader = HistoricDataReader::new();
        let result = historic_data_reader.load_parquet(1, 1);
        // Assert no errors occurred
        assert!(
            result.is_ok(),
            "Error occurred: {:?}",
            result.err().unwrap()
        );
    }
}
