use crate::nodes::electric::generic_node;
use crate::units::electrical::power::Power;
use crate::units::electrical::utils::Unit;
use crate::units::electrical::voltage::Voltage;
use graph_processing::vertex::Vertex;
use graph_processing::vertex::VertexContext;
pub struct Slack {
    pub theta: f64,
    pub generic_node: generic_node::GenericEnergyNode,
}

/// Slack represents a node in the power grid that is the reference point for voltage and phase angle.
/// (angle in radians)
impl Slack {
    pub fn new(voltage: Voltage, theta: f64) -> Self {
        Slack {
            generic_node: generic_node::GenericEnergyNode::new_type(
                generic_node::BusType::Slack,
                Unit::Voltage(voltage),
                Unit::Power(Power::new(20.0, 20.0)),
            ),
            //in radians
            theta,
        }
    }
    pub fn set_phase_angle(&mut self, active_power: f64, s: f64) {
        if s == 0.0 {
            self.theta = 0.0;
            return;
        }
        self.theta = 1.0 / (active_power / s).acos().cos();
    }
    pub fn get_phase_angle(&self) -> f64 {
        self.theta
    }
}
impl Vertex for Slack {
    fn do_superstep(&mut self, _ctx: VertexContext) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::electrical::voltage::Voltage;

    #[test]
    fn test_slackbus_node() {
        let voltage = Voltage::new(1.0, 0.0);
        let slackbus = Slack::new(voltage, 0.0);
        assert_eq!(slackbus.theta, 0.0);
    }
}
