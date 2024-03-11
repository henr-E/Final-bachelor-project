//! This module contains definitions for sensors and their data output formats.
//! The sensors are categorized by type, mainly referring to the specific weather/energy related parameter(s) being measured.
//!
//! Each 'Sensor' must implement a 'generate' function. This function generates the data collected by the 'Sensor' ('SensorOutput') within the specified time range.
//! Each 'SensorOutput' must implement a 'to_json' function. This function converts the 'SensorOutput' to json.
//!

use serde::{Deserialize, Serialize};

/// Represents geographic coordinates with latitude and longitude.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

/// A trait for sharing functionality related to sensor output
pub trait SensorOutput {
    /// Returns a JSON string representation of the struct.
    ///
    /// # Panics
    ///
    /// The sensor could not successfully be converted to a JSON string.
    ///
    fn to_json(&self) -> String;
}

/// A trait for sharing functionality related to sensors themselves.
pub trait Sensor {
    /// Generate a string vector containing sensor data recorded between the specified time frame, from 'begin' to 'end'. Each element in the vector represents a distinct data output from the sensor.
    ///
    /// # Arguments
    ///
    /// * `begin` - Unix timestamp in seconds, the start of the timeframe
    /// * `end` - Unix timestamp in seconds, the end of the timeframe
    fn generate(&self, begin: u64, end: u64) -> Vec<String>;
}

/// Module containing temperature-related functionality.
pub mod temperature {
    use super::*;

    /// Represents a concrete temperature sensor called [SensorTemperature]
    pub struct SensorTemperature {
        /// Geographical coordinates of the sensor.
        pub coordinates: Coordinates,
        /// The time interval between sensor readings (in seconds).
        pub interval: u32,
    }

    /// Represents the output of the concrete temperature sensor called [SensorTemperature]
    #[derive(Serialize, Deserialize, Debug)]
    pub struct SensorTemperatureOutput {
        /// The geographic coordinates of the sensor.
        pub coordinates: Coordinates,
        /// The timestamp of the sensor reading.
        pub timestamp: u64,
        /// The time interval between sensor readings (in seconds).
        pub interval: u32,
        /// The temperature value recorded by the sensor (in Celsius).
        pub temperature: f64,
    }

    impl SensorOutput for SensorTemperatureOutput {
        fn to_json(&self) -> String {
            serde_json::to_string(self).unwrap()
        }
    }

    /// Specific implementation for the concrete temperature sensor called [SensorTemperature].
    impl Sensor for SensorTemperature {
        fn generate(&self, begin: u64, end: u64) -> Vec<String> {
            let mut results = Vec::new();
            let mut timestamp = begin;

            while timestamp <= end {
                let output = SensorTemperatureOutput {
                    coordinates: self.coordinates.clone(),
                    timestamp,
                    interval: self.interval,
                    temperature: 20.0,
                };
                results.push(output.to_json());
                timestamp += self.interval as u64;
            }
            results
        }
    }
}

/// Module containing precipitation-related functionality.
pub mod precipitation {
    use super::*;

    pub struct SensorPrecipitation {
        pub coordinates: Coordinates,
        pub interval: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct SensorPrecipitationOutput {
        pub coordinates: Coordinates,
        pub timestamp: u64,
        pub interval: u32,
        pub precipitation: f64,
    }

    impl SensorOutput for SensorPrecipitationOutput {
        fn to_json(&self) -> String {
            serde_json::to_string(self).unwrap()
        }
    }

    impl Sensor for SensorPrecipitation {
        fn generate(&self, begin: u64, end: u64) -> Vec<String> {
            let mut results = Vec::new();
            let mut timestamp = begin;

            while timestamp <= end {
                let output = SensorPrecipitationOutput {
                    coordinates: self.coordinates.clone(),
                    timestamp,
                    interval: self.interval,
                    precipitation: 0.0,
                };
                results.push(output.to_json());
                timestamp += self.interval as u64;
            }
            results
        }
    }
}

/// Module containing wind velocity and direction related functionality.
pub mod wind_vel_and_dir {
    use super::*;

