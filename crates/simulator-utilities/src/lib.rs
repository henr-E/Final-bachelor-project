//! Small utility crate to share some common functionality required by different simulators.

/// module for utility functions related to sensors and sensor data retrieval
pub mod sensor {
    use bigdecimal::ToPrimitive;
    use sqlx::types::BigDecimal;
    use thiserror::Error;

    use sensor_store::{Quantity, Sensor, SensorStore};

    /// Retrieve all values for the `Signal` matching the specified [`Quantity`] and the id of the provided [`Sensor`].
    ///
    /// Retrieves all entries in sensor_values.value where sensor_values.sensor_signal_id refers to a
    /// sensor_signals object corresponding with the provided quantity and the id of the provided sensor.
    /// Returns an error if the entries can not be converted or if no sensor_signals object corresponds with the id of the provided sensor and the provided quantity.
    ///
    /// # Errors
    ///
    /// Returns an error if no `Signal` corresponds to the provided [`Quantity`].
    async fn values_for_quantity(
        sensor_store: &SensorStore,
        sensor: &Sensor<'_>,
        quantity: Quantity,
    ) -> Result<Vec<BigDecimal>, Error> {
        // Check if any signal's quantity matches the specified quantity
        for signal in sensor.signals().iter() {
            if signal.quantity == quantity {
                let signal_values = signal.values(sensor_store).await?;
                return Ok(signal_values
                    .values
                    .iter()
                    .map(|v| v.value.clone())
                    .collect());
            }
        }
        // Quantity not found in signals, return Error
        Err(Error::QuantityNotFoundForSensor(
            quantity,
            sensor.name.to_string(),
        ))
    }
    /// Averages a dataset.
    ///
    /// This function will make chunks of `average_amt` and average them.
    /// The function is used because the dataset from the generator is quite sparse, since it only
    /// generates unique datapoints per hour, while the generator pushes it a lot more frequently.
    pub fn average_dataset(dataset: &mut Vec<f64>, average_amt: usize) {
        *dataset = dataset
            .chunks(average_amt)
            .map(|chunk| chunk.iter().sum::<f64>() / chunk.len() as f64)
            .collect();
    }

    /// Retrieve all values  as f64's for the `Signal` matching the specified [`Quantity`] and the id of the provided [`Sensor`].
    ///
    /// Retrieves all entries in sensor_values.value as f64's where sensor_values.sensor_signal_id refers to a
    /// sensor_signals object corresponding with the provided quantity and the id of the provided sensor.
    /// Returns an error if the entries can not be converted to f64's or if no sensor_signals object corresponds with the id of the provided sensor and the provided quantity.
    ///
    /// # Errors
    ///
    /// Returns an error if no `Signal` corresponds to the provided [`Quantity`] or the values can not be represented
    /// as f64.
    pub async fn values_for_quantity_as_f64(
        sensor_store: &SensorStore,
        sensor: &Sensor<'_>,
        quantity: Quantity,
    ) -> Result<Vec<f64>, Error> {
        // Retrieve values (BigDecimal) for the specified quantity
        let values_as_big_decimals: Vec<BigDecimal> =
            values_for_quantity(sensor_store, sensor, quantity).await?;
        // Convert BigDecimals to f64's.
        let values_as_floats: Result<Vec<f64>, Error> = values_as_big_decimals
            .into_iter()
            .map(|bd| {
                bd.to_f64().ok_or_else(|| {
                    Error::NumberFormatConversion(
                        bd.to_string(),
                        String::from("BigDecimal"),
                        String::from("f64"),
                    )
                })
            })
            .collect();
        values_as_floats
    }

    /// Simulator-utilities error type related to sensor and sensor data retrieval.
    #[derive(Error, Debug)]
    pub enum Error {
        #[error("Database error {0}")]
        Database(#[from] sensor_store::Error),
        #[error("Failed to convert value '{0}' from format '{1}' to format '{2}'.")]
        NumberFormatConversion(String, String, String),
        #[error("Quantity '{0}' not found for sensor '{1}'.")]
        QuantityNotFoundForSensor(Quantity, String),
    }
}
