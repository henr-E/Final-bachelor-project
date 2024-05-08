use crate::units::{power::Power, voltage::Voltage};

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

/// Represents a node in an electrical power system bus network.
#[derive(Clone, Debug, Copy)]
pub struct BusNode {
    /// A unique identifier for the node within the bus network.
    id: usize,

    /// Indicates whether the bus node is currently active. Inactive nodes may be ignored in simulations or calculations.
    active: bool,

    /// Specifies the type of the bus node (PV, PQ, or Slack) which affects power flow calculations.
    bus_type: BusType,

    /// Power value at the node, encapsulating both generation and consumption characteristics.
    power: Power,

    /// Current voltage at the node, represented by magnitude and phase angle.
    voltage: Voltage,

    /// The type of power (e.g., solar, wind, fossil) at this node, reflecting the energy source or consumption nature.
    energy_type: PowerType,
}

impl BusNode {
    /// Creates a new generator bus node.
    ///
    /// # Parameters
    /// - `active_power_pu`: Active power in per unit.
    /// - `voltage_magnitude_pu`: Voltage magnitude in per unit.
    /// - `power_type`: Type of power source, specified by `PowerType`.
    ///
    /// # Returns
    /// A new `BusNode` instance configured as a generator with specified parameters.
    pub fn generator(
        id: usize,
        active_power_pu: f64,
        voltage_magnitude_pu: f64,
        power_type: PowerType,
    ) -> Self {
        BusNode {
            id,
            active: true,
            bus_type: BusType::Generator,
            power: Power::new(active_power_pu, 0.001),
            voltage: Voltage::new(voltage_magnitude_pu, 0.001),
            energy_type: power_type,
        }
    }

    /// Creates a new load bus node.
    ///
    /// # Parameters
    /// - `active_power_pu`: Active power in per unit.
    /// - `reactive_power_pu`: Reactive power in per unit.
    ///
    /// # Returns
    /// A new `BusNode` instance configured as a load with specified power characteristics.
    pub fn load(id: usize, active_power_pu: f64, reactive_power_pu: f64) -> Self {
        BusNode {
            id,
            active: true,
            bus_type: BusType::Load,
            power: Power::new(active_power_pu.abs(), reactive_power_pu),
            voltage: Voltage::new(1.0, 0.001),
            energy_type: PowerType::Load,
        }
    }

    /// Creates a new slack bus node.
    ///
    /// # Returns
    /// A new `BusNode` instance configured as a slack node with default voltage and power.
    pub fn slack(id: usize) -> Self {
        BusNode {
            id,
            active: true,
            bus_type: BusType::Slack,
            voltage: Voltage::new(1.0, 0.001),
            power: Power::new(0.10, 0.001),
            energy_type: PowerType::Storage,
        }
    }

    /// Adjusts the node's power and voltage to specified base values.
    ///
    /// This method scales the node's power and voltage to new per unit (PU) values based on the given bases.
    /// This function does nothing if the node is of `BusType::Slack`.
    ///
    /// # Parameters
    /// - `v_base`: The new voltage base value.
    /// - `s_base`: The new power base value.
    pub fn set_pu(&mut self, v_base: f64, s_base: f64) {
        if self.bus_type() == BusType::Slack {
            return;
        }
        self.power = Power::new(self.power().active / s_base, self.power().reactive / s_base);
        self.voltage = Voltage::new(self.voltage().amplitude / v_base, self.voltage().angle);
    }

    /// Resets the node's power and voltage from specified base values.
    ///
    /// This method scales the node's power and voltage from per unit (PU) values back to absolute values based on the given bases.
    /// This function does nothing if the node is of `BusType::Slack`.
    ///
    /// # Parameters
    /// - `v_base`: The original voltage base value to scale back to.
    /// - `s_base`: The original power base value to scale back to.
    pub fn reset_pu(&mut self, v_base: f64, s_base: f64) {
        if self.bus_type() == BusType::Slack {
            return;
        }
        self.power = Power::new(self.power().active * s_base, self.power().reactive * s_base);
        self.voltage = Voltage::new(self.voltage().amplitude * v_base, self.voltage().angle);
    }

    /// Returns the energy type of the bus node.
    ///
    /// # Returns
    /// The `PowerType` describing the energy source or load type of this node.
    pub fn energy_type(&self) -> PowerType {
        self.energy_type
    }

    /// Returns the unique identifier of the bus node.
    ///
    /// # Returns
    /// The node's ID as a `usize`.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Returns the active status of the bus node.
    ///
    /// # Returns
    /// A boolean indicating whether the node is active (`true`) or not (`false`).
    ///
    /// NOTE: Currently unused, can be interesting for future implementations
    #[allow(dead_code)]
    pub fn active(&self) -> bool {
        self.active
    }

    /// Sets the active status of the bus node.
    ///
    /// # Parameters
    /// - `active`: Boolean value to set the node's active status.
    ///
    /// NOTE: Currently unused, can be interesting for future implementations
    #[allow(dead_code)]
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Returns the current voltage of the bus node.
    ///
    /// # Returns
    /// The node's voltage as a `Voltage`.
    pub fn voltage(&self) -> Voltage {
        self.voltage
    }

    /// Sets the voltage of the bus node.
    ///
    /// # Parameters
    /// - `voltage`: A `Voltage` to set for the node.
    pub fn set_voltage(&mut self, voltage: Voltage) {
        self.voltage = voltage;
    }

    /// Returns the power characteristics of the bus node.
    ///
    /// # Returns
    /// The node's power as a `Power`.
    pub fn power(&self) -> Power {
        self.power
    }

    /// Sets the power characteristics of the bus node.
    ///
    /// # Parameters
    /// - `power`: A `Power` to set for the node.
    #[allow(dead_code)]
    pub fn set_power(&mut self, power: Power) {
        self.power = power;
    }

    /// Returns the type of bus.
    ///
    /// # Returns
    /// The bus type as `BusType`.
    pub fn bus_type(&self) -> BusType {
        self.bus_type
    }

    /// Checks if the bus node is a slack bus.
    ///
    /// # Returns
    /// `true` if the node is a slack bus, otherwise `false`.
    pub fn is_slack(&self) -> bool {
        matches!(self.bus_type, BusType::Slack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bus_node() {
        let node = BusNode::generator(0, 1.0, 1.0, PowerType::Fossil);
        assert!(node.active());
        assert_eq!(node.bus_type(), BusType::Generator);
        assert_eq!(node.power(), Power::new(1.0, 0.001));
        assert_eq!(node.voltage(), Voltage::new(1.0, 0.001));
    }
    #[test]
    fn test_bus_node_generator() {
        let mut node = BusNode::generator(0, 1.0, 1.0, PowerType::Fossil);
        assert!(node.active());
        assert_eq!(node.bus_type(), BusType::Generator);
        assert_eq!(node.power(), Power::new(1.0, 0.001));
        assert_eq!(node.voltage(), Voltage::new(1.0, 0.001));
        node.set_power(Power::new(2.0, 0.001));
        assert_eq!(node.power(), Power::new(2.0, 0.001));
        node.set_active(false);
        assert!(!node.active());
        node.set_active(true);
        assert!(node.active());
        assert!(!node.is_slack());
    }
}
