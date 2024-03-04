use std::{env, net::SocketAddr, process::ExitCode};

use component_library::TimeComponent;
use simulator_communication::{ComponentsInfo, Graph, Server, Simulator};
use tracing::{error, info};

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    _ = dotenvy::dotenv();
    tracing_subscriber::fmt().init();

    let listen_addr = match env::var("TIME_SIMULATOR_ADDR")
        .unwrap_or("127.0.0.1:8101".to_string())
        .parse::<SocketAddr>()
    {
        Ok(v) => v,
        Err(err) => {
            error!("Could not parse bind address: {err}.");
            return ExitCode::FAILURE;
        }
    };

    let server = Server::<TimeSimulator>::new();

    info!("Starting time simulator server on `{listen_addr}`.");
    if let Err(err) = server.listen_on(listen_addr).await {
        error!("Server return an error: {err}.");
        return ExitCode::FAILURE;
    }

    info!("Server exited successfully.");
    ExitCode::SUCCESS
}

/// Advances time in the simulation.
pub struct TimeSimulator {
    /// How much time advances per frame.
    delta_time: std::time::Duration,
}

impl Simulator for TimeSimulator {
    fn get_component_info() -> ComponentsInfo {
        ComponentsInfo::new()
            .add_required_component::<TimeComponent>()
            .add_output_component::<TimeComponent>()
    }

    fn new(delta_time: std::time::Duration, _graph: Graph) -> Self {
        info!("Started new simulation.");
        Self { delta_time }
    }

    fn do_timestep(&mut self, mut graph: Graph) -> Graph {
        info!("Executing timestep.");

        let Some(time) = graph.get_global_component_mut::<TimeComponent>() else {
            error!("No time component was found.");
            return graph;
        };
        time.0 += self.delta_time;

        graph
    }
}
