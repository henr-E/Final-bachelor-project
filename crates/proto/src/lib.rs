pub mod simulator {
    pub use proto_simulator::{simulator_client::*, simulator_server::*, *};

    mod proto_simulator {
        tonic::include_proto!("simulator");
    }
}
