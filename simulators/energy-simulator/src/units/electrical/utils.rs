use crate::units::electrical::admittance::Admittance;
use crate::units::electrical::current::Current;
use crate::units::electrical::impedance::Impedance;
use crate::units::electrical::power::Power;
use crate::units::electrical::voltage::Voltage;
use num_complex::ComplexFloat;

#[derive(Clone, Debug, Copy)]
#[allow(dead_code)]
pub enum Unit {
    Voltage(Voltage),
    Admittance(Admittance),
    Power(Power),
    Current(Current),
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

pub fn active_power(
    v1: Voltage,
    v2: Voltage,
    delta_angle: f64,
    series_impedance: Impedance,
) -> f64 {
    let series_admittance = series_impedance.to_complex().recip();
    let gij = series_admittance.re; // Series conductance
    let bij = series_admittance.im; // Series susceptance

    gij * v1.amplitude.powi(2)
        - v1.amplitude * v2.amplitude * (gij * delta_angle.cos() + bij * delta_angle.sin())
}
