//! Module that provides functionality to convert a [Measurement] to a different [Unit] and/or prefix.
use rand_distr::num_traits::ToPrimitive;
use sqlx::types::BigDecimal;
use thiserror::Error;
use unit_conversions::temperature::{celsius, fahrenheit, kelvin};

use sensor_store::quantity::Quantity;
use sensor_store::unit::Unit;

use crate::measurements::Measurement;

/// Errors that can occur when a [Measurement] is converted to a different prefix or unit.
#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Invalid unit '{0:?}' for quantity '{1:?}'.")]
    InvalidUnit(Unit, Quantity),
    #[error("Unable to represent prefix {0} as a float.")]
    PrefixConversionFailed(BigDecimal),
    #[error("The measurement contains a prefix {0} that can not be represented by a float.")]
    MeasurementPrefixConversionFailed(BigDecimal),
}

/// Takes a [Measurement] and returns a corresponding [Measurement] with the specified [Unit] and prefix.
/// # Arguments
///
/// * `measurement` - The measurement from which the new measurement should be constructed.
/// * `unit` - The unit that the returned measurement should contain.
/// * `prefix` - The prefix that the returned measurement should contain.
pub fn convert(
    measurement: &Measurement,
    unit: Unit,
    prefix: BigDecimal,
) -> Result<Measurement, ConversionError> {
    // if the unit does correspond to the quantity of the measurement, no conversion can take place
    if !measurement.quantity.associated_units().contains(&unit) {
        return Err(ConversionError::InvalidUnit(unit, measurement.quantity));
    }

    // multiply the value with the prefix (contained in the provided measurement)
    let value_times_prefix_measurement = measurement.value
        * measurement.prefix.to_f64().ok_or_else(|| {
            ConversionError::MeasurementPrefixConversionFailed(measurement.prefix.clone())
        })?;

    // do the unit conversion
    let converted_value: f64 = match (measurement.quantity, measurement.unit, unit) {
        // Temperature conversions
        (Quantity::Temperature, Unit::Celsius, Unit::Fahrenheit) => {
            celsius::to_fahrenheit(value_times_prefix_measurement)
        }
        (Quantity::Temperature, Unit::Celsius, Unit::Kelvin) => {
            celsius::to_kelvin(value_times_prefix_measurement)
        }
        (Quantity::Temperature, Unit::Fahrenheit, Unit::Celsius) => {
            fahrenheit::to_celsius(value_times_prefix_measurement)
        }
        (Quantity::Temperature, Unit::Fahrenheit, Unit::Kelvin) => {
            fahrenheit::to_kelvin(value_times_prefix_measurement)
        }
        (Quantity::Temperature, Unit::Kelvin, Unit::Celsius) => {
            kelvin::to_celsius(value_times_prefix_measurement)
        }
        (Quantity::Temperature, Unit::Kelvin, Unit::Fahrenheit) => {
            kelvin::to_fahrenheit(value_times_prefix_measurement)
        }
        _ => value_times_prefix_measurement,
    };
    // divide the value by the target prefix
    let converted_value_divided_by_prefix: f64 = converted_value
        / prefix
            .to_f64()
            .ok_or_else(|| ConversionError::PrefixConversionFailed(prefix.clone()))?;
    // construct a new measurement with the new value, unit and prefix
    let converted_measurement = Measurement {
        unit,
        quantity: measurement.quantity,
        value: converted_value_divided_by_prefix,
        prefix,
    };
    Ok(converted_measurement)
}

#[cfg(test)]
mod tests {
    use sqlx::types::BigDecimal;

    use sensor_store::quantity::Quantity;
    use sensor_store::unit::Unit;

    use crate::measurements::convert::convert;
    use crate::measurements::Measurement;

    /// Ensures that the implemented unit conversions (for temperature) yield correct measurements.
    #[test]
    fn test_convert_temperature() {
        // Test Celsius to Fahrenheit conversion
        let temp_celsius = Measurement {
            value: 14.0,
            unit: Unit::Celsius,
            quantity: Quantity::Temperature,
            prefix: BigDecimal::from(1),
        };

        let temp_fahrenheit = Measurement {
            value: 57.2,
            unit: Unit::Fahrenheit,
            quantity: Quantity::Temperature,
            prefix: BigDecimal::from(1),
        };

        let temp_kelvin = Measurement {
            value: 287.15,
            unit: Unit::Kelvin,
            quantity: Quantity::Temperature,
            prefix: BigDecimal::from(1),
        };

        // Test Celsius to Fahrenheit conversion
        let result_celsius_to_fahrenheit =
            convert(&temp_celsius, Unit::Fahrenheit, BigDecimal::from(1));
        assert!(result_celsius_to_fahrenheit.is_ok());
        assert_eq!(result_celsius_to_fahrenheit.unwrap(), temp_fahrenheit);

        // Test Celsius to Kelvin conversion
        let result_celsius_to_kelvin = convert(&temp_celsius, Unit::Kelvin, BigDecimal::from(1));
        assert!(result_celsius_to_kelvin.is_ok());
        assert_eq!(result_celsius_to_kelvin.unwrap(), temp_kelvin);

        // Test Fahrenheit to Kelvin conversion
        let result_fahrenheit_to_kelvin =
            convert(&temp_fahrenheit, Unit::Kelvin, BigDecimal::from(1));
        assert!(result_fahrenheit_to_kelvin.is_ok());
        assert_eq!(result_fahrenheit_to_kelvin.unwrap(), temp_kelvin);

        // Test Kelvin to Celsius conversion
        let result_kelvin_to_celsius = convert(&temp_kelvin, Unit::Celsius, BigDecimal::from(1));
        assert!(result_kelvin_to_celsius.is_ok());
        assert_eq!(result_kelvin_to_celsius.unwrap(), temp_celsius);

        // Test Kelvin to Fahrenheit conversion
        let result_kelvin_to_fahrenheit =
            convert(&temp_kelvin, Unit::Fahrenheit, BigDecimal::from(1));
        assert!(result_kelvin_to_fahrenheit.is_ok());
        assert_eq!(result_kelvin_to_fahrenheit.unwrap(), temp_fahrenheit);
    }

    /// Ensure correct conversion due to a difference in prefix
    #[test]
    fn test_convert_different_prefix() {
        let pressure_hecto_pascal = Measurement {
            value: 1015.15,
            unit: Unit::Pascal,
            quantity: Quantity::Pressure,
            prefix: BigDecimal::from(100), // Different prefix
        };

        let pressure_pascal = Measurement {
            value: 101515.0,
            unit: Unit::Pascal,
            quantity: Quantity::Pressure,
            prefix: BigDecimal::from(1), // Different prefix
        };

        // Test hecto pascal to pascal
        let result_hecto_pascal_to_pascal =
            convert(&pressure_hecto_pascal, Unit::Pascal, BigDecimal::from(1));
        assert!(result_hecto_pascal_to_pascal.is_ok());
        assert_eq!(result_hecto_pascal_to_pascal.unwrap(), pressure_pascal);
    }
}
