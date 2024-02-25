pub mod energy {
    use simulator_communication_macros::Component;
    use simulator_communication_macros::ComponentPiece;

    /// Common data found in every type of energy node.
    #[derive(ComponentPiece)]
    pub struct GenericEnergyNode {
        is_active: bool,
        current_power: f64,
    }

    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_consumer_node", ty = "node")]
    pub struct EnergyConsumerNode {
        pub generic_node: GenericEnergyNode,
        // Other fields specific to ConsumerNode
        /// Demand in megawattsper hour (MWh)
        pub demand: f64,
        /// measure of how demand responds to change in price            
        pub demand_elasticity: f64,
    }

    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_storage_node", ty = "node")]
    pub struct EnergyStorageNode {
        pub generic_node: GenericEnergyNode,
        /// Capacity in megawatts
        pub capacity: f64,
        /// Current energy content in MWh           
        pub charge_state: f64,
        /// Maximum charge rate in MW       
        pub max_charge_rate: f64,
        /// Maximum discharge rate in MW
        pub max_discharge_rate: f64,
        /// Efficiency factor (0 to 1), representing energy loss during charge/discharge
        pub efficiency: f64,
    }

    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_producer_node", ty = "node")]
    pub struct EnergyProducerNode {
        pub generic_node: GenericEnergyNode,
        // Other fields specific to ProducerNode
        /// Capacity in megawatts
        pub capacity: f64,
        /// Energy produced in MWH          
        pub energy_production: f64,
        /// e.g "fossil", "nuclear", "renewable"
        pub power_type: String,
    }

    #[derive(ComponentPiece, Component)]
    #[component(name = "energy_transmission_node", ty = "node")]
    pub struct EnergyTransmissionNode {
        pub generic_node: GenericEnergyNode,
        // Other fields specific to TransmissionNode
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
    }
}
