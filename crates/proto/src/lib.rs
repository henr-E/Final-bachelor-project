pub mod sensor_data_ingest {
    pub use proto_sensor_data_ingest::{
        data_ingest_service_client::*, data_ingest_service_server::*, *,
    };

    mod proto_sensor_data_ingest {
        tonic::include_proto!("sensor.data_ingest");
    }

    impl ParseFailure {
        /// Create a [`ParseFailure`] given a `reason` and details as a string.
        pub fn new_string_detail(reason: ParseFailureReason, details: impl Into<String>) -> Self {
            Self {
                reason: reason.into(),
                details: Some(prost_value::from_string(details)),
            }
        }

        /// Create a [`ParseFailure`] given a `reason` without `details`.
        pub fn new_empty(reason: ParseFailureReason) -> Self {
            Self {
                reason: reason.into(),
                details: None,
            }
        }
    }

    // Convert a single [`ParseFailure`] into a [`ParseResult`].
    impl From<ParseFailure> for ParseResult {
        fn from(value: ParseFailure) -> Self {
            ParseResult {
                failures: vec![value],
            }
        }
    }

    // Convert a list of [`ParseFailure`] into a [`ParseResult`].
    impl<V> From<V> for ParseResult
    where
        V: Into<Vec<ParseFailure>>,
    {
        fn from(value: V) -> Self {
            ParseResult {
                failures: value.into(),
            }
        }
    }
}

pub mod simulation {
    pub use proto_simulation::*;

    mod proto_simulation {
        tonic::include_proto!("simulation.simulation");
    }

    pub mod simulator {
        use crate::simulation;
        pub use proto_simulator::{simulator_client::*, simulator_server::*, *};

        mod proto_simulator {
            tonic::include_proto!("simulation.simulator");
        }
    }

    pub mod simulation_manager {
        use crate::simulation;
        pub use proto_simulation_manager::{
            simulation_manager_client::*, simulation_manager_server::*, *,
        };

        mod proto_simulation_manager {
            tonic::include_proto!("simulation.simulation_manager");
        }
    }
}

pub mod frontend {
    use crate::simulation;
    use crate::simulation::simulation_manager;

    pub use proto_frontend_simulation::{
        simulation_interface_service_client::*, simulation_interface_service_server::*, *,
    };

    pub use proto_twin::twin_service_client::TwinServiceClient;
    pub use proto_twin::twin_service_server::{TwinService, TwinServiceServer};

    pub use auth_proto::{authentication_service_client::*, authentication_service_server::*, *};
    pub use proto_sensor_crud::{sensor_crud_service_client::*, sensor_crud_service_server::*};

    mod auth_proto {
        tonic::include_proto!("authentication.auth");
    }

    mod proto_frontend_simulation {
        tonic::include_proto!("simulation.frontend");
    }
    pub mod proto_twin {
        tonic::include_proto!("twin");
    }
    pub mod proto_sensor_crud {
        tonic::include_proto!("sensor.crud");
    }
}
