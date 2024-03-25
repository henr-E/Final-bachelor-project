//! Module for generating fake measurements related to weather.
use std::error::Error;
use std::fs::File;

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
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

/// Represents a record in "aws_synops_weather_2023_hourly.csv" ([CsvFile::AwsSynopsWeather2023Hourly]).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WeatherRecord {
    month: u32,
    day: u32,
    hour: u32,
    #[serde(flatten)]
    weather_data: WeatherData,
}

/// The data from the record which we will store.
#[derive(Default, Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WeatherData {
    temperature: f64,
    precipitation: f64,
    wind_speed: f64,
    wind_direction: f64,
    cloudiness: f64,
    relative_humidity: f64,
    air_pressure: f64,
}

impl WeatherData {
    /// Maps a [Quantity] to a field in [WeatherData].
    pub fn get_value(&self, quantity: Quantity) -> Result<f64, Box<dyn Error>> {
        let value = match quantity {
            Quantity::Temperature => self.temperature,
            Quantity::Rainfall => self.precipitation,
            Quantity::WindSpeed => self.wind_speed,
            Quantity::WindDirection => self.wind_direction,
            Quantity::Cloudiness => self.cloudiness,
            Quantity::RelativeHumidity => self.relative_humidity,
            Quantity::Pressure => self.air_pressure,
            _ => {
                return Err(format!(
                    "Weather data does not contain info for parameter type {:?}",
                    quantity
                )
                .into());
            }
        };
        Ok(value)
    }
}

/// A structure for generating fake measurements related to weather.
pub(super) struct WeatherDataGenerator {
    /// Weather data for every hour of the year.
    data: Vec<WeatherData>,
    /// Used for randomizing the data.
    rng: ThreadRng,
    data_loaded: bool,
}

impl WeatherDataGenerator {
    /// Constructor for [WeatherDataGenerator].
    pub(super) fn new() -> Self {
        // load data from csv
        Self {
            data: Vec::with_capacity(366 * 24),
            rng: thread_rng(),
            data_loaded: false,
        }
    }

    /// Loads the historic weather data into self.data.
    fn load_csv(&mut self) -> Result<(), Box<dyn Error>> {
        // first clear the data
        self.data.clear();
        // create a path to assets/sensor-data/aws_synops_weather_2023_hourly.csv
        let mut file_path = navigate_to_assets_sensor_data_directory()?;
        file_path.push(CsvFile::AwsSynopsWeather2023Hourly.file_name());

        // open the file and create a reader for parsing
        let file = File::open(file_path)?;
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(CsvFile::AwsSynopsWeather2023Hourly.delimiter())
            .from_reader(file);

        let mut expected_date_time: NaiveDateTime = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        // Parse the CSV file
        for record_result in reader.deserialize() {
            let record: Result<WeatherRecord, csv::Error> = record_result;
            let record = match record {
                Ok(record) => record,
                Err(err) => {
                    self.data.clear(); // Clear the vector if an error occurs while parsing the record
                    return Err(err.into());
                }
            };

            // ensure that the provided month, day and hour in the csv file match with the position in the vector.
            let expected_month = expected_date_time.month();
            let expected_day = expected_date_time.day();
            let expected_hour = expected_date_time.hour();
            if expected_month != (record.month)
                || expected_day != record.day
                || expected_hour != (record.hour)
            {
                return Err(format!(
                    "Record for month {}, day {} and hour {} is missing.",
                    expected_month, expected_day, expected_hour
                )
                .into());
            }
            // push the data
            self.data.push(record.weather_data);
            expected_date_time += Duration::try_hours(1).unwrap();
        }
        self.data_loaded = true;
        Ok(())
    }

