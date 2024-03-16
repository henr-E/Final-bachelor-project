use crate::{quantity::Quantity, unit::Unit, Signal, Signals};
use sqlx::types::BigDecimal;
use std::borrow::Cow;

pub struct Sensor<'a> {
    pub name: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    signals: Signals<'a>,
}

impl<'a> Sensor<'a> {
    pub(crate) fn builder(
        name: impl Into<Cow<'a, str>>,
        description: Option<impl Into<Cow<'a, str>>>,
    ) -> SensorBuilder<'a> {
        SensorBuilder::new(name, description)
    }

    pub fn signals(&self) -> &Signals<'a> {
        &self.signals
    }
}

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

    pub(crate) fn build(self) -> Sensor<'a> {
        Sensor {
            name: self.name,
            description: self.description,
            signals: self.signals,
        }
    }
}
