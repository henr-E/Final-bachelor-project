use std::str::FromStr;

use crate::quantity::Quantity;
use strum::Display;

/// Represents a unit a [`Quantity`] can be measured in.
#[derive(sqlx::Type, enumset::EnumSetType, Debug, Hash, Display)]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "unit", rename_all = "lowercase")]
pub enum Unit {
    Ampere,
    Candela,
    Celsius,
    Coulomb,
    Degrees,
    Fahrenheit,
    Farad,
    Feet,
    Hertz,
    Joule,
    Kelvin,
    Kilogram,
    Lux,
    Metre,
    MetersPerSecond,
    MillimetersPerHour,
    Newton,
    Nit,
    Ohm,
    Okta,
    Pascal,
    Percentage,
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
            Unit::Degrees => Quantity::WindDirection,
            Unit::Coulomb => Quantity::Charge,
            Unit::Farad => Quantity::Capacitance,
            Unit::Hertz => Quantity::Frequency,
            Unit::Joule => Quantity::Energy,
            Unit::Kilogram | Unit::Pound => Quantity::Mass,
            Unit::Lux => Quantity::Illuminance,
            Unit::MetersPerSecond => Quantity::WindSpeed,
            Unit::Metre | Unit::Feet => Quantity::Length,
            Unit::MillimetersPerHour => Quantity::Rainfall,
            Unit::Newton => Quantity::Force,
            Unit::Nit => Quantity::Luminance,
            Unit::Ohm => Quantity::Resistance,
            Unit::Okta => Quantity::Cloudiness,
            Unit::Pascal => Quantity::Pressure,
            Unit::Percentage => Quantity::RelativeHumidity,
            Unit::Utc => Quantity::Timestamp,
            Unit::Volt => Quantity::Potential,
            Unit::Watt => Quantity::Power,
        }
    }

    pub fn base_unit(self) -> Self {
        self.associated_quantity().associated_base_unit()
    }
}
impl FromStr for Unit {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ampere" => Ok(Unit::Ampere),
            "candela" => Ok(Unit::Candela),
            "celsius" => Ok(Unit::Celsius),
            "coulomb" => Ok(Unit::Coulomb),
            "fahrenheit" => Ok(Unit::Fahrenheit),
            "farad" => Ok(Unit::Farad),
            "feet" => Ok(Unit::Feet),
            "hertz" => Ok(Unit::Hertz),
            "joule" => Ok(Unit::Joule),
            "kelvin" => Ok(Unit::Kelvin),
            "kilogram" => Ok(Unit::Kilogram),
            "lux" => Ok(Unit::Lux),
            "metre" => Ok(Unit::Metre),
            "millimetersperhour" => Ok(Unit::MillimetersPerHour),
            "newton" => Ok(Unit::Newton),
            "nit" => Ok(Unit::Nit),
            "ohm" => Ok(Unit::Ohm),
            "pascal" => Ok(Unit::Pascal),
            "pound" => Ok(Unit::Pound),
            "volt" => Ok(Unit::Volt),
            "watt" => Ok(Unit::Watt),
            _ => Err(()),
        }
    }
}
