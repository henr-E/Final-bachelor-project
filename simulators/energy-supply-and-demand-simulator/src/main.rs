use component_library::energy::{EnergyConsumerNode, EnergyProducerNode};
use rand::prelude::*;
use simulator_communication::{ComponentsInfo, Graph, Server, Simulator};
use std::{env, net::SocketAddr, process::ExitCode, time::Duration};
use tracing::{error, info};

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().init();

    let listen_addr = match env::var("ENERGY_SUPPLY_AND_DEMAND_SIMULATOR_ADDR")
        .unwrap_or("0.0.0.0:8101".to_string())
        .parse::<SocketAddr>()
    {
        Ok(v) => v,
        Err(err) => {
            error!("Could not parse bind address: {err}.");
            return ExitCode::FAILURE;
        }
    };

    let server = Server::<EnergySupplyAndDemandSimulator>::new();

    info!("Starting supply and demand simulator server on `{listen_addr}`.");
    if let Err(err) = server.listen_on(listen_addr).await {
        error!("Server return an error: {err}.");
        return ExitCode::FAILURE;
    }

    info!("Server exited successfully.");
    ExitCode::SUCCESS
}

///Simulator that gives a random demand and supply to a consumer and producer node respectively every timestep
pub struct EnergySupplyAndDemandSimulator {
    delta_time: Duration,
}

impl Simulator for EnergySupplyAndDemandSimulator {
    fn get_component_info() -> ComponentsInfo {
        ComponentsInfo::new()
            .add_required_component::<EnergyConsumerNode>()
            .add_required_component::<EnergyProducerNode>()
            .add_output_component::<EnergyConsumerNode>()
            .add_output_component::<EnergyProducerNode>()
    }

    fn new(delta_time: std::time::Duration, _graph: Graph) -> Self {
        Self {
            // How much time advances per frame.
            delta_time,
        }
    }

    fn do_timestep(&mut self, mut graph: Graph) -> Graph {
        for (_, _, component) in graph.get_all_nodes_mut::<EnergyConsumerNode>().unwrap() {
            let current_demand = component.demand;
            let delta_demand =
                rand::thread_rng().gen_range(-25.0..25.0) * self.delta_time.as_secs() as f64;
            component.demand = (current_demand + delta_demand).clamp(0.0, 500.0)
        }

        for (_, _, component) in graph.get_all_nodes_mut::<EnergyProducerNode>().unwrap() {
            let current_capacity = component.capacity;
            let delta_capacity =
                rand::thread_rng().gen_range(-50.0..50.0) * self.delta_time.as_secs() as f64;
            component.capacity = (current_capacity + delta_capacity).clamp(1000.0, 2000.0)
        }
        graph
    }
}
