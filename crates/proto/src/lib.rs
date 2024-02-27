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

pub mod simulator {
    pub use proto_simulator::{simulator_client::*, simulator_server::*, *};

    mod proto_simulator {
        tonic::include_proto!("simulator");
    }
}
