//! Module that contains functionality to generate sensor data for multiple virtual sensors at once.
//! Also contains functionality to construct virtual sensors based on the sensors registered in the database.

use futures::stream::StreamExt;
use tracing::error;

use proto::sensor_data_ingest::sensor_data_file::FileFormat;
use proto::sensor_data_ingest::{JsonFileFormat, SensorDataFile};
use sensor_store::SensorStore;

use crate::measurements::MeasurementsGenerator;
use crate::virtual_sensor::VirtualSensor;

/// Instances of this struct can be used to (fake) generate sensor data for multiple sensors at once.
pub struct SensorDataGenerator<'a> {
    /// A vector containing zero or more sensors.
    virtual_sensors: Vec<VirtualSensor<'a>>,
}

impl<'a> SensorDataGenerator<'a> {
    /// Creates a new [SensorDataGenerator] instance without virtual sensors attached.
    pub fn new() -> Self {
        Self {
            virtual_sensors: Vec::new(),
        }
    }

    /// Construct virtual sensors based on the sensors registered in the database and stores them in self.virtual_sensors.
    ///
    /// # Arguments
    ///
    /// * `sensor_store` - A mutable reference to the database wrapper.
    pub async fn retrieve_sensors_from_db(&mut self, sensor_store: &'a SensorStore) {
        let mut virtual_sensors = Vec::new();

        match sensor_store.get_all_sensors().await {
            Ok(sensor_stream) => {
                // Iterate over the stream of sensors
                sensor_stream
                    .for_each(|sensor_result| {
                        match sensor_result {
                            Ok(sensor) => {
                                // Create a SensorWrapper object with the retrieved sensor and interval
                                let virtual_sensor = VirtualSensor::new(
                                    sensor,
                                    3600,
                                    FileFormat::Json(JsonFileFormat {}),
                                );
                                virtual_sensors.push(virtual_sensor);
                            }
                            Err(err) => error!("Error retrieving sensor: {}", err),
                        }
                        futures::future::ready(())
                    })
                    .await;
            }
            Err(err) => error!("Error retrieving all sensors: {}", err),
        }
        // Update the sensor_data field with the retrieved sensor data
        self.virtual_sensors = virtual_sensors;
    }

    /// Returns a vector containing sensor data generated by the attached virtual sensors in the timeframe starting at `begin` and ending at `end`.
    ///
    /// # Arguments
    ///
    /// * `begin` - Unix timestamp in seconds, the start of the timeframe
    /// * `end` - Unix timestamp in seconds, the end of the timeframe
    pub fn generate(&mut self, begin: u64, end: u64) -> Vec<SensorDataFile> {
        // Instantiate a MeasurementsGenerator object to generate fake data for certain fields in the output
        let mut measurements_generator = MeasurementsGenerator::new();

        // Use iterator map to generate SensorDataFile objects for each virtual sensor
        self.virtual_sensors
            .iter()
            .map(|sensor| sensor.generate_sensor_data_file(begin, end, &mut measurements_generator))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use chrono::{NaiveDate, NaiveDateTime};
    use serde_json::Value;
    use sqlx::types::BigDecimal;
    use uuid::Uuid;

    use proto::sensor_data_ingest::sensor_data_file::FileFormat::Json;
    use proto::sensor_data_ingest::JsonFileFormat;
    use sensor_store::quantity::Quantity;
    use sensor_store::sensor::SensorBuilder;
    use sensor_store::unit::Unit;

    use crate::sensor_data_generator::SensorDataGenerator;
    use crate::virtual_sensor::VirtualSensor;

    /// Tests the generation of sensor data for multiple sensors and quantities.
    #[test]
    fn test_sensor_data_generator() {
        let mut sensor_builder = SensorBuilder {
            id: Uuid::new_v4(),
            name: Cow::Borrowed("myTempSensor"),
            description: Some(Cow::Borrowed("A sensor that measures temperature.")),
            location: (0.0, 0.0),
            signals: Default::default(),
            twin_id: -1,
        };
        sensor_builder.add_signal(
            0,
            Cow::Borrowed("Temperature(C)"),
            Quantity::Temperature,
            Unit::Celsius,
            BigDecimal::from(1),
        );

        let sensor1 = sensor_builder.build();

        let virtual_sensor1 = VirtualSensor::new(sensor1, 7200, Json(JsonFileFormat {}));

        let mut sensor_builder2 = SensorBuilder {
            id: Uuid::new_v4(),
            name: Cow::Borrowed("myTimeStampSensor"),
            description: Some(Cow::Borrowed("A sensor that measures timestamp.")),
            location: (1.0, 1.0),
            signals: Default::default(),
            twin_id: -1,
        };

        sensor_builder2.add_signal(
            0,
            Cow::Borrowed("time"),
            Quantity::Timestamp,
            Unit::Utc,
            BigDecimal::from(1),
        );

        let sensor2 = sensor_builder2.build();
        let virtual_sensor2 = VirtualSensor::new(sensor2, 3600, Json(JsonFileFormat {}));

        let mut sensor_data_generator = SensorDataGenerator::new();
        sensor_data_generator.virtual_sensors.push(virtual_sensor1);
        sensor_data_generator.virtual_sensors.push(virtual_sensor2);

        let date_time_begin: NaiveDateTime = NaiveDate::from_ymd_opt(2024, 3, 2)
            .unwrap()
            .and_hms_opt(23, 44, 44)
            .unwrap();
        let timestamp_begin = date_time_begin.and_utc().timestamp();

        let date_time_end: NaiveDateTime = NaiveDate::from_ymd_opt(2024, 3, 3)
            .unwrap()
            .and_hms_opt(1, 0, 0)
            .unwrap();
        let timestamp_end = date_time_end.and_utc().timestamp();

        let sensor_data_files =
            sensor_data_generator.generate(timestamp_begin as u64, timestamp_end as u64);

        // Check that there are 2 SensorDataFiles present
        assert_eq!(sensor_data_files.len(), 2);

        let mut temperature_count = 0;
        let mut time_count = 0;

        for data_file in &sensor_data_files {
            let data: Value = serde_json::from_slice(&data_file.data).unwrap();
            let measurements = data.get("measurements").unwrap().as_array().unwrap();

            for measurement in measurements {
                // Check if JSON object contains keys for temperature and time
                if measurement.get("Temperature(C)").is_some() {
                    // Increment temperature count
                    temperature_count += 1;
                } else if let Some(time) = measurement.get("time").and_then(Value::as_f64) {
                    // Increment time count
                    time_count += 1;

                    // Assert that time value is within the expected range
                    assert!(time >= timestamp_begin as f64 || time <= timestamp_end as f64);
                }
            }
        }

        // Assert that there is exactly one measurement with key Temperature(C)
        assert_eq!(temperature_count, 1);

        // Assert that there are exactly two measurements with key time
        assert_eq!(time_count, 2);
    }
}
