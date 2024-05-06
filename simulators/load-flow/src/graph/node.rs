use crate::units::{power::Power, voltage::Voltage};
use core::sync::atomic::{AtomicUsize, Ordering};

/// Represents the type of bus in an electrical network.
/// Be sure to read the documentation on what these types are
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BusType {
    /// Usually called PV in literature, but we understand this can be confusing
    Generator,
    /// Usually called PQ
    Load,
    /// Also known as reference bus or swing bus
    Slack,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PowerType {
    Fossil,
    Renewable,
    Nuclear,
    Hydro,
    Solar,
    Wind,
    Battery,
    Storage,
    Load,
}

#[derive(Clone, Debug, Copy)]
pub struct BusNode {
    id: usize,
    active: bool,
    bus_type: BusType,
    power: Power,
    voltage: Voltage,
    energy_type: PowerType,
}

impl BusNode {
    fn next_id() -> usize {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    pub fn generator(
        active_power_pu: f64,
        voltage_magnitude_pu: f64,
        power_type: PowerType,
    ) -> Self {
        BusNode {
            id: Self::next_id(),
            active: true,
            bus_type: BusType::Generator,
            power: Power::new(active_power_pu, 0.001),
            voltage: Voltage::new(voltage_magnitude_pu, 0.001),
            energy_type: power_type,
        }
    }

    pub fn load(active_power_pu: f64, reactive_power_pu: f64) -> Self {
        BusNode {
            id: Self::next_id(),
            active: true,
            bus_type: BusType::Load,
            power: Power::new(active_power_pu.abs(), reactive_power_pu),
            voltage: Voltage::new(1.0, 0.001),
            energy_type: PowerType::Load,
        }
    }
    pub fn set_pu(&mut self, v_base: f64, s_base: f64) {
        if self.bus_type() == BusType::Slack {
            return;
        }
        self.power = Power::new(self.power().active / s_base, self.power().reactive / s_base);
        self.voltage = Voltage::new(self.voltage().amplitude / v_base, self.voltage().angle);
    }
    pub fn reset_pu(&mut self, v_base: f64, s_base: f64) {
        if self.bus_type() == BusType::Slack {
            return;
        }
        self.power = Power::new(self.power().active * s_base, self.power().reactive * s_base);
        self.voltage = Voltage::new(self.voltage().amplitude * v_base, self.voltage().angle);
    }

    pub fn energy_type(&self) -> PowerType {
        self.energy_type
    }

    pub fn slack() -> Self {
        BusNode {
            id: Self::next_id(),
            active: true,
            bus_type: BusType::Slack,
            voltage: Voltage::new(1.0, 0.001),
            power: Power::new(0.10, 0.001),
            energy_type: PowerType::Storage,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
    #[allow(dead_code)]
    pub fn active(&self) -> bool {
        self.active
    }
    #[allow(dead_code)]
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn voltage(&self) -> Voltage {
        self.voltage
    }

    pub fn set_voltage(&mut self, voltage: Voltage) {
        self.voltage = voltage;
    }

    pub fn power(&self) -> Power {
        self.power
    }
    #[allow(dead_code)]
    pub fn set_power(&mut self, power: Power) {
        self.power = power;
    }

    pub fn bus_type(&self) -> BusType {
        self.bus_type
    }

    pub fn is_slack(&self) -> bool {
        matches!(self.bus_type, BusType::Slack)
    }

    pub fn power_type(&self) -> PowerType {
        self.energy_type
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bus_node() {
        let node = BusNode::generator(1.0, 1.0, PowerType::Fossil);
        assert!(node.active());
        assert_eq!(node.bus_type(), BusType::Generator);
        assert_eq!(node.power(), Power::new(1.0, 0.001));
        assert_eq!(node.voltage(), Voltage::new(1.0, 0.001));
    }
    #[test]
    fn test_bus_node_generator() {
        let mut node = BusNode::generator(1.0, 1.0, PowerType::Fossil);
        assert!(node.active());
        assert_eq!(node.bus_type(), BusType::Generator);
        assert_eq!(node.power(), Power::new(1.0, 0.001));
        assert_eq!(node.voltage(), Voltage::new(1.0, 0.001));
        assert_eq!(node.power_type(), PowerType::Fossil);
        node.set_power(Power::new(2.0, 0.001));
        assert_eq!(node.power(), Power::new(2.0, 0.001));
        node.set_active(false);
        assert!(!node.active());
        node.set_active(true);
        assert!(node.active());
        assert!(!node.is_slack());
    }
}