    pub struct SensorWindVelocityAndDirection {
        pub coordinates: Coordinates,
        pub interval: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct SensorWindVelocityAndDirectionOutput {
        pub coordinates: Coordinates,
        pub timestamp: u64,
        pub interval: u32,
        pub velocity: f64,
        pub direction: f64,
    }

    impl SensorOutput for SensorWindVelocityAndDirectionOutput {
        fn to_json(&self) -> String {
            serde_json::to_string(self).unwrap()
        }
    }

    impl Sensor for SensorWindVelocityAndDirection {
        fn generate(&self, begin: u64, end: u64) -> Vec<String> {
            let mut results = Vec::new();
            let mut timestamp = begin;

            while timestamp <= end {
                let output = SensorWindVelocityAndDirectionOutput {
                    coordinates: self.coordinates.clone(),
                    timestamp,
                    interval: self.interval,
                    velocity: 0.0, // velocity is constant for all measurements, should change in the future
                    direction: 0.0, // direction is constant for all measurements, should change in the future
                };
                results.push(output.to_json());
                timestamp += self.interval as u64;
            }
            results
        }
    }
}

/// Module containing functionality related to cloud coverage
pub mod cloud_coverage {
    use super::*;

    pub struct SensorCloudCoverage {
        pub coordinates: Coordinates,
        pub interval: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct SensorCloudCoverageOutput {
        pub coordinates: Coordinates,
        pub timestamp: u64,
        pub interval: u32,
        pub cloud_coverage: f64,
    }

    impl SensorOutput for SensorCloudCoverageOutput {
        fn to_json(&self) -> String {
            serde_json::to_string(self).unwrap()
        }
    }

    impl Sensor for SensorCloudCoverage {
        fn generate(&self, begin: u64, end: u64) -> Vec<String> {
            let mut results = Vec::new();
            let mut timestamp = begin;

            while timestamp <= end {
                let output = SensorCloudCoverageOutput {
                    coordinates: self.coordinates.clone(),
                    timestamp,
                    interval: self.interval,
                    cloud_coverage: 0.0, // cloud coverage is constant for all measurements, should change in the future
                };
                results.push(output.to_json());
                timestamp += self.interval as u64;
            }
            results
        }
    }
}

/// Module containing functionality related to energy
pub mod energy {
    use super::*;

    /// Represents a concrete power sensor called [SensorPower].
    // TODO: Remove this
    #[allow(unused)]
    pub struct SensorPower {
        /// Geographical coordinates of the sensor.
        pub coordinates: Coordinates,
        /// The time interval between sensor readings (in seconds).
        pub interval: u32,
    }

    /// Represents the output of the concrete power sensor called SensorPower.
    #[derive(Serialize, Deserialize, Debug)]
    pub struct SensorPowerOutput {
        /// The geographic coordinates of the sensor.
        pub coordinates: Coordinates,
        /// The timestamp of the sensor reading.
        pub timestamp: u64,
        /// The time interval between sensor readings (in seconds).
        pub interval: u32,
        /// The global active power recorded by the sensor.
        pub global_active_power: f64,
        /// The global reactive power recorded by the sensor.
        pub global_reactive_power: f64,
        /// The voltage recorded by the sensor.
        pub voltage: f64,
        /// The global intensity recorded by the sensor.
        pub global_intensity: f64,
        /// The sub metering 1 recorded by the sensor.
        pub sub_metering_1: f64,
        /// The sub metering 2 recorded by the sensor.
        pub sub_metering_2: f64,
        /// The sub metering 3 recorded by the sensor.
        pub sub_metering_3: f64,
    }

    impl SensorOutput for SensorPowerOutput {
        fn to_json(&self) -> String {
            serde_json::to_string(self).unwrap()
        }
    }

