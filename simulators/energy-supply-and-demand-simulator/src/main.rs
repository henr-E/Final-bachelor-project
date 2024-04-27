use component_library::energy::{
    Bases, ConsumerNode, PowerType, ProducerNode, ProductionOverview, TransmissionEdge,
};
use component_library::global::chrono::{NaiveDateTime, Timelike};
use component_library::global::{
    IlluminanceComponent, SupplyAndDemandAnalytics, TemperatureComponent, TimeComponent,
};
use rand::prelude::*;
use simulator_communication::{ComponentsInfo, Graph, Server, Simulator};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::{env, net::SocketAddr, process::ExitCode, time::Duration};
use tracing::{debug, error, info};

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

    // Manager address
    let connector_addr =
        env::var("SIMULATOR_CONNECTOR_ADDR").unwrap_or("http://127.0.0.1:8099".to_string());

    let server = Server::<EnergySupplyAndDemandSimulator>::new();

    info!("Starting supply and demand simulator server on `{listen_addr}`.");
    if let Err(err) = server.start(listen_addr, connector_addr).await {
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
            .add_optional_component::<SupplyAndDemandAnalytics>()
            .add_required_component::<ConsumerNode>()
            .add_required_component::<ProducerNode>()
            .add_required_component::<TransmissionEdge>()
            .add_required_component::<Bases>()
            .add_optional_component::<TimeComponent>()
            .add_optional_component::<TemperatureComponent>()
            .add_output_component::<ConsumerNode>()
            .add_output_component::<ProducerNode>()
            .add_output_component::<TransmissionEdge>()
            .add_output_component::<SupplyAndDemandAnalytics>()
    }

    fn new(delta_time: std::time::Duration, _graph: Graph) -> Self {
        Self {
            // How much time advances per frame.
            delta_time,
        }
    }

    fn do_timestep(&mut self, mut graph: Graph) -> Graph {
        let current_temp = match graph.get_global_component_mut::<TemperatureComponent>() {
            Some(temperature) => temperature.current_temp,
            None => 17.5,
        };

        let _current_illuminance = match graph.get_global_component_mut::<IlluminanceComponent>() {
            Some(illuminance) => illuminance.current_illuminance,
            None => 0.0,
        };

        let mut binding = TimeComponent(NaiveDateTime::default());
        let current_time = match graph.get_global_component_mut::<TimeComponent>() {
            Some(timecomponent) => timecomponent,
            None => &mut binding,
        };
        let time_effect = if current_time.0.hour() >= 6 && current_time.0.hour() <= 18 {
            1.0
        } else {
            0.5
        };
        let temp_effect = (current_temp - 17.5).abs();
        let target_demand = 100.0 + temp_effect * 10.0 * time_effect;

        for (_, _, component) in graph.get_all_nodes_mut::<ConsumerNode>().unwrap() {
            let current_demand = component.active_power;
            let adjustment = rand::thread_rng().gen_range(-0.01..0.05);
            let delta_demand =
                (target_demand - current_demand) * adjustment * self.delta_time.as_secs() as f64;
            let mut rng = rand::thread_rng();
            let direction_factor = if rng.gen_bool(0.1) { -1.0 } else { 1.0 };
            component.active_power =
                (current_demand + delta_demand * direction_factor).clamp(0.0, 500.0)
        }

        let current_illuminance = match graph.get_global_component::<IlluminanceComponent>() {
            Some(item) => item.current_illuminance,
            None => 0.0,
        };

        let mut rng = rand::thread_rng();
        // TODO: get capacity.
        let capacity = 1.0;
        // TODO: get wind speed
        // meters per second
        let wind_speed: f64 = 1.0;

        for (_, _, component) in graph.get_all_nodes_mut::<ProducerNode>().unwrap() {
            match &component.power_type {
                PowerType::Solar => {
                    component.active_power =
                        capacity * current_illuminance * self.delta_time.as_secs_f64() / 3600.0
                            * 0.2;
                }
                PowerType::Wind => {
                    // https://thundersaidenergy.com/downloads/wind-power-impacts-of-larger-turbines/
                    // c_p = efficiency percentage. Theoretical maximum * small error factor.
                    let efficiency = rng.gen_range(0.98..=1.00);
                    let c_p = 0.593 * efficiency;
                    // rho = air_dencity, kg / m^3. source: wikipedia
                    let rho = 1.204;
                    // lenght of a single blade in meters. range of average lenghts.
                    let blade_lenght: f64 = rng.gen_range(35.0..45.0);
                    component.active_power =
                        0.5 * c_p * rho * PI * blade_lenght.powi(2) * wind_speed.powi(3);
                }
                PowerType::Nuclear => {
                    let efficiency = rng.gen_range(0.99..=1.00);
                    component.active_power =
                        capacity * efficiency * self.delta_time.as_secs_f64() / 3600.0;
                }
                _ => {
                    let current_capacity = component.active_power;
                    let delta_capacity = rand::thread_rng().gen_range(-50.0..50.0)
                        * self.delta_time.as_secs() as f64;
                    component.active_power =
                        (current_capacity + delta_capacity).clamp(1000.0, 2000.0);
                }
            }
        }

        let mut num_consumer_nodes = 0;
        let mut num_producer_nodes = 0;
        let mut total_demand = 0.0;
        let mut total_capacity = 0.0;

        let components = graph.get_all_nodes::<ConsumerNode>().into_iter().flatten();
        for (_, _, component) in components {
            num_consumer_nodes += 1;
            total_demand += component.active_power
        }

        let num_edges = graph
            .get_all_nodes::<TransmissionEdge>()
            .into_iter()
            .flatten()
            .count();

        let mut power_type_percentages: HashMap<PowerType, f64> = HashMap::new();

        let components = graph.get_all_nodes::<ProducerNode>().into_iter().flatten();
        for (_, _, component) in components {
            num_producer_nodes += 1;
            total_capacity += component.active_power;
            let counter = power_type_percentages
                .entry(component.power_type)
                .or_insert(0.0);
            *counter += component.active_power;
        }

        for (_, percentage) in power_type_percentages.iter_mut() {
            *percentage /= total_capacity
        }

        if let Some(analytics) = graph.get_global_component_mut::<SupplyAndDemandAnalytics>() {
            let mut vec_overview = Vec::<ProductionOverview>::new();
            for (power_type, percentage) in power_type_percentages {
                vec_overview.push(ProductionOverview {
                    power_type,
                    percentage,
                })
            }
            analytics.energy_production_overview = vec_overview;
            analytics.consumer_nodes_count = num_consumer_nodes;
            analytics.producer_nodes_count = num_producer_nodes;
            analytics.transmission_edges_count = num_edges as i32;
            analytics.total_demand = total_demand;
            analytics.total_capacity = total_capacity;
            analytics.utilization = total_demand / total_capacity;
        } else {
            debug!("No analytics component found");
        }

        graph
    }
}