    /// Returns a (fake) value for the specified day of the year (0-365), hour (0-23) and [Quantity], based on the historic data.
    ///
    /// # Arguments
    ///
    /// * `day` - Value between 0 and 365. Represents the different days of a leap year.
    /// * `hour` - Value between 0 (00:00) and 23 (23:00). Represents the different hours of the day.
    /// * `variance` - Used to control te randomness of the weather measurement. A value of zero returns the historic data value.
    fn sample(&mut self, day: u32, hour: u32, variance: u64, quantity: Quantity) -> Option<f64> {
        // Load the csv into memory if not already loaded.
        if !self.data_loaded {
            self.load_csv().ok()?;
        }
        // retrieve the corresponding historic value.
        let value = self.data[day as usize * 24 + hour as usize]
            .get_value(quantity)
            .ok()?;
        // randomize (if variance greater than 0).
        let randomized_value = randomize_value(&mut self.rng, value, variance);
        Some(randomized_value)
    }

    /// Returns a fake weather measurement for the specified timestamp and quantity.
    /// # Arguments
    ///
    /// * `timestamp` - Unix timestamp in seconds. Used as a reference point in order to retrieve a value similar to the historic value.
    /// * `variance` - Used to control te randomness of the measurement.
    /// * `quantity` - The [Quantity] for which a measurement should be generated.
    pub(super) fn get_measurement(
        &mut self,
        timestamp: u64,
        variance: u64,
        quantity: Quantity,
    ) -> Option<Measurement> {
        // convert timestamp to DateTime
        let date_timestamp = timestamp_to_date_time(timestamp).ok()?;
        // get a value based on the historic data.
        let value = self.sample(
            date_timestamp.ordinal0(),
            date_timestamp.hour(),
            variance,
            quantity,
        )?;
        // get the unit and prefix of the data present in self.data
        let (unit, prefix) = match quantity {
            Quantity::Pressure => (Unit::Pascal, BigDecimal::from(100)),
            Quantity::Rainfall => (Unit::MillimetersPerHour, BigDecimal::from(1)),
            Quantity::Temperature => (Unit::Celsius, BigDecimal::from(1)),
            Quantity::WindSpeed => (Unit::MetersPerSecond, BigDecimal::from(1)),
            Quantity::WindDirection => (Unit::Degrees, BigDecimal::from(1)),
            Quantity::Cloudiness => (Unit::Okta, BigDecimal::from(1)),
            Quantity::RelativeHumidity => (Unit::Percentage, BigDecimal::from(1)),
            _ => return None,
        };
        // create a measurement
        let measurement: Measurement = Measurement {
            unit,
            quantity,
            value,
            prefix,
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

    use crate::measurements::weather::WeatherDataGenerator;
    use crate::measurements::Measurement;

    /// Tests if the historic data (csv) can be correctly parsed.
    #[test]
    fn test_load_csv() {
        let mut generator = WeatherDataGenerator::new();
        // Load CSV data
        let result = generator.load_csv();
        // Assert no errors occurred
        assert!(
            result.is_ok(),
            "Error occurred: {:?}",
            result.err().unwrap()
        );
        // Assert vector length is as expected (366 days * 24 hours per day)
        assert_eq!(generator.data.len(), 366 * 24);
    }

    /// Tests that the measurements are actually based on historic data.
    #[test]
    fn test_get_measurement() {
        // specify a time range for which electricity consumption data should be generated
        let date_time: NaiveDateTime = NaiveDate::from_ymd_opt(2024, 3, 2)
            .unwrap()
            .and_hms_opt(20, 44, 44)
            .unwrap();

        let timestamp = date_time.and_utc().timestamp();

        let mut weather_data_generator = WeatherDataGenerator::new();

        // Temperature;Precipitation;WindSpeed;WindDirection;Cloudiness;RelativeHumidity;AirPressure
        // 6.05;0.00;7.60;35.5;0;87.38;1023.18

        // retrieve measurements for specified time range (make sure variance is equal to 0 to get historic value.)
        let act_temperature =
            weather_data_generator.get_measurement(timestamp as u64, 0, Quantity::Temperature);

        let exp_temperature = Measurement {
            unit: Unit::Celsius,
            quantity: Quantity::Temperature,
            value: 6.05,
            prefix: BigDecimal::from(1),
        };

        assert!(
            act_temperature.is_some(),
            "Failed to retrieve the measurement."
        );
        assert_eq!(
            act_temperature.unwrap(),
            exp_temperature,
            "Measurement does not match the expected value."
        );

        // retrieve measurements for specified time range
        let act_rainfall =
            weather_data_generator.get_measurement(timestamp as u64, 0, Quantity::Rainfall);

        let exp_rainfall = Measurement {
            unit: Unit::MillimetersPerHour,
            quantity: Quantity::Rainfall,
            value: 0.00,
            prefix: BigDecimal::from(1),
        };

        assert!(
            act_rainfall.is_some(),
            "Failed to retrieve the measurement."
        );
        assert_eq!(
            act_rainfall.unwrap(),
            exp_rainfall,
            "Measurement does not match the expected value."
        );

        // retrieve measurements for specified time range
        let act_wind_speed =
            weather_data_generator.get_measurement(timestamp as u64, 0, Quantity::WindSpeed);

        let exp_wind_speed = Measurement {
            unit: Unit::MetersPerSecond,
            quantity: Quantity::WindSpeed,
            value: 7.60,
            prefix: BigDecimal::from(1),
        };

        assert!(
            act_wind_speed.is_some(),
            "Failed to retrieve the measurement."
        );
        assert_eq!(
            act_wind_speed.unwrap(),
            exp_wind_speed,
            "Measurement does not match the expected value."
        );

        // retrieve measurements for specified time range
        let act_wind_direction =
            weather_data_generator.get_measurement(timestamp as u64, 0, Quantity::WindDirection);

        let exp_wind_direction = Measurement {
            unit: Unit::Degrees,
            quantity: Quantity::WindDirection,
            value: 35.5,
            prefix: BigDecimal::from(1),
        };

        assert!(
            act_wind_direction.is_some(),
            "Failed to retrieve the measurement."
        );
        assert_eq!(
            act_wind_direction.unwrap(),
            exp_wind_direction,
            "Measurement does not match the expected value."
        );

        // retrieve measurements for specified time range
        let act_cloudiness =
            weather_data_generator.get_measurement(timestamp as u64, 0, Quantity::Cloudiness);

        let exp_cloudiness = Measurement {
            unit: Unit::Okta,
            quantity: Quantity::Cloudiness,
            value: 0.0,
            prefix: BigDecimal::from(1),
        };

        assert!(
            act_cloudiness.is_some(),
            "Failed to retrieve the measurement."
        );
        assert_eq!(
            act_cloudiness.unwrap(),
            exp_cloudiness,
            "Measurement does not match the expected value."
        );

        // retrieve measurements for specified time range
        let act_relative_humidity =
            weather_data_generator.get_measurement(timestamp as u64, 0, Quantity::RelativeHumidity);

        let exp_relative_humidity = Measurement {
            unit: Unit::Percentage,
            quantity: Quantity::RelativeHumidity,
            value: 87.38,
            prefix: BigDecimal::from(1),
        };

        assert!(
            act_relative_humidity.is_some(),
            "Failed to retrieve the measurement."
        );
        assert_eq!(
            act_relative_humidity.unwrap(),
            exp_relative_humidity,
            "Measurement does not match the expected value."
        );

        // retrieve measurements for specified time range
        let act_air_pressure =
            weather_data_generator.get_measurement(timestamp as u64, 0, Quantity::Pressure);

        let exp_air_pressure = Measurement {
            unit: Unit::Pascal,
            quantity: Quantity::Pressure,
            value: 1023.18,
            prefix: BigDecimal::from(100),
        };

        assert!(
            act_air_pressure.is_some(),
            "Failed to retrieve the measurement."
        );
        assert_eq!(
            act_air_pressure.unwrap(),
            exp_air_pressure,
            "Measurement does not match the expected value."
        );
    }
}
