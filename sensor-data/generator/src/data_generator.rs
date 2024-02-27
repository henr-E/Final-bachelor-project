//! This file contains functionality related to the generation of sensor data.

use crate::sensors::Sensor;

/// Instances of this struct can be used to generate sensor data.
pub struct SensorDataGenerator {
    /// A vector containing zero or more sensors (possibly of different types).
    sensors: Vec<Box<dyn Sensor>>,
}

impl SensorDataGenerator {
    /// Creates a new [SensorDataGenerator] instance without sensors attached.
    pub fn new() -> Self {
        SensorDataGenerator {
            sensors: Vec::new(),
        }
    }
    /// Attach a sensor.
    /// # Arguments
    ///
    /// * `sensor` - The sensor to be attached.
    ///
    /// # Example
    ///
    /// ```
    /// // create a SensorDataGenerator object
    /// let mut sensor_data_generator = SensorDataGenerator::new();
    ///
    /// // create a sensor
    /// let sensor1 = Box::new(SensorTemperature {coordinates: Coordinates {latitude: 23.5, longitude: 60.2, }, interval: 30,});
    ///
    /// // add the sensor
    /// sensor_data_generator.add_sensor(sensor1);
    /// ```
    ///
    ///
    pub fn add_sensor(&mut self, sensor: Box<dyn Sensor>) {
        self.sensors.push(sensor);
    }

    /// Generate a vector of bytes representing sensor data recorded between the specified time frame, from 'begin' to 'end' by the attached sensors.
    ///
    /// The output will have the following format (in bytes):
    /// {"entries" = \[...\]}
    /// The "entries" field will contain a vector of outputs generated by all the sensors.
    ///
    /// # Arguments
    ///
    /// * `begin` - Unix timestamp in seconds, the start of the timeframe
    /// * `end` - Unix timestamp in seconds, the end of the timeframe
    ///
    /// # Example
    ///
    /// ```
    /// // create a SensorDataGenerator object
    /// let mut sensor_data_generator = SensorDataGenerator::new();
    ///
    /// // create a sensor
    /// let sensor1 = Box::new(SensorTemperature {coordinates: Coordinates {latitude: 23.5, longitude: 60.2, }, interval: 30,});
    ///
    /// // add the sensor
    /// sensor_data_generator.add_sensor(sensor1);
    ///
    /// // generate data for a time range
    /// let begin: u64 = 0;
    /// let end: u64 = 75;
    /// let generated_data = sensor_data_generator.generate_data(begin, end);
    /// ```
    pub fn generate_data(&self, begin: u64, end: u64) -> Vec<u8> {
        let mut output_sensors = Vec::new();
        for sensor in &self.sensors {
            // for each sensor, retrieve the output generated between begin and end.
            let output_sensor = sensor.generate(begin, end);
            output_sensors.extend(output_sensor);
        }
        // serialize output_sensors as a JSON byte vector ({"entries" = \[...\]})
        serde_json::to_vec(&output_sensors).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::sensors::{
        precipitation::SensorPrecipitation,
        temperature::{SensorTemperature, SensorTemperatureOutput},
        Coordinates, SensorOutput,
    };

    use super::*;

    // A sensor for testing
    struct MockSensor {
        id: String,
    }

    // The output format of the test sensor.
    #[derive(Serialize, Deserialize, Debug)]
    struct MockSensorOutput {
        id: String,
    }

    impl Sensor for MockSensor {
        // The mock sensor outputs its id every second
        fn generate(&self, begin: u64, end: u64) -> Vec<String> {
            let mut results = Vec::new();
            for _timestamp in begin..=end {
                results.push(
                    MockSensorOutput {
                        id: self.id.clone(),
                    }
                    .to_json(),
                );
            }
            results
        }
    }

    impl SensorOutput for MockSensorOutput {
        // convert the mock sensor output to a JSON string
        fn to_json(&self) -> String {
            serde_json::to_string(self).unwrap()
        }
    }

    #[test]
    fn test_sensor_data_generator1() {
        // create a SensorDataGenerator object
        let mut sensor_data_generator = SensorDataGenerator::new();

        // create two sensors that output their id each second
        let sensor_a = Box::new(MockSensor {
            id: String::from("a"),
        });
        let sensor_b = Box::new(MockSensor {
            id: String::from("b"),
        });

        // add sensors to data generator
        sensor_data_generator.add_sensor(sensor_a);
        sensor_data_generator.add_sensor(sensor_b);

        // generate data for a time range
        let generated_data = sensor_data_generator.generate_data(0, 1);

        let v: Vec<String> = serde_json::from_slice(&generated_data).unwrap();

        // both sensor_a and sensor_b sent data at time 0 and 1
        assert_eq!(v.len(), 4);

        // the first 2 outputs in the vector were generated by sensor a, the last 2 by sensor b
        let output1: MockSensorOutput = serde_json::from_str(&v[0].clone()).unwrap();
        let output2: MockSensorOutput = serde_json::from_str(&v[1].clone()).unwrap();
        let output3: MockSensorOutput = serde_json::from_str(&v[2].clone()).unwrap();
        let output4: MockSensorOutput = serde_json::from_str(&v[3].clone()).unwrap();

        assert_eq!(output1.id, "a");
        assert_eq!(output2.id, "a");
        assert_eq!(output3.id, "b");
        assert_eq!(output4.id, "b");
    }

    #[test]
    fn test_sensor_data_generator2() {
        // create a SensorDataGenerator object
        let mut sensor_data_generator = SensorDataGenerator::new();

        // Add a temperature sensor
        let sensor1 = Box::new(SensorTemperature {
            coordinates: Coordinates {
                latitude: 23.5,
                longitude: 60.2,
            },
            interval: 30, // set the interval to 30 seconds
        });

        // Add a precipitation sensor
        let sensor2 = Box::new(SensorPrecipitation {
            coordinates: Coordinates {
                latitude: 18.5,
                longitude: 0.2,
            },
            interval: 60, // set the interval to 60 seconds
        });

        // add sensors to data generator
        sensor_data_generator.add_sensor(sensor1);
        sensor_data_generator.add_sensor(sensor2);

        // generate data for a time range
        let begin: u64 = 0;
        let end: u64 = 75;
        let generated_data = sensor_data_generator.generate_data(begin, end);

        let v: Vec<String> = serde_json::from_slice(&generated_data).unwrap();

        // sensor1 sent data at times 0, 30, 60; sensor2 sent data at times 0, 60 -> total = 5
        assert_eq!(v.len(), 5);

        // make sure the third output was generated by the temperature sensor by deserializing
        let x: SensorTemperatureOutput = serde_json::from_str(&v[2].clone()).unwrap();
        // check temperature value
        assert_eq!(x.temperature, 20.0)
    }
}
