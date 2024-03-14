use crate::nodes::electric::generic_node;
use crate::units::electrical::power::Power;
use crate::units::electrical::utils::Unit;
use crate::units::electrical::voltage::Voltage;
use graph_processing::vertex::Vertex;
use graph_processing::vertex::VertexContext;

/// Load (or PQ) Bus: Represents a point in the system where power is consumed, typically modeling demands such as homes or businesses.
pub struct Load {
    pub generic_node: generic_node::GenericEnergyNode,
}

impl Load {
    pub fn new(power: Power, voltage: Voltage) -> Self {
        Load {
            generic_node: generic_node::GenericEnergyNode::new_type(
                generic_node::BusType::Load,
                Unit::Voltage(voltage),
                Unit::Power(power),
            ),
        }
    }
}
impl Vertex for Load {
    fn do_superstep(&mut self, _ctx: VertexContext) {
    }
}
