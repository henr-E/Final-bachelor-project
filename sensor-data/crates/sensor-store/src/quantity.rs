use crate::unit::Unit;
use std::collections::HashSet;
use strum::{Display, EnumString};

/// Kind of signal the sensor is measuring.
#[derive(sqlx::Type, enumset::EnumSetType, Debug, Hash, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "quantity", rename_all = "lowercase")]
pub enum Quantity {
    Capacitance,
    Charge,
    Cloudiness,
    Current,
    Energy,
    Force,
    Frequency,
    Illuminance,
    Irradiance,
    Length,
    Luminance,
    LuminousIntensity,
    Mass,
    Potential,
    Power,
    Pressure,
    Rainfall,
    RelativeHumidity,
    Resistance,
    Temperature,
    Timestamp,
    WindDirection,
    WindSpeed,
}

impl Quantity {
    /// Returns an [`EnumSet`] returning all [`Quantity`] variants.
    ///
    /// [`EnumSet`]: enumset::EnumSet
    pub fn all() -> enumset::EnumSet<Self> {
        enumset::EnumSet::<Self>::all()
    }

    /// Get the base [`Unit`] associated with the sensor quantity.
    pub fn associated_base_unit(self) -> Unit {
        match self {
            Quantity::Capacitance => Unit::Farad,
            Quantity::Charge => Unit::Coulomb,
            Quantity::Cloudiness => Unit::Okta,
            Quantity::Current => Unit::Ampere,
            Quantity::Energy => Unit::Joule,
            Quantity::Force => Unit::Newton,
            Quantity::Frequency => Unit::Hertz,
            Quantity::Illuminance => Unit::Lux,
            Quantity::Irradiance => Unit::WattsPerSquareMetre,
            Quantity::Length => Unit::Meter,
            Quantity::Luminance => Unit::Nit,
            Quantity::LuminousIntensity => Unit::Candela,
            Quantity::Mass => Unit::Kilogram,
            Quantity::Potential => Unit::Volt,
            Quantity::Power => Unit::Watt,
            Quantity::Pressure => Unit::Pascal,
            Quantity::Rainfall => Unit::MillimetersPerHour,
            Quantity::RelativeHumidity => Unit::Percentage,
            Quantity::Resistance => Unit::Ohm,
            Quantity::Temperature => Unit::Celsius,
            Quantity::Timestamp => Unit::Utc,
            Quantity::WindDirection => Unit::Degrees,
            Quantity::WindSpeed => Unit::MetersPerSecond,
        }
    }

    /// Get all supported [`Unit`s](Unit) associated with the sensor quantity.
    pub fn associated_units(self) -> HashSet<Unit> {
        HashSet::from_iter(match self {
            Quantity::Capacitance => vec![Unit::Farad],
            Quantity::Charge => vec![Unit::Coulomb],
            Quantity::Cloudiness => vec![Unit::Okta],
            Quantity::Current => vec![Unit::Ampere],
            Quantity::Energy => vec![Unit::Joule],
            Quantity::Force => vec![Unit::Newton],
            Quantity::Frequency => vec![Unit::Hertz],
            Quantity::Illuminance => vec![Unit::Lux],
            Quantity::Irradiance => vec![Unit::WattsPerSquareMetre],
            Quantity::Length => vec![Unit::Meter, Unit::Feet, Unit::Mile],
            Quantity::Luminance => vec![Unit::Nit],
            Quantity::LuminousIntensity => vec![Unit::Candela],
            Quantity::Mass => vec![Unit::Kilogram, Unit::Pound],
            Quantity::Potential => vec![Unit::Volt],
            Quantity::Power => vec![Unit::Watt],
            Quantity::Pressure => vec![Unit::Pascal],
            Quantity::Rainfall => vec![Unit::MillimetersPerHour],
            Quantity::Resistance => vec![Unit::Ohm],
            Quantity::Temperature => vec![Unit::Celsius, Unit::Fahrenheit, Unit::Kelvin],
            Quantity::Timestamp => vec![Unit::Utc],
            Quantity::WindSpeed => vec![Unit::MetersPerSecond],
            Quantity::WindDirection => vec![Unit::Degrees],
            Quantity::RelativeHumidity => vec![Unit::Percentage],
        })
    }
}
