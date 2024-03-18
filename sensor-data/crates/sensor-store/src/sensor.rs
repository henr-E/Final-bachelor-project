use crate::{quantity::Quantity, unit::Unit};
use sqlx::types::BigDecimal;
use std::{borrow::Cow, collections::HashSet};
use uuid::Uuid;

/// Represents a sensor with associated [`Signals`].
pub struct Sensor<'a> {
    /// Id of the sensor as registered in the database.
    pub id: Uuid,
    pub name: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    signals: Signals<'a>,
}

pub type Signals<'a> = HashSet<Signal<'a>>;

/// Represents a signal field when ingesting sensor data.
#[derive(PartialEq, Eq, Hash)]
pub struct Signal<'a> {
    /// Name of the field in the data.
    pub name: Cow<'a, str>,
    /// quantity of the signal.
    pub quantity: Quantity,
    /// Unit of the field in the data.
    pub unit: Unit,
    /// Prefix of the value compared to the unit.
    pub prefix: BigDecimal,
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
        name: impl Into<Cow<'a, str>>,
        quantity: Quantity,
        unit: Unit,
        prefix: BigDecimal,
    ) {
        self.signals.insert(Signal {
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
