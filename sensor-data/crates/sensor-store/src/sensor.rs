use crate::{quantity::Quantity, unit::Unit, Signal, Signals};
use sqlx::types::BigDecimal;
use std::borrow::Cow;

/// Represents a sensor with associated [`Signals`].
pub struct Sensor<'a> {
    pub name: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    signals: Signals<'a>,
}

impl<'a> Sensor<'a> {
    /// Returns a [`SensorBuilder`] used when retrieving a sensor from the database.
    pub(crate) fn builder(
        name: impl Into<Cow<'a, str>>,
        description: Option<impl Into<Cow<'a, str>>>,
    ) -> SensorBuilder<'a> {
        SensorBuilder::new(name, description)
    }

    /// Returns the [`Signals`] being measured by this sensor.
    pub fn signals(&self) -> &Signals<'a> {
        &self.signals
    }
}

/// Represents a sensor while it is being built from entries in the database.
pub(crate) struct SensorBuilder<'a> {
    name: Cow<'a, str>,
    description: Option<Cow<'a, str>>,
    signals: Signals<'a>,
}

impl<'a> SensorBuilder<'a> {
    fn new(name: impl Into<Cow<'a, str>>, description: Option<impl Into<Cow<'a, str>>>) -> Self {
        Self {
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
            name: self.name,
            description: self.description,
            signals: self.signals,
        }
    }
}
