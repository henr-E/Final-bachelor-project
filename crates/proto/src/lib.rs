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

    impl ParseResult {
        pub fn new_ok() -> Self {
            Self {
                failures: Vec::new(),
            }
        }

        pub fn ok(&self) -> bool {
            self.failures.is_empty()
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

    pub mod simulator_connection {
        pub use proto_simulator_connection::{
            simulator_connection_client::*, simulator_connection_server::*, *,
        };

        mod proto_simulator_connection {
            tonic::include_proto!("simulation.simulator_connection");
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
    pub use proto_sensor_crud::{sensor_crud_service_client::*, sensor_crud_service_server::*, *};

    mod auth_proto {
        tonic::include_proto!("authentication.auth");
    }

    mod proto_frontend_simulation {
        tonic::include_proto!("simulation.frontend");
    }
    pub mod proto_twin {
        tonic::include_proto!("twin");
    }
    mod proto_sensor_crud {
        tonic::include_proto!("sensor.crud");
    }

    impl CrudFailure {
        pub fn new(reasons: impl IntoIterator<Item = CrudFailureReason>) -> Self {
            Self {
                reasons: reasons.into_iter().map(i32::from).collect(),
            }
        }

        pub fn new_single(reason: CrudFailureReason) -> Self {
            Self {
                reasons: vec![reason.into()],
            }
        }

        pub fn new_database_error() -> Self {
            Self::new_single(CrudFailureReason::DatabaseInsertionError)
        }

        pub fn new_uuid_format_error() -> Self {
            Self::new_single(CrudFailureReason::UuidFormatError)
        }

        pub fn new_uuid_not_found_error() -> Self {
            Self::new_single(CrudFailureReason::UuidNotPresentError)
        }
    }

    impl BigInt {
        pub fn one() -> Self {
            Self {
                sign: false,
                integer: vec![1],
                exponent: 0,
            }
        }
    }

    impl CreateSensorResponse {
        pub fn uuid(uuid: uuid::Uuid) -> Self {
            use self::create_sensor_response::Result;
            Self {
                result: Some(Result::Uuid(uuid.to_string())),
            }
        }

        pub fn failures(failures: CrudFailure) -> Self {
            use self::create_sensor_response::Result;
            Self {
                result: Some(Result::Failures(failures)),
            }
        }
    }

    impl ReadSensorResponse {
        pub fn sensor(sensor: self::Sensor) -> Self {
            use self::read_sensor_response::Result;
            Self {
                result: Some(Result::Sensor(sensor)),
            }
        }

        pub fn failures(failures: CrudFailure) -> Self {
            use self::read_sensor_response::Result;
            Self {
                result: Some(Result::Failures(failures)),
            }
        }
    }

    impl UpdateSensorResponse {
        pub fn success(success: bool) -> Self {
            use self::update_sensor_response::Result;
            Self {
                result: Some(Result::Success(success)),
            }
        }

        pub fn failures(failures: CrudFailure) -> Self {
            use self::update_sensor_response::Result;
            Self {
                result: Some(Result::Failures(failures)),
            }
        }
    }

    impl DeleteSensorResponse {
        pub fn success(success: bool) -> Self {
            use self::delete_sensor_response::Result;
            Self {
                result: Some(Result::Success(success)),
            }
        }

        pub fn failures(failures: CrudFailure) -> Self {
            use self::delete_sensor_response::Result;
            Self {
                result: Some(Result::Failures(failures)),
            }
        }
    }

    impl From<CreateSensorResponse> for Result<tonic::Response<CreateSensorResponse>, tonic::Status> {
        fn from(value: CreateSensorResponse) -> Self {
            Ok(tonic::Response::new(value))
        }
    }

    impl From<ReadSensorResponse> for Result<tonic::Response<ReadSensorResponse>, tonic::Status> {
        fn from(value: ReadSensorResponse) -> Self {
            Ok(tonic::Response::new(value))
        }
    }

    impl From<UpdateSensorResponse> for Result<tonic::Response<UpdateSensorResponse>, tonic::Status> {
        fn from(value: UpdateSensorResponse) -> Self {
            Ok(tonic::Response::new(value))
        }
    }

    impl From<DeleteSensorResponse> for Result<tonic::Response<DeleteSensorResponse>, tonic::Status> {
        fn from(value: DeleteSensorResponse) -> Self {
            Ok(tonic::Response::new(value))
        }
    }
}
