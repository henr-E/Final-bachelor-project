use crate::{
    nodes::electric::generic::{self},
    units::electrical::{
        admittance::Admittance, current::Current, impedance::Impedance, power::Power, utils::Unit,
        voltage::Voltage,
    },
};

use graph_processing::vertex::{Vertex, VertexContext};
use num_complex::{Complex, ComplexFloat};

/// Transmission Line: Represents the transmission line that carries electrical power, linking power sources with consumption areas.
pub struct Transmission {
    pub generic: generic::GenericEnergyNode,
    /// Operating voltage in kilovolts (kV)
    pub operating_voltage: f64,
    /// Maximum capacity in megawatts (MW)
    pub maximum_power_capacity: f64,
    /// Current capacity in megawatts (MW)
    pub current_capacity: f64,
    /// Ohms per meter
    pub resistance_per_meter: f64,
    /// Ohms per meter for AC lines
    pub reactance_per_meter: f64,
    /// Length of the transmission line in meters (m)
    pub length: f64,
    pub admittance: Admittance,
    shunt_admittance: Admittance,
    pub current: Current,
}

impl Transmission {
    pub fn new(
        operating_voltage: f64,
        maximum_power_capacity: f64,
        resistance_per_meter: f64,
        reactance_per_meter: f64,
        length: f64,
        admittance: Admittance,
    ) -> Self {
        Transmission {
            generic: generic::GenericEnergyNode::new_type(
                generic::BusType::Transmission,
                Unit::Admittance(admittance),
                Unit::Power(Power::new(0.0, 0.0)),
            ),
            operating_voltage,
            maximum_power_capacity,
            resistance_per_meter,
            reactance_per_meter,
            length,
            current_capacity: 0.0,
            admittance,
            shunt_admittance: Admittance::new(0.0, 0.0),
            current: Current::new(0.0, 0.0),
        }
    }

    fn get_series_impedance(&self) -> Impedance {
        Impedance::from_complex(Complex::new(0.01, 0.01))
    }

    /// Calculates the current at the sending end of a transmission line (the point at which electrical power enters)
    pub fn calculate_sending_current(&self, v_sending: Voltage, v_receiving: Voltage) -> Current {
        let v_sending_phasor = v_sending.to_complex();
        let v_receiving_phasor = v_receiving.to_complex();
        let delta_v = v_sending_phasor - v_receiving_phasor;
        let series_impedance = self.get_series_impedance();
        let current_sending = series_impedance.to_complex().recip() * delta_v
            + (self.get_shunt_susceptance() / 2.0) * v_sending_phasor;
        Current::from_complex(current_sending)
    }

    fn get_shunt_susceptance(&self) -> f64 {
        if self.length / 1000.0 < 80.0 {
            return 0.0;
        }
        self.shunt_admittance.susceptance
    }
    pub fn get_angle(&self) -> f64 {
        (self.admittance.susceptance / self.admittance.conductance).atan2(1.0)
    }
}

impl Vertex for Transmission {
    fn do_superstep(&mut self, _ctx: VertexContext) {}
}
