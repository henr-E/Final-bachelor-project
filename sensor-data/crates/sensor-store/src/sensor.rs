use crate::{
    error::Error,
    quantity::Quantity,
    signal::{Signal, SignalValues, Signals},
    unit::Unit,
    SensorStore,
};
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgInterval, types::BigDecimal};
use std::{borrow::Cow, collections::HashMap};
use uuid::Uuid;

/// Represents a sensor with associated [`Signals`].
pub struct Sensor<'a> {
    /// Id of the sensor as registered in the database.
    pub id: Uuid,
    /// Name of the sensor.
    pub name: Cow<'a, str>,
    /// Description of the sensor.
    pub description: Option<Cow<'a, str>>,
    /// Signals associated with the sensor.
    signals: Signals<'a>,
}

impl<'a> Sensor<'a> {
    /// Returns a [`SensorBuilder`] used when retrieving a sensor from the database.
    pub(crate) fn builder(
        id: Uuid,
        name: impl Into<Cow<'a, str>>,
        description: Option<impl Into<Cow<'a, str>>>,
    ) -> SensorBuilder<'a> {
        SensorBuilder::new(id, name, description)
    }

    /// Returns the [`Signals`] being measured by this sensor.
    pub fn signals(&self) -> &Signals<'a> {
        &self.signals
    }

    /// For every [`Signal`] of the [`Sensor`], get the [`SignalValues`] whose timestamp is within
    /// the provided `interval` into the past starting from `now`.
    ///
    /// NOTE: This function assumes that the `sensor_signal_id` (from the database) is unique over
    /// all sensors.
    pub async fn signal_values_for_interval_since_now<'s, I>(
        &'s self,
        sensor_store: &SensorStore,
        interval: I,
    ) -> Result<HashMap<i32, SignalValues<'_, 's>>, Error>
    where
        I: TryInto<PgInterval> + Clone,
        I::Error: std::error::Error + Send + Sync + 'static,
    {
        let mut result = HashMap::with_capacity(self.signals.len());

        for signal in self.signals.iter() {
            let signal_values = signal
                .values_for_interval_since_now(sensor_store, interval.clone())
                .await?;

            result.insert(signal.id, signal_values);
        }

        Ok(result)
    }

    /// For every [`Signal`] of the [`Sensor`], get the [`SignalValues`] whose timestamp is between
    /// the provided timestamps.
    ///
    /// Which timestamp is earlier than the other does not matter as this is flipped automatically.
    ///
    /// NOTE: This function assumes that the `sensor_signal_id` (from the database) is unique over
    /// all sensors.
    pub async fn signal_values_between_timestamps<'s>(
        &'s self,
        sensor_store: &SensorStore,
        start: impl Into<DateTime<Utc>> + Clone,
        end: impl Into<DateTime<Utc>> + Clone,
    ) -> Result<HashMap<i32, SignalValues<'_, 's>>, Error> {
        let mut result = HashMap::with_capacity(self.signals.len());

        for signal in self.signals.iter() {
            let signal_values = signal
                .values_between_timestamps(sensor_store, start.clone(), end.clone())
                .await?;

            result.insert(signal.id, signal_values);
        }

        Ok(result)
    }
}

/// Represents a sensor while it is being built from entries in the database.
pub(crate) struct SensorBuilder<'a> {
    pub(crate) id: Uuid,
    pub(crate) name: Cow<'a, str>,
    pub(crate) description: Option<Cow<'a, str>>,
    pub(crate) signals: Signals<'a>,
}

impl<'a> SensorBuilder<'a> {
    fn new(
        id: Uuid,
        name: impl Into<Cow<'a, str>>,
        description: Option<impl Into<Cow<'a, str>>>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.map(|d| d.into()),
            signals: Signals::new(),
        }
    }

    /// Adds a single [`Signal`] to the [`Sensor`].
    pub(crate) fn add_signal(
        &mut self,
        id: i32,
        name: impl Into<Cow<'a, str>>,
        quantity: Quantity,
        unit: Unit,
        prefix: BigDecimal,
    ) {
        self.signals.insert(Signal {
            id,
            name: name.into(),
            quantity,
            unit,
            prefix,
        });
    }

    /// Locks in the [`SensorBuilder`] and constructs an actual [`Sensor`] from it.
    pub(crate) fn build(self) -> Sensor<'a> {
        Sensor {
            id: self.id,
            name: self.name,
            description: self.description,
            signals: self.signals,
        }
    }
}
