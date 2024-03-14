use crate::{
    nodes::electric::generic_node::{self},
    units::electrical::{
        admittance::Admittance,
        current::Current,
        impedance::Impedance,
        power::Power,
        utils::{active_power, Unit},
        voltage::Voltage,
    },
};

use graph_processing::vertex::{Vertex, VertexContext};
use num_complex::{Complex, ComplexFloat};

#[allow(dead_code)]
pub struct Transmission {
    pub generic_node: generic_node::GenericEnergyNode,
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

#[allow(dead_code)]
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
            generic_node: generic_node::GenericEnergyNode::new_type(
                generic_node::BusType::Transmission,
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
    /// Calculate the total resistance (Rtotal) of the transmission line (in Ohms)
    pub fn total_resistance(&self) -> f64 {
        self.resistance_per_meter * self.length
    }

    /// Calculate the total reactance (Xtotal) of the transmission line (in Ohms)
    pub fn total_reactance(&self) -> f64 {
        self.reactance_per_meter * self.length
    }

    /// Calculate the impedance (Z) of the transmission line (in Ohms)
    pub fn impedance(&self) -> f64 {
        let resistance = self.total_resistance();
        let reactance = self.total_reactance();

        // Calculate the square root of the sum of the squares of resistance and reactance
        (resistance.powi(2) + reactance.powi(2)).sqrt()
    }

    /// Calculate the current (I) flowing through the transmission line (in Ampere)
    pub fn current(&self) -> f64 {
        let impedance = self.impedance();
        let operating_voltage_v = self.operating_voltage * 1000.0; // Convert kV to V

        operating_voltage_v / impedance
    }

    /// Calculate the power loss in the transmission line (in Watts)
    pub fn power_loss(&self) -> f64 {
        let current = self.current();
        let total_resistance = self.total_resistance();

        current.powi(2) * total_resistance
    }
    pub fn set_current(&mut self, current: f64) {
        self.current_capacity = current;
    }
    pub fn get_current(&self) -> f64 {
        self.current_capacity
    }

    /// Calculates the current at the sending end of a transmission line (the point at which electrical power enters)
    pub fn calculate_sending_current(&self, v_sending: Voltage, v_receiving: Voltage) -> Current {
        let v_sending_phasor = v_sending.to_complex();
        let v_receiving_phasor = v_receiving.to_complex();
        let delta_v = v_sending_phasor - v_receiving_phasor;
        let series_impedance = Complex::new(0.01, 0.01); // TODO: Remove magic number/Calculate from grid
        let current_sending = series_impedance.recip() * delta_v
            + (self.get_shunt_susceptance() / 2.0) * v_sending_phasor;
        Current::from_complex(current_sending)
    }

    pub fn calculate_sending_active_power(&self, v_sending: Voltage, v_receiving: Voltage) -> f64 {
        let delta_angle = v_sending.angle - v_receiving.angle;
        let series_impedance = Impedance {
            resistance: 0.01,
            reactance: 0.01,
        }; // TODO: Remove magic number/Calculate from grid


        active_power(v_sending, v_receiving, delta_angle, series_impedance)
    }

    pub fn calculate_receiving_active_power(
        &self,
        v_sending: Voltage,
        v_receiving: Voltage,
    ) -> f64 {
        let delta_angle = v_sending.angle - v_receiving.angle;
        let series_impedance = Impedance {
            resistance: 0.01,
            reactance: 0.01,
        }; // TODO: Remove magic number/Calculate from grid


        active_power(v_receiving, v_sending, delta_angle, series_impedance)
    }

    pub fn calculate_sending_reactive_power(
        &self,
        v_sending: Voltage,
        v_receiving: Voltage,
    ) -> f64 {
        let delta_angle = v_sending.angle - v_receiving.angle;
        let series_admittance = Complex::new(0.01, 0.01).recip(); // TODO: Remove magic number/Calculate from grid
        let gij = series_admittance.re; // Series conductance
        let bij = series_admittance.im; // Series susceptance



        -bij * v_sending.amplitude.powi(2)
            - v_sending.amplitude
                * v_receiving.amplitude
                * (gij * delta_angle.cos() - bij * delta_angle.sin())
            - (self.get_shunt_susceptance() / 2.0) * v_sending.amplitude.powi(2)
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
impl generic_node::Updatable for Transmission {
    fn update(&mut self, update_transmission: Power) {
        self.admittance = Admittance::new(update_transmission.active, update_transmission.reactive);
    }
    fn get_power(&self) -> Power {
        Power {
            active: 0.0,
            reactive: 0.0,
        }
    }
}

impl Vertex for Transmission {
    fn do_superstep(&mut self, _ctx: VertexContext) {
    }
}
