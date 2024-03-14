use std::collections::HashMap;

use crate::units::electrical::power::Power;
use crate::units::electrical::utils::Unit;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use graph_processing::vertex::Vertex;
use graph_processing::vertex::VertexContext;

#[derive(Clone, PartialEq, Debug)]
pub enum BusType {
    Generator,
    Load,
    Slack,
    Transmission,
}

pub trait Updatable {
    fn update(&mut self, new_values: Power);
    fn get_power(&self) -> Power;
}

pub trait EnergyNode {
    fn is_active(&self) -> bool;
    fn set_active(&mut self, active: bool);
    fn get_voltage(&self) -> Unit;
    fn get_power(&self) -> Unit;
    fn set_unit(&mut self, unit: Unit);
}

pub struct GenericEnergyNode {
    is_active: bool,
    pub voltage: Unit,
    pub power: Unit,
    bus_type: BusType,
    id: usize,
    /// For load/generators this holds transmission line id and their admittance
    /// For transmission nodes this holds the id of the connected nodes and their voltage
    neighbours: HashMap<usize, Unit>,
    slack_neighbours: HashMap<usize, (Unit, f64)>,
    lines: HashMap<usize, Unit>,
}

impl Vertex for GenericEnergyNode {
    fn do_superstep(&mut self, _ctx: VertexContext) {
    }
}

impl GenericEnergyNode {
    /// Constructor for GenericEnergyNode without an incremented static ID
    pub fn new(voltage: Unit, power: Unit) -> Self {
        GenericEnergyNode {
            is_active: true,
            voltage,
            bus_type: BusType::Load,
            id: Self::get_next_id(),
            neighbours: HashMap::new(),
            lines: HashMap::new(),
            slack_neighbours: HashMap::new(),
            power,
        }
    }
    pub fn new_type(bus_type: BusType, voltage: Unit, power: Unit) -> Self {
        GenericEnergyNode {
            is_active: true,
            voltage,
            bus_type,
            id: Self::get_next_id(),
            neighbours: HashMap::new(),
            lines: HashMap::new(),
            slack_neighbours: HashMap::new(),
            power,
        }
    }
    pub fn get_bus_type(&self) -> &BusType {
        &self.bus_type
    }
    fn get_next_id() -> usize {
        // Define a static counter and increment it
        static mut COUNTER: AtomicUsize = AtomicUsize::new(0);
        unsafe { COUNTER.fetch_add(1, Ordering::Relaxed) }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }
    pub fn add_neighbour(&mut self, id: usize, value: Unit) {
        // Add a value to the map with the current id
        self.neighbours.insert(id, value);
    }
    pub fn get_nr_neighbours(&self) -> usize {
        self.neighbours.len()
    }
    pub fn get_neighbours(&self) -> &HashMap<usize, Unit> {
        &self.neighbours
    }
    pub fn add_line(&mut self, id: usize, value: Unit) {
        // Add a value to the map with the current id
        self.lines.insert(id, value);
    }
    pub fn get_lines(&self) -> &HashMap<usize, Unit> {
        &self.lines
    }
    pub fn get_nr_lines(&self) -> usize {
        self.lines.len()
    }
    pub fn clear_neighbours(&mut self) {
        self.neighbours.clear();
    }
    pub fn add_slack_neighbour(&mut self, id: usize, value: Unit, angle: f64) {
        // Add a value to the map with the current id
        self.slack_neighbours.insert(id, (value, angle));
    }
    pub fn clear_slack_neighbours(&mut self) {
        self.slack_neighbours.clear();
    }
    pub fn get_nr_slack_neighbours(&self) -> usize {
        self.slack_neighbours.len()
    }
    pub fn get_slack_neighbours(&self) -> &HashMap<usize, (Unit, f64)> {
        &self.slack_neighbours
    }
}
impl EnergyNode for GenericEnergyNode {
    fn is_active(&self) -> bool {
        self.is_active
    }

    fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    fn set_unit(&mut self, unit: Unit) {
        match unit {
            Unit::Voltage(voltage) => {
                self.voltage = Unit::Voltage(voltage);
            }

            Unit::Power(power) => {
                self.power = Unit::Power(power);
            }
            Unit::Admittance(admittance) => {
                self.voltage = Unit::Admittance(admittance);
            }
            _ => {}
        }
    }
    fn get_voltage(&self) -> Unit {
        self.voltage
    }
    fn get_power(&self) -> Unit {
        self.power
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::electrical::voltage::Voltage;

    #[test]
    fn test_generic_node() {
        let voltage = Voltage::new(1.0, 0.0);
        let power = Power::new(1.0, 0.0);
        let node = GenericEnergyNode::new(Unit::Voltage(voltage), Unit::Power(power));
        assert!(node.is_active);
    }
}
