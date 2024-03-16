use std::collections::HashSet;

use crate::unit::Unit;

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[sqlx(type_name = "quantity", rename_all = "lowercase")]
pub enum Quantity {
    Capacitance,
    Charge,
    Current,
    Energy,
    Force,
    Frequency,
    Illuminance,
    Length,
    Luminance,
    LuminousIntensity,
    Mass,
    Potential,
    Power,
    Pressure,
    Rainfall,
    Resistance,
    Temperature,
}

impl Quantity {
    pub fn associated_base_unit(self) -> Unit {
        match self {
            Quantity::Capacitance => Unit::Farad,
            Quantity::Charge => Unit::Watt,
            Quantity::Current => Unit::Ampere,
            Quantity::Energy => Unit::Joule,
            Quantity::Force => Unit::Newton,
            Quantity::Frequency => Unit::Hertz,
            Quantity::Illuminance => Unit::Lux,
            Quantity::Length => Unit::Metre,
            Quantity::Luminance => Unit::Nit,
            Quantity::LuminousIntensity => Unit::Candela,
            Quantity::Mass => Unit::Kilogram,
            Quantity::Potential => Unit::Volt,
            Quantity::Power => Unit::Watt,
            Quantity::Pressure => Unit::Pascal,
            Quantity::Rainfall => Unit::MillimetersPerHour,
            Quantity::Resistance => Unit::Ohm,
            Quantity::Temperature => Unit::Celsius,
        }
    }

    pub fn associated_units(self) -> HashSet<Unit> {
        HashSet::from_iter(match self {
            Quantity::Capacitance => vec![Unit::Farad],
            Quantity::Charge => vec![Unit::Watt],
            Quantity::Current => vec![Unit::Ampere],
            Quantity::Energy => vec![Unit::Joule],
            Quantity::Force => vec![Unit::Newton],
            Quantity::Frequency => vec![Unit::Hertz],
            Quantity::Illuminance => vec![Unit::Lux],
            Quantity::Length => vec![Unit::Metre, Unit::Feet],
            Quantity::Luminance => vec![Unit::Nit],
            Quantity::LuminousIntensity => vec![Unit::Candela],
            Quantity::Mass => vec![Unit::Kilogram, Unit::Pound],
            Quantity::Potential => vec![Unit::Volt],
            Quantity::Power => vec![Unit::Watt],
            Quantity::Pressure => vec![Unit::Pascal],
            Quantity::Rainfall => vec![Unit::MillimetersPerHour],
            Quantity::Resistance => vec![Unit::Ohm],
            Quantity::Temperature => vec![Unit::Celsius, Unit::Fahrenheit, Unit::Kelvin],
        })
    }
}
