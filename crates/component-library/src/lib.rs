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
    use simulator_communication::component::ComponentPiece;
    use simulator_communication::component_structure::ComponentStructure;
    use simulator_communication::prost_types::{value::Kind, Value};
    use simulator_communication::proto::ComponentPrimitive;
    use simulator_communication_macros::Component;
    use simulator_communication_macros::ComponentPiece;

    /// Per unit values are expressed relative to a chosen base value for each quantity.
    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_bases_node", ty = "node")]
    pub struct Bases {
        /// Base apparent power. Often expressed in volt-amperes (VA) or megavolt-amperes (MVA)
        pub s_base: f64,
        // Base voltage. Often expressed in volts (V) or kilovolts (kV)
        pub v_base: f64,
        // Base active power. Often expressed in watts (W) or megawatts (MW)
        pub p_base: f64,
    }

    /// Represents a node in the energy system that's demanding power, with properties like voltage amplitude, voltage angle, active power, and reactive power.
    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_load_node", ty = "node")]
    pub struct LoadNode {
        /// Voltage amplitude in p.u
        pub voltage_amplitude: f64,
        /// Voltage angle in radians
        pub voltage_angle: f64,
        /// Active power in p.u
        pub active_power: f64,
        /// Reactive power in p.u
        pub reactive_power: f64,
    }
    /// The slack is a mathematical concept used for load-flow analysis. Each network has one slack node. This node serves as a reference point with known voltage magnitude and angle, facilitating power flow analysis and ensuring the balance of power generation and consumption within the system.
    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_slack_node", ty = "node")]
    pub struct SlackNode {
        /// Voltage amplitude in p.u
        pub voltage_amplitude: f64,
        /// Voltage angle in radians
        pub voltage_angle: f64,
        /// Active power in p.u
        pub active_power: f64,
        /// Reactive power in p.u
        pub reactive_power: f64,
    }
    /// Represents a node in the system that's generating power, with properties like voltage amplitude, voltage angle, active power, reactive power, and power type.
    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_generator_node", ty = "node")]
    pub struct GeneratorNode {
        /// Voltage amplitude in p.u
        pub voltage_amplitude: f64,
        /// Voltage angle in radians
        pub voltage_angle: f64,
        /// Active power in p.u
        pub active_power: f64,
        /// Type of power produced
        pub power_type: PowerType,
    }

    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_producer_node", ty = "node")]
    pub struct ProducerNode {
        /// Voltage amplitude in volts
        pub voltage_amplitude: f64,
        /// Voltage angle in radians
        pub voltage_angle: f64,
        /// Active power in watts
        pub active_power: f64,
        /// Reactive power in VAR
        pub reactive_power: f64,
        /// Type of power produced
        pub power_type: PowerType,
    }

    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_consumer_node", ty = "node")]
    pub struct ConsumerNode {
        /// Voltage amplitude in volts
        pub voltage_amplitude: f64,
        /// Voltage angle in radians
        pub voltage_angle: f64,
        /// Active power in watts
        pub active_power: f64,
        /// Reactive power in VAR
        pub reactive_power: f64,
    }

    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_transmission_edge", ty = "edge")]
    pub struct TransmissionEdge {
        pub resistance_per_meter: f64,
        /// Ohms per meter for AC lines
        pub reactance_per_meter: f64,
        /// Length of the transmission line in meters (m)
        pub length: f64,
        /// Type of the transmission line
        pub line_type: CableType,
        /// Current flowing through the transmission line in amperes
        pub current: f64,
    }
    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_load_flow", ty = "node")]
    pub struct EnergyLoadFlow {
        /// Total generators in the system
        pub total_generators: i32,
        /// Total slack nodes in the system
        pub total_slack_nodes: i32,
        /// Total load nodes in the system
        pub total_load_nodes: i32,
        /// Total transmission edges in the system
        pub total_transmission_edges: i32,
        /// total nodes in the system
        pub total_nodes: i32,
        /// Total incoming power to the system
        pub total_incoming_power: f64,
        /// Total outgoing power from the system
        pub total_outgoing_power: f64,
        /// vector of power type and percentage of how much power they account for
        pub energy_production_overview: Vec<ProductionOverview>,
    }
    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_prodction_overview", ty = "node")]
    pub struct ProductionOverview {
        pub power_type: PowerType,
        pub percentage: f64,
    }
    #[derive(Clone, Debug, Copy, PartialEq)]
    pub enum CableType {
        ACSRConductor,
        AACConductor,
        AAACConductor,
        XLPECable,
        PILCCable,
    }
    impl ComponentPiece for CableType {
        fn get_structure() -> ComponentStructure {
            ComponentStructure::Primitive(ComponentPrimitive::String.into())
        }

        fn from_value(value: Value) -> Option<Self> {
            match value.kind? {
                Kind::StringValue(s) => match s.as_str() {
                    "ACSR_Conductor" => Some(Self::ACSRConductor),
                    "AAC_Conductor" => Some(Self::AACConductor),
                    "AAAC_Conductor" => Some(Self::AAACConductor),
                    "XLPE_Conductor" => Some(Self::XLPECable),
                    "PILC_Conductor" => Some(Self::PILCCable),
                    _ => None,
                },
                _ => None,
            }
        }

        fn to_value(&self) -> Value {
            let s = match self {
                CableType::ACSRConductor => "ACSR_Conductor",
                CableType::AACConductor => "AAC_Conductor",
                CableType::AAACConductor => "AAAC_Conductor",
                CableType::XLPECable => "XLPE_Conductor",
                CableType::PILCCable => "PILC_Conductor",
            };
            Value {
                kind: Some(Kind::StringValue(s.to_owned())),
            }
        }
    }
    #[derive(Clone, Debug, Copy, PartialEq)]
    pub enum PowerType {
        Fossil,
        Renewable,
        Nuclear,
        Hydro,
        Solar,
        Wind,
        Battery,
        Storage,
    }
    impl ComponentPiece for PowerType {
        fn get_structure() -> ComponentStructure {
            ComponentStructure::Primitive(ComponentPrimitive::String.into())
        }

        fn from_value(value: Value) -> Option<Self> {
            match value.kind? {
                Kind::StringValue(s) => match s.as_str() {
                    "Fossil" => Some(Self::Fossil),
                    "Renewable" => Some(Self::Renewable),
                    "Nuclear" => Some(Self::Nuclear),
                    "Hydro" => Some(Self::Hydro),
                    "Solar" => Some(Self::Solar),
                    "Wind" => Some(Self::Wind),
                    "Battery" => Some(Self::Battery),
                    "Storage" => Some(Self::Storage),
                    _ => None,
                },
                _ => None,
            }
        }

        fn to_value(&self) -> Value {
            let s = match self {
                PowerType::Fossil => "Fossil",
                PowerType::Renewable => "Renewable",
                PowerType::Nuclear => "Nuclear",
                PowerType::Hydro => "Hydro",
                PowerType::Solar => "Solar",
                PowerType::Wind => "Wind",
                PowerType::Battery => "Battery",
                PowerType::Storage => "Storage",
            };
            Value {
                kind: Some(Kind::StringValue(s.to_owned())),
            }
        }
    }
}
