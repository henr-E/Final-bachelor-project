use crate::nodes::electric::generic;
use crate::units::electrical::power::Power;
use crate::units::electrical::utils::Unit;
use crate::units::electrical::voltage::Voltage;
use graph_processing::vertex::Vertex;
use graph_processing::vertex::VertexContext;

/// Load (or PQ) Bus: Represents a point in the system where power is consumed, typically modeling demands such as homes or businesses.
pub struct Load {
    pub generic: generic::GenericEnergyNode,
}

impl Load {
    pub fn new(power: Power, voltage: Voltage) -> Self {
        Load {
            generic: generic::GenericEnergyNode::new_type(
                generic::BusType::Load,
                Unit::Voltage(voltage),
                Unit::Power(power),
            ),
        }
    }
}
impl Vertex for Load {
    fn do_superstep(&mut self, _ctx: VertexContext) {}
}
