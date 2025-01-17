use crate::quantity::Quantity;
use strum::{Display, EnumString};

/// Represents a unit a [`Quantity`] can be measured in.
#[derive(sqlx::Type, enumset::EnumSetType, Debug, Hash, Display, EnumString)]
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
    Meter,
    MetersPerSecond,
    MillimetersPerHour,
    Mile,
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
    WattsPerSquareMetre,
}

impl Unit {
    /// Returns an [`EnumSet`] containing all [`Unit`] variants.
    ///
    /// [`EnumSet`]: enumset::EnumSet
    pub fn all() -> enumset::EnumSet<Self> {
        enumset::EnumSet::<Self>::all()
    }

    /// Returns the [`Quantity`] that is measured in this unit.
    pub fn associated_quantity(self) -> Quantity {
        match self {
            Unit::Ampere => Quantity::Current,
            Unit::Candela => Quantity::LuminousIntensity,
            Unit::Celsius | Unit::Fahrenheit | Unit::Kelvin => Quantity::Temperature,
            Unit::Coulomb => Quantity::Charge,
            Unit::Degrees => Quantity::WindDirection,
            Unit::Farad => Quantity::Capacitance,
            Unit::Hertz => Quantity::Frequency,
            Unit::Joule => Quantity::Energy,
            Unit::Kilogram | Unit::Pound => Quantity::Mass,
            Unit::Lux => Quantity::Illuminance,
            Unit::MetersPerSecond => Quantity::WindSpeed,
            Unit::Meter | Unit::Feet => Quantity::Length,
            Unit::Mile => Quantity::Length,
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
            Unit::WattsPerSquareMetre => Quantity::Irradiance,
        }
    }

    pub fn to_rink(self) -> String {
        match self {
            Unit::MetersPerSecond => "meters/second".to_string(),
            Unit::MillimetersPerHour => "millimeters/hour".to_string(),
            Unit::Percentage | Unit::Okta => "percent".to_string(),
            Unit::WattsPerSquareMetre => "W/m^2".to_string(),
            _ => self.to_string(),
        }
    }

    pub fn rink_multiplier(self) -> f64 {
        match self {
            Unit::Okta => 12.5,
            _ => 1.0,
        }
    }

    pub fn base_unit(self) -> Self {
        self.associated_quantity().associated_base_unit()
    }
}
