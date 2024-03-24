use component_library::global::TemperatureComponent;
use noise::{NoiseFn, Perlin};
use simulator_communication::{ComponentsInfo, Graph, Server, Simulator};
use std::{env, net::SocketAddr, process::ExitCode, time::Duration};
use tracing::{error, info};

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    _ = dotenvy::dotenv();
    tracing_subscriber::fmt().init();

    let listen_addr = match env::var("TEMPERATURE_SIMULATOR_ADDR")
        .unwrap_or("127.0.0.1:8101".to_string())
        .parse::<SocketAddr>()
    {
        Ok(v) => v,
        Err(err) => {
            error!("Could not parse bind address: {err}.");
            return ExitCode::FAILURE;
        }
    };

    let server = Server::<TemperatureSimulator>::new();

    // Manager address
    let connector_addr =
        env::var("SIMULATOR_CONNECTOR_ADDR").unwrap_or("http://127.0.0.1:8099".to_string());

    info!("Starting temperature simulator server on `{listen_addr}`.");
    if let Err(err) = server.start(listen_addr, connector_addr).await {
        error!("Server return an error: {err}.");
        return ExitCode::FAILURE;
    }

    info!("Server exited successfully.");
    ExitCode::SUCCESS
}

pub struct TemperatureSimulator {
    // How much time advances per frame.
    delta_time: Duration,
    perlin: Perlin,
}

impl Simulator for TemperatureSimulator {
    fn get_component_info() -> ComponentsInfo {
        ComponentsInfo::new()
            .add_required_component::<TemperatureComponent>()
            .add_output_component::<TemperatureComponent>()
    }

    fn new(delta_time: std::time::Duration, _graph: Graph) -> Self {
        let perlin = Perlin::default();
        Self { delta_time, perlin }
    }

    fn do_timestep(&mut self, mut graph: Graph) -> Graph {
        if let Some(temperature) = graph.get_global_component_mut::<TemperatureComponent>() {
            //This is a temporary temperature calculation, more discussion will be needed
            const MONTH_IN_SECS: f64 = 2_592_000.0; // Approximate a month as 30 days
            let time_factor =
                (self.delta_time.as_secs_f64().ln() - 1f64.ln()) / (MONTH_IN_SECS.ln() - 1f64.ln());
            let max_fluctuation = 1.0 + time_factor * 0.5 * (30.0 - 1.0);
            let noise = self.perlin.get([self.delta_time.as_secs_f64(), 0.0]);
            let current_temp = temperature.current_temp;
            let delta_temp = noise * max_fluctuation;
            temperature.current_temp = (current_temp + delta_temp).clamp(-5.0, 30.0);
        } else {
            error!("No temperature component was found.");
        };
        graph
    }
}
