//! This module contains a struct representing a virtual sensor that can send output.

use std::collections::HashMap;

use serde_json::{json, Value};

use proto::sensor_data_ingest::{sensor_data_file::FileFormat, SensorDataFile};
use sensor_store::sensor::Sensor;

use crate::measurements::convert::convert;
use crate::measurements::{Measurement, MeasurementsGenerator};

/// A 'virtual' sensor. This sensor can generate (fake) output.
pub struct VirtualSensor<'a> {
    /// The sensor that is emulated.
    sensor: Sensor<'a>,
    /// The time interval in seconds between each output.
    interval: u64,
    /// The file format of the data that is outputted.
    file_format: FileFormat,
}

impl<'a> VirtualSensor<'a> {
    /// Returns a virtual sensor.
    /// # Arguments
    ///
    /// * `sensor` - The sensor that is emulated.
    /// * `interval` - The time interval in seconds between each output.
    /// * `file_format` - The file format of the data that is outputted.
    pub fn new(sensor: Sensor<'a>, interval: u64, file_format: FileFormat) -> Self {
        Self {
            sensor,
            interval,
            file_format,
        }
    }
    /// Returns a SensorDataFile object representing the output of the virtual sensor generated in the timeframe starting at `begin` and ending at `end`.
    /// The data field of the SensorDataFile will contain a single JSON object with a single toplevel field 'measurements'.
    /// A vector of measurements taken at different timestamps will be contained by this field.
    ///
    /// # Arguments
    ///
    /// * `begin` - Unix timestamp in seconds, the start of the timeframe
    /// * `end` - Unix timestamp in seconds, the end of the timeframe
    /// * `measurements_generator` - Used to generate measurements for the different fields in the sensor output.
    pub fn generate_sensor_data_file(
        &self,
        begin: u64,
        end: u64,
        measurements_generator: &mut MeasurementsGenerator,
    ) -> SensorDataFile {
        let mut measurements: Vec<Value> = Vec::new();

        let mut map_fields_data: HashMap<&str, Value> = HashMap::new();
        // step forward in time with steps of self.interval until end is reached
        let mut timestamp = begin;
        while timestamp < end {
            for signal in self.sensor.signals() {
                // default measurement: to be used if something goes wrong or no historic data is present
                let default_measurement = Measurement {
                    unit: signal.unit,
                    quantity: signal.quantity,
                    value: 0.0,
                    prefix: signal.prefix.clone(),
                };
                // try to create a (randomized) measurement based on historic data
                let measurement = measurements_generator.get_measurement(
                    timestamp + self.interval,
                    100,
                    signal.quantity,
                );

                // apply correct unit conversion from historic units and prefix to required units and prefix
                let converted_measurement = match measurement {
                    None => default_measurement,
                    Some(measurement) => convert(&measurement, signal.unit, signal.prefix.clone())
                        .unwrap_or(default_measurement),
                };
                // insert the measurement as json object in a map with a key equal to the signal name
                map_fields_data.insert(&signal.name, json!(converted_measurement.value));
            }
            // convert the map to a json value
            let json_object: Value = json!(map_fields_data);
            measurements.push(json_object);
            timestamp += self.interval;
        }
        let data_json = json!({
        "measurements": measurements});
        let data = serde_json::to_vec(&data_json).unwrap();
        SensorDataFile {
            data,
            sensor_id: self.sensor.id.to_string(),
            file_format: Some(self.file_format.clone()),
        }
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

    use crate::measurements::MeasurementsGenerator;
    use crate::virtual_sensor::VirtualSensor;

    /// Tests if the output generated by the virtual sensor contains the required keys and that the interval is used correctly.
    #[test]
    fn test_generate_sensor_data_file() {
        let mut sensor_builder = SensorBuilder {
            id: Uuid::new_v4(),
            name: Cow::Borrowed("mySensor"),
            description: Some(Cow::Borrowed("A sensor that measures some variables.")),
            location: (0.0, 0.0),
            signals: Default::default(),
            twin_id: -1,
            building_id: None,
        };
        sensor_builder.add_signal(
            0,
            Cow::Borrowed("Temperature(C)"),
            Quantity::Temperature,
            Unit::Celsius,
            BigDecimal::from(1),
        );

        sensor_builder.add_signal(
            0,
            Cow::Borrowed("Weight(g)"),
            Quantity::Mass,
            Unit::Kilogram,
            BigDecimal::from(1 / 1000),
        );

        sensor_builder.add_signal(
            0,
            Cow::Borrowed("electricity(W)"),
            Quantity::Power,
            Unit::Watt,
            BigDecimal::from(1),
        );

        let sensor = sensor_builder.build();

        let virtual_sensor = VirtualSensor::new(sensor, 3600, Json(JsonFileFormat {}));

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

        let mut measurements_generator = MeasurementsGenerator::new();

        let sensor_data_file = virtual_sensor.generate_sensor_data_file(
            timestamp_begin as u64,
            timestamp_end as u64,
            &mut measurements_generator,
        );

        let data: Value = serde_json::from_slice(&sensor_data_file.data).unwrap();
        let measurements = data.get("measurements").unwrap().as_array().unwrap();

        // Check that there are exactly two measurements taken by the virtual sensor
        assert_eq!(measurements.len(), 2);

        for measurement in measurements {
            // Check if JSON object contains keys for temperature, weight, and electricity
            assert!(measurement.get("Temperature(C)").is_some());
            assert!(measurement.get("Weight(g)").is_some());
            assert!(measurement.get("electricity(W)").is_some());
        }
    }
}
