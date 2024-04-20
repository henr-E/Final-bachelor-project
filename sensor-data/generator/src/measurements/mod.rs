//! Module that provides functionality to generate fake data for different kinds of quantities.
use sqlx::types::BigDecimal;

use sensor_store::quantity::Quantity;
use sensor_store::unit::Unit;

use crate::measurements::electricity_consumption::ElectricityConsumptionGenerator;
use crate::measurements::read_historic_data::HistoricDataReader;
use crate::measurements::time::TimestampGenerator;
use crate::measurements::weather::WeatherDataGenerator;

pub mod convert;
mod electricity_consumption;
mod read_historic_data;
mod time;
mod utilities;
mod weather;

/// A struct describing a measurement
#[derive(Debug, PartialEq)]
pub struct Measurement {
    /// The unit of the measurement.
    pub unit: Unit,
    /// The quantity of the measurement.
    pub quantity: Quantity,
    /// The value of the measurement.
    pub value: f64,
    /// Prefix of the value compared to the unit.
    pub prefix: BigDecimal,
}

/// Instances of [MeasurementsGenerator] can be used to generate fake measurements for different quantities.
pub struct MeasurementsGenerator {
    /// Reads historic data values from assets
    historic_data_reader: HistoricDataReader,
    /// Generates measurements related to energy consumption.
    electricity_consumption_generator: ElectricityConsumptionGenerator,
    /// Generates measurements related to weather.
    weather_data_generator: WeatherDataGenerator,
    /// Generates measurements related to time.
    timestamp_generator: TimestampGenerator,
}

impl MeasurementsGenerator {
    /// Returns a [MeasurementsGenerator] object.
    pub fn new() -> Self {
        Self {
            historic_data_reader: HistoricDataReader::new(),
            electricity_consumption_generator: ElectricityConsumptionGenerator::new(),
            weather_data_generator: WeatherDataGenerator::new(),
            timestamp_generator: TimestampGenerator {},
        }
    }

    /// Returns a fake measurement for the specified quantity and the specified timestamp.
    /// The measurement is based of historic data. The variance argument can be used to introduce randomness in the returned measurement.
    /// A variance of 0 will not randomize the returned measurement. If no historic data can be found for the provided quantity, None will be returned.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Unix timestamp in seconds. This timestamp is used to index the historic data.
    /// * `variance` - Used to control te randomness of the measurements.
    /// * `quantity` - The [Quantity] for which the fake measurement should be generated.
    pub fn get_measurement(
        &mut self,
        timestamp: u64,
        variance: u64,
        quantity: Quantity,
    ) -> Option<Measurement> {
        let measurement: Option<Measurement> = match quantity {
            // energy consumption
            Quantity::Power => self.electricity_consumption_generator.get_measurement(
                timestamp,
                variance,
                &mut self.historic_data_reader,
            ),
            // weather related quantities
            Quantity::Temperature
            | Quantity::Rainfall
            | Quantity::WindSpeed
            | Quantity::WindDirection
            | Quantity::Irradiance => self.weather_data_generator.get_measurement(
                timestamp,
                variance,
                quantity,
                &mut self.historic_data_reader,
            ),
            Quantity::Timestamp => self.timestamp_generator.get_measurement(timestamp),
            // for currently unsupported quantities, return None
            _ => None,
        };
        measurement
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveDateTime};

    use sensor_store::quantity::Quantity;

    use crate::measurements::MeasurementsGenerator;

    /// Ensures that for quantities that are not supported (no data source or not yet implemented) None is returned
    #[test]
    fn test_get_measurement_for_unsupported_quantity() {
        // specify a time range for which  data should be generated
        let date_time: NaiveDateTime = NaiveDate::from_ymd_opt(2024, 3, 2)
            .unwrap()
            .and_hms_opt(20, 44, 44)
            .unwrap();
        let timestamp = date_time.and_utc().timestamp();

        // construct a MeasurementsGenerator object
        let mut measurements_generator = MeasurementsGenerator::new();

        // generate measurements for electricity consumption for the specified interval
        let data = measurements_generator.get_measurement(timestamp as u64, 0, Quantity::Mass);

        assert_eq!(
            data, None,
            "None was not returned for an unsupported quantity."
        );
    }
}
