use crate::quantity::Quantity;
use strum::Display;

/// Represents a unit a [`Quantity`] can be measured in.
#[derive(sqlx::Type, enumset::EnumSetType, Debug, Hash, Display)]
#[sqlx(type_name = "unit", rename_all = "lowercase")]
pub enum Unit {
    Ampere,
    Candela,
    Celsius,
    Coulomb,
    Fahrenheit,
    Farad,
    Feet,
    Hertz,
    Joule,
    Kelvin,
    Kilogram,
    Lux,
    Metre,
    MillimetersPerHour,
    Newton,
    Nit,
    Ohm,
    Pascal,
    Pound,
    Utc,
    Volt,
    Watt,
}

impl Unit {
    /// Returns the [`Quantity`] that is measured in this unit.
    pub fn associated_quantity(self) -> Quantity {
        match self {
            Unit::Ampere => Quantity::Current,
            Unit::Candela => Quantity::LuminousIntensity,
            Unit::Celsius | Unit::Fahrenheit | Unit::Kelvin => Quantity::Temperature,
            Unit::Coulomb => Quantity::Charge,
            Unit::Farad => Quantity::Capacitance,
            Unit::Hertz => Quantity::Frequency,
            Unit::Joule => Quantity::Energy,
            Unit::Kilogram | Unit::Pound => Quantity::Mass,
            Unit::Lux => Quantity::Illuminance,
            Unit::Metre | Unit::Feet => Quantity::Length,
            Unit::MillimetersPerHour => Quantity::Rainfall,
            Unit::Newton => Quantity::Force,
            Unit::Nit => Quantity::Luminance,
            Unit::Ohm => Quantity::Resistance,
            Unit::Pascal => Quantity::Pressure,
            Unit::Utc => Quantity::Timestamp,
            Unit::Volt => Quantity::Potential,
            Unit::Watt => Quantity::Power,
        }
    }

    pub fn base_unit(self) -> Self {
        self.associated_quantity().associated_base_unit()
    }
}
