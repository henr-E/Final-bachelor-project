use component_library::global::TimeComponent;
use simulator_communication::{
    simulator::SimulationError, ComponentsInfo, Graph, Server, Simulator,
};
use std::{env, net::SocketAddr, process::ExitCode};
use tracing::{debug, error, info};

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

    // Manager address
    let connector_addr =
        env::var("SIMULATOR_CONNECTOR_ADDR").unwrap_or("http://127.0.0.1:8099".to_string());

    let server = Server::<TimeSimulator>::new();

    info!("Starting time simulator server on `{listen_addr}`.");
    if let Err(err) = server
        .start(listen_addr, connector_addr, "time simulator")
        .await
    {
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

    async fn new(delta_time: std::time::Duration, _graph: Graph) -> Result<Self, SimulationError> {
        info!("Started new simulation.");
        Ok(Self { delta_time })
    }

    async fn do_timestep(&mut self, mut graph: Graph) -> Result<Graph, SimulationError> {
        debug!("Executing timestep.");

        let Some(time) = graph.get_global_component_mut::<TimeComponent>() else {
            return Err(SimulationError::InvalidInput(
                "missing time component found".to_string(),
            ));
        };
        time.0 += self.delta_time;
        Ok(graph)
    }
}