    /// Specific implementation for the concrete temperature sensor called [SensorTemperature].
    impl SensorPower {
        // TODO: Remove this
        #[allow(unused)]
        pub fn generate(&self, begin: u64, end: u64) -> Vec<String> {
            let mut results = Vec::new();
            let mut timestamp = begin;

            while timestamp <= end {
                let output = SensorPowerOutput {
                    coordinates: self.coordinates.clone(),
                    timestamp,
                    interval: self.interval,
                    global_active_power: 0.0,   // Placeholder value
                    global_reactive_power: 0.0, // Placeholder value
                    voltage: 0.0,               // Placeholder value
                    global_intensity: 0.0,      // Placeholder value
                    sub_metering_1: 0.0,        // Placeholder value
                    sub_metering_2: 0.0,        // Placeholder value
                    sub_metering_3: 0.0,        // Placeholder value
                };
                results.push(output.to_json());
                timestamp += self.interval as u64;
            }
            results
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_json() {
        let sensor_data_temperature = temperature::SensorTemperatureOutput {
            coordinates: Coordinates {
                latitude: 41.0,
                longitude: 15.1,
            },
            timestamp: 1568416,
            interval: 60,
            temperature: 14.3,
        };

        let sensor_data_precipitation = precipitation::SensorPrecipitationOutput {
            coordinates: Coordinates {
                latitude: 12.2,
                longitude: 31.8,
            },
            timestamp: 111111111,
            interval: 360,
            precipitation: 2.5,
        };

        let sensor_data_power = energy::SensorPowerOutput {
            coordinates: Coordinates {
                latitude: 51.260197,
                longitude: 4.402771,
            },
            timestamp: 1708511635,
            interval: 60,
            global_active_power: 4.216,
            global_reactive_power: 0.418,
            voltage: 234.840,
            global_intensity: 18.400,
            sub_metering_1: 0.000,
            sub_metering_2: 1.000,
            sub_metering_3: 17.000,
        };

        let sensor_data: Vec<Box<dyn SensorOutput>> = vec![
            Box::new(sensor_data_temperature),
            Box::new(sensor_data_precipitation),
            Box::new(sensor_data_power),
        ];

        assert_eq!(sensor_data[0].to_json(), "{\"coordinates\":{\"latitude\":41.0,\"longitude\":15.1},\"timestamp\":1568416,\"interval\":60,\"temperature\":14.3}");
        assert_eq!(sensor_data[1].to_json(), "{\"coordinates\":{\"latitude\":12.2,\"longitude\":31.8},\"timestamp\":111111111,\"interval\":360,\"precipitation\":2.5}");
        assert_eq!(
            sensor_data[2].to_json(),
            r#"{"coordinates":{"latitude":51.260197,"longitude":4.402771},"timestamp":1708511635,"interval":60,"global_active_power":4.216,"global_reactive_power":0.418,"voltage":234.84,"global_intensity":18.4,"sub_metering_1":0.0,"sub_metering_2":1.0,"sub_metering_3":17.0}"#
        )
    }

    #[test]
    fn test_generate_temperature() {
        let coordinates = Coordinates {
            latitude: 23.5,
            longitude: 60.2,
        };
        let sensor_temperature = temperature::SensorTemperature {
            coordinates,
            interval: 60,
        };

        let expected = [
            r#"{"coordinates":{"latitude":23.5,"longitude":60.2},"timestamp":5,"interval":60,"temperature":20.0}"#,
            r#"{"coordinates":{"latitude":23.5,"longitude":60.2},"timestamp":65,"interval":60,"temperature":20.0}"#,
            r#"{"coordinates":{"latitude":23.5,"longitude":60.2},"timestamp":125,"interval":60,"temperature":20.0}"#,
        ];

        let actual = sensor_temperature.generate(5, 126);

        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            assert_eq!(actual, expected);
        }
    }
}
