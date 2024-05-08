use std::{borrow::Cow, collections::HashMap};

use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgInterval, types::BigDecimal};
use uuid::Uuid;

use crate::{
    error::Error,
    quantity::Quantity,
    signal::{Signal, SignalValues, Signals},
    unit::Unit,
    SensorStore,
};

/// Represents a sensor with associated [`Signals`].
pub struct Sensor<'a> {
    /// Id of the sensor as registered in the database.
    pub id: Uuid,
    /// Name of the sensor.
    pub name: Cow<'a, str>,
    pub location: (f64, f64),
    pub description: Option<Cow<'a, str>>,
    /// Signals associated with the sensor.
    signals: Signals<'a>,
    pub twin_id: i32,
    pub building_id: Option<i32>,
}

impl<'a> Sensor<'a> {
    /// Returns a [`SensorBuilder`] used when retrieving a sensor from the database.
    pub fn builder(
        id: Uuid,
        name: impl Into<Cow<'a, str>>,
        description: Option<impl Into<Cow<'a, str>>>,
        location: impl Into<(f64, f64)>,
        twin_id: i32,
        building_id: Option<i32>,
    ) -> SensorBuilder<'a> {
        SensorBuilder::new(id, name, description, location, twin_id, building_id)
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
    ) -> Result<Option<HashMap<i32, SignalValues<'_, 's>>>, Error>
    where
        I: TryInto<PgInterval> + Clone,
        I::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        let mut result = HashMap::with_capacity(self.signals.len());

        for signal in self.signals.iter() {
            let signal_values = signal
                .values_for_interval_since_now(sensor_store, interval.clone())
                .await?;

            let Some(signal_values) = signal_values else {
                continue;
            };
            result.insert(signal.id, signal_values);
        }

        Ok(match result.is_empty() {
            true => None,
            false => Some(result),
        })
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
pub struct SensorBuilder<'a> {
    pub id: Uuid,
    pub name: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    pub location: (f64, f64),
    pub signals: Signals<'a>,
    pub twin_id: i32,
    pub building_id: Option<i32>,
}

impl<'a> SensorBuilder<'a> {
    pub fn new(
        id: Uuid,
        name: impl Into<Cow<'a, str>>,
        description: Option<impl Into<Cow<'a, str>>>,
        location: impl Into<(f64, f64)>,
        twin_id: i32,
        building_id: Option<i32>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.map(|d| d.into()),
            location: location.into(),
            signals: Signals::new(),
            twin_id,
            building_id,
        }
    }

    /// Adds a single [`Signal`] to the [`Sensor`].
    pub fn add_signal(
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
    pub fn build(self) -> Sensor<'a> {
        Sensor {
            id: self.id,
            name: self.name,
            description: self.description,
            location: self.location,
            signals: self.signals,
            twin_id: self.twin_id,
            building_id: self.building_id,
        }
    }
}
