use crate::nodes::electric::generic;
use crate::units::electrical::power::Power;
use crate::units::electrical::utils::Unit;
use crate::units::electrical::voltage::Voltage;
use graph_processing::vertex::Vertex;
use graph_processing::vertex::VertexContext;

/// Slack represents a node in the power grid that is the reference point for voltage and phase angle.
pub struct Slack {
    pub phase_angle: f64,
    pub generic: generic::GenericEnergyNode,
}

impl Slack {
    pub fn new(voltage: Voltage, phase_angle: f64) -> Self {
        Slack {
            generic: generic::GenericEnergyNode::new_type(
                generic::BusType::Slack,
                Unit::Voltage(voltage),
                Unit::Power(Power::new(20.0, 20.0)),
            ),
            //in radians
            phase_angle,
        }
    }

    pub fn set_phase_angle(&mut self, active_power: f64, s: f64) {
        if s == 0.0 {
            self.phase_angle = 0.0;
            return;
        }
        self.phase_angle = 1.0 / (active_power / s).acos().cos();
    }

    pub fn get_phase_angle(&self) -> f64 {
        self.phase_angle
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
        assert_eq!(slackbus.phase_angle, 0.0);
    }
}
