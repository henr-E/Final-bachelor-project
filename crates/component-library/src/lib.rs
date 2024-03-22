pub mod global {
    pub use chrono;
    use chrono::{DateTime, NaiveDateTime};
    use simulator_communication::{
        component::ComponentPiece, component_structure::ComponentStructure, Component,
        ComponentPiece, Value,
    };
    /// The current time of a frame in the simulation.
    ///
    /// Accurate up to the millisecond.
    #[derive(Component, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    #[component(name = "global_time", ty = "global")]
    pub struct TimeComponent(pub NaiveDateTime);

    /// Used to automate the [`ComponentPiece`] implementation for [`TimeComponent`] and
    /// avoid having to write it out manually, while keeping a nice API ([`NaiveDateTime`]).
    #[derive(ComponentPiece)]
    struct TimeComponentImpl {
        unix_timestamp_millis: i64,
    }

    impl ComponentPiece for TimeComponent {
        fn get_structure() -> ComponentStructure {
            TimeComponentImpl::get_structure()
        }

        fn from_value(value: Value) -> Option<Self> {
            TimeComponentImpl::from_value(value)
                .and_then(|v| DateTime::from_timestamp_millis(v.unix_timestamp_millis))
                .as_ref()
                .map(DateTime::naive_utc)
                .map(TimeComponent)
        }

        fn to_value(&self) -> Value {
            TimeComponentImpl::to_value(&TimeComponentImpl {
                unix_timestamp_millis: self.0.and_utc().timestamp_millis(),
            })
        }
    }

    #[derive(ComponentPiece, Component)]
    #[component(name = "global_temperature", ty = "global")]
    pub struct TemperatureComponent {
        //This is the current temperature in degrees (celsius)
        pub current_temp: f64,
    }
}
pub mod energy {
    use simulator_communication_macros::Component;
    use simulator_communication_macros::ComponentPiece;

    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_consumer_node", ty = "node")]
    pub struct EnergyConsumerNode {
        /// Demand in megawattsper hour (MWh)
        pub demand: f64,
        /// Voltage used in V
        pub voltage: f64,
        /// measure of how demand responds to change in price
        pub demand_elasticity: f64,
    }

    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_storage_node", ty = "node")]
    pub struct EnergySlackNode {
        /// Voltage in V
        pub voltage: f64,
    }

    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_producer_node", ty = "node")]
    pub struct EnergyProducerNode {
        /// Capacity in megawatts
        pub capacity: f64,
        /// Energy produced in MWH
        pub energy_production: f64,
        /// Volatge produced in V
        pub voltage: f64,
        /// e.g "fossil", "nuclear", "renewable"
        pub power_type: String,
    }

    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_transmission_edge", ty = "edge")]
    pub struct EnergyTransmissionEdge {
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
        /// Conductance (G) in siemens (S), indicating the real part of admittance facilitating power flow.
        pub conductance: f64,
        /// Susceptance (B) in siemens (S), indicating the imaginary part of admittance storing and releasing energy.
        pub susceptance: f64,
    }
}
