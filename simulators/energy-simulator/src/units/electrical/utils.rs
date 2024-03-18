use crate::units::electrical::admittance::Admittance;
use crate::units::electrical::power::Power;
use crate::units::electrical::voltage::Voltage;

#[derive(Clone, Debug, Copy)]
pub enum Unit {
    Voltage(Voltage),
    Admittance(Admittance),
    Power(Power),
}

impl Unit {
    /// Function to transform Unit to Voltage
    pub fn to_voltage(self) -> Option<Voltage> {
        match self {
            Unit::Voltage(voltage) => Some(Voltage::new(voltage.amplitude, voltage.angle)),
            _ => None,
        }
    }
    /// Function to transform Unit to admittance
    pub fn to_admittance(self) -> Option<Admittance> {
        match self {
            Unit::Admittance(admittance) => Some(Admittance::new(
                admittance.conductance,
                admittance.susceptance,
            )),
            _ => None,
        }
    }
    pub fn to_power(self) -> Option<Power> {
        match self {
            Unit::Power(power) => Some(Power::new(power.active, power.reactive)),
            _ => None,
        }
    }
}
