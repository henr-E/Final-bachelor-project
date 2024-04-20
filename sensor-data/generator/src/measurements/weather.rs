//! Module for generating fake measurements related to weather.

use chrono::{Datelike, Timelike};
use sqlx::types::BigDecimal;

use sensor_store::quantity::Quantity;
use sensor_store::unit::Unit;

use crate::measurements::read_historic_data::HistoricDataReader;
use crate::measurements::utilities::timestamp_to_date_time;
use crate::measurements::Measurement;

/// A structure for generating fake measurements related to weather.
pub(super) struct WeatherDataGenerator {}

impl WeatherDataGenerator {
    /// Constructor for [WeatherDataGenerator].
    pub(super) fn new() -> Self {
        Self {}
    }

    /// Returns a fake weather measurement for the specified timestamp and quantity.
    /// # Arguments
    ///
    /// * `timestamp` - Unix timestamp in seconds. Used as a reference point in order to retrieve a value similar to the historic value.
    /// * `variance` - Used to control te randomness of the measurement.
    /// * `quantity` - The [Quantity] for which a measurement should be generated.
    /// * `historic_data_reader` - Used to read historic data values from assets
    pub(super) fn get_measurement(
        &mut self,
        timestamp: u64,
        variance: u64,
        quantity: Quantity,
        historic_data_reader: &mut HistoricDataReader,
    ) -> Option<Measurement> {
        // convert timestamp to DateTime
        let date_timestamp = timestamp_to_date_time(timestamp).ok()?;
        // get a value based on the historic data.
        let value = historic_data_reader.sample(
            date_timestamp.month(),
            date_timestamp.day(),
            date_timestamp.hour(),
            variance,
            quantity,
        )?;
        // get the unit and prefix of the data present in self.data
        let (unit, prefix) = match quantity {
            Quantity::Rainfall => (Unit::MillimetersPerHour, BigDecimal::from(1)),
            Quantity::Temperature => (Unit::Celsius, BigDecimal::from(1)),
            Quantity::WindSpeed => (Unit::MetersPerSecond, BigDecimal::from(1)),
            Quantity::WindDirection => (Unit::Degrees, BigDecimal::from(1)),
            Quantity::Irradiance => (Unit::WattsPerSquareMetre, BigDecimal::from(1)),
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

    use crate::measurements::read_historic_data::HistoricDataReader;
    use crate::measurements::weather::WeatherDataGenerator;
    use crate::measurements::Measurement;

    /// Tests that the measurements are actually based on historic data.
    #[test]
    fn test_get_measurement() {
        // specify a time range for which electricity consumption data should be generated
        let date_time: NaiveDateTime = NaiveDate::from_ymd_opt(2024, 3, 8)
            .unwrap()
            .and_hms_opt(12, 44, 44)
            .unwrap();

        let timestamp = date_time.and_utc().timestamp();

        let mut historic_data_reader = HistoricDataReader::new();

        let mut weather_data_generator = WeatherDataGenerator::new();

        // value comes from the parquet file assets/sensor-data/historic-data/2013-03-08.parquet at 12:00:00 first house
        // Temperature;WindDirection; WindSpeed; Precipitation; Irradiance
        // 9.8,60.0,4.0,0.6,251.0

        // retrieve measurements for specified time range (make sure variance is equal to 0 to get historic value.)
        let act_temperature = weather_data_generator.get_measurement(
            timestamp as u64,
            0,
            Quantity::Temperature,
            &mut historic_data_reader,
        );

        let exp_temperature = Measurement {
            unit: Unit::Celsius,
            quantity: Quantity::Temperature,
            value: 9.8,
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
        let act_rainfall = weather_data_generator.get_measurement(
            timestamp as u64,
            0,
            Quantity::Rainfall,
            &mut historic_data_reader,
        );

        let exp_rainfall = Measurement {
            unit: Unit::MillimetersPerHour,
            quantity: Quantity::Rainfall,
            value: 0.6,
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
        let act_wind_speed = weather_data_generator.get_measurement(
            timestamp as u64,
            0,
            Quantity::WindSpeed,
            &mut historic_data_reader,
        );

        let exp_wind_speed = Measurement {
            unit: Unit::MetersPerSecond,
            quantity: Quantity::WindSpeed,
            value: 4.0,
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
        let act_wind_direction = weather_data_generator.get_measurement(
            timestamp as u64,
            0,
            Quantity::WindDirection,
            &mut historic_data_reader,
        );

        let exp_wind_direction = Measurement {
            unit: Unit::Degrees,
            quantity: Quantity::WindDirection,
            value: 60.0,
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
        let act_irradiance = weather_data_generator.get_measurement(
            timestamp as u64,
            0,
            Quantity::Irradiance,
            &mut historic_data_reader,
        );

        let exp_irradiance = Measurement {
            unit: Unit::WattsPerSquareMetre,
            quantity: Quantity::Irradiance,
            value: 251.0,
            prefix: BigDecimal::from(1),
        };

        assert!(
            act_irradiance.is_some(),
            "Failed to retrieve the measurement."
        );
        assert_eq!(
            act_irradiance.unwrap(),
            exp_irradiance,
            "Measurement does not match the expected value."
        );
    }
}
