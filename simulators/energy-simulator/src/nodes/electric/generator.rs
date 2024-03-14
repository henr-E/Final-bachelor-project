use crate::nodes::electric::generic_node;
use crate::units::electrical::power::Power;
use crate::units::electrical::utils::Unit;
use crate::units::electrical::voltage::Voltage;
use graph_processing::vertex::Vertex;
use graph_processing::vertex::VertexContext;

/// Generator (or PV) Bus: Represents a point where power is generated, allowing control over the amount of electricity supplied to the network.
pub struct Generator {
    pub generic_node: generic_node::GenericEnergyNode,

    /// Capacity in megawatts
    pub capacity: f64,
    /// Energy produced in MWH
    pub energy_production: f64,
    /// e.g "fossil", "nuclear", "renewable"
    pub power_type: String,
}
impl Generator {
    pub fn new(capacity: f64, power: Power, voltage: Voltage) -> Self {
        Generator {
            generic_node: generic_node::GenericEnergyNode::new_type(
                generic_node::BusType::Generator,
                Unit::Voltage(voltage),
                Unit::Power(power),
            ),
            capacity,
            power_type: "".to_string(),
            energy_production: 0.0,
        }
    }
}

impl Vertex for Generator {
    fn do_superstep(&mut self, _ctx: VertexContext) {
    }
}
