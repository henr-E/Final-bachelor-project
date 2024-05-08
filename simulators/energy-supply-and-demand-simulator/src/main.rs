use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::hash::{Hash, Hasher};
use std::{env, net::SocketAddr, process::ExitCode, time::Duration};

use futures::stream::StreamExt;
use itertools::izip;
use rand::prelude::*;
use tracing::{debug, error, info, Level};

use component_library::energy::{
    PowerType, ProductionOverview, SensorGeneratorNode, SensorLoadNode,
};
use component_library::global::{
    IrradianceComponent, SupplyAndDemandAnalytics, TimeComponent, WindSpeedComponent,
};
use component_library::Building;
use predictions::VAR;
use sensor_store::{Quantity, Sensor, SensorStore};
use simulator_communication::graph::NodeId;
use simulator_communication::simulator::SimulationError;
use simulator_communication::{ComponentsInfo, Graph, Server, Simulator};
use simulator_utilities::sensor::values_for_quantity_as_f64;

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let listen_addr = match env::var("ENERGY_SUPPLY_AND_DEMAND_SIMULATOR_ADDR")
        .unwrap_or("0.0.0.0:8102".to_string())
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
    if let Err(err) = server
        .start(
            listen_addr,
            connector_addr,
            "energy supply and demand simulator",
        )
        .await
    {
        error!("Server return an error: {err}.");
        return ExitCode::FAILURE;
    }

    info!("Server exited successfully.");
    ExitCode::SUCCESS
}

///Simulator that gives a random demand and supply to a consumer and producer node respectively every time step
pub struct EnergySupplyAndDemandSimulator {
    delta_time: Duration,
    /// Contains sensor data for energy consumption (in Watts) per building
    models: HashMap<i32, VAR>,
}

impl Simulator for EnergySupplyAndDemandSimulator {
    fn get_component_info() -> ComponentsInfo {
        info!("Getting components.");
        ComponentsInfo::new()
            .add_optional_component::<SupplyAndDemandAnalytics>()
            // time component is required to make predictions
            .add_required_component::<TimeComponent>()
            // components that the simulator will retrieve and return with adjusted values
            .add_required_component::<Building>()
            .add_required_component::<SensorLoadNode>()
            .add_required_component::<SensorGeneratorNode>()
            .add_optional_component::<WindSpeedComponent>()
            .add_optional_component::<IrradianceComponent>()
            .add_output_component::<SensorLoadNode>()
            .add_output_component::<SensorGeneratorNode>()
            .add_output_component::<SupplyAndDemandAnalytics>()
    }

    async fn new(delta_time: std::time::Duration, graph: Graph) -> Result<Self, SimulationError> {
        info!("Started new energy simulator.");

        let mut sensor_values_power_per_building = HashMap::new();
        // store sensor values per building for power consumption

        // try to connect with sensor database
        let sensor_store = match SensorStore::new().await {
            Ok(sensor_store) => sensor_store,
            Err(err) => {
                error!("Database connection failed: {}.", err);
                return Ok(Self {
                    delta_time,
                    models: HashMap::new(),
                });
            }
        };

        // loop over consumer nodes
        for (node_id, _, _) in graph.get_all_nodes::<SensorLoadNode>().unwrap() {
            let Some(building_component) = graph.get_node_component::<Building>(node_id) else {
                error!("Building component not found for consumer node with id {node_id:?}.");
                continue;
            };

            // get building id from consumer node
            let building_id = building_component.building_id;
            debug!("building_id = {building_id}.");
            // retrieve corresponding sensor
            match sensor_store.get_sensor_for_building(building_id).await {
                Ok(sensor) => {
                    // retrieve corresponding sensor power values (if any)
                    match values_for_quantity_as_f64(&sensor_store, &sensor, Quantity::Power).await
                    {
                        Err(_) => {}
                        Ok(sensor_data_power) => {
                            debug!(
                                "amt power data for building = {:?}, first = {:?}, last = {:?}.",
                                sensor_data_power.len(),
                                sensor_data_power.first(),
                                sensor_data_power.last()
                            );
                            if !sensor_data_power.is_empty() {
                                sensor_values_power_per_building
                                    .insert(building_id, sensor_data_power);
                            }
                        }
                    }
                }
                Err(err) => {
                    error!("Error retrieving the sensor for building: {}.", err);
                    continue;
                }
            }
        }
        info!("Retrieved all energy consumer nodes.");

        // Retrieve all global sensors
        let mut global_sensors: Vec<Sensor> = Vec::new();
        match sensor_store.get_all_global_sensors().await {
            Ok(sensor_stream) => {
                // Iterate over the stream of sensors
                sensor_stream
                    .for_each(|sensor_result| {
                        match sensor_result {
                            Ok(sensor) => global_sensors.push(sensor),
                            Err(err) => error!("Error retrieving sensor: {}.", err),
                        }
                        futures::future::ready(())
                    })
                    .await;
            }
            Err(err) => error!("Error retrieving all sensors: {}.", err),
        }
        info!("Retrieved all global sensors.");

        // store global sensor values per quantity
        let mut global_sensor_temperatures = Vec::new();
        let mut global_sensor_wind_speed = Vec::new();
        let mut global_sensor_irradiance = Vec::new();
        debug!("amt global sensors: {}.", global_sensors.len());
        for sensor in &global_sensors {
            // retrieve corresponding sensor temperature values (if any)
            match values_for_quantity_as_f64(&sensor_store, sensor, Quantity::Temperature).await {
                Err(_) => {
                    return Ok(Self {
                        delta_time,
                        models: HashMap::new(),
                    })
                }
                Ok(sensor_data_temperature) => {
                    global_sensor_temperatures = sensor_data_temperature;
                }
            }
            // retrieve corresponding sensor irradiance values (if any)
            match values_for_quantity_as_f64(&sensor_store, sensor, Quantity::Irradiance).await {
                Err(_) => {
                    return Ok(Self {
                        delta_time,
                        models: HashMap::new(),
                    })
                }
                Ok(sensor_data_irradiance) => {
                    global_sensor_irradiance = sensor_data_irradiance;
                }
            }
            // retrieve corresponding sensor wind speed values (if any)
            match values_for_quantity_as_f64(&sensor_store, sensor, Quantity::WindSpeed).await {
                Err(_) => {
                    return Ok(Self {
                        delta_time,
                        models: HashMap::new(),
                    })
                }
                Ok(sensor_data_wind_speed) => {
                    global_sensor_wind_speed = sensor_data_wind_speed;
                }
            }
        }
        // make models for all buildings.
        let mut models = HashMap::new();
        for (building_id, mut energy_data) in sensor_values_power_per_building.into_iter() {
            let mut global_sensor_temperatures = global_sensor_temperatures.clone();
            let mut global_sensor_irradiance = global_sensor_irradiance.clone();
            let mut global_sensor_wind_speed = global_sensor_wind_speed.clone();

            let mut data: Vec<f64> = Vec::new();
            // keep `min_length` amount of last sensor_values.
            let min_length = energy_data
                .len()
                .min(global_sensor_temperatures.len())
                .min(global_sensor_irradiance.len())
                .min(global_sensor_wind_speed.len());

            debug!("minimum length of data = {min_length}.");

            if min_length == 0 {
                continue;
            }

            // NOTE: clone due to independent amount of data per sensor.
            energy_data.drain(..energy_data.len() - min_length);
            global_sensor_temperatures.drain(..global_sensor_temperatures.len() - min_length);
            global_sensor_irradiance.drain(..global_sensor_irradiance.len() - min_length);
            global_sensor_wind_speed.drain(..global_sensor_wind_speed.len() - min_length);

            for (&energy, &temperature, &irradiance, &wind_speed) in izip!(
                &energy_data,
                &global_sensor_temperatures,
                &global_sensor_irradiance,
                &global_sensor_wind_speed
            ) {
                data.append(&mut vec![energy, temperature, irradiance, wind_speed]);
            }
            if data.is_empty() {
                debug!("data was empty for building_id = {building_id}.");
                continue;
            }
            info!("Training model with {} rows.", data.len() / 5);
            models.insert(
                building_id,
                match tokio::task::spawn_blocking(|| VAR::new(data, 4)).await {
                    Err(err) => {
                        error!("trainer thread crashed: {err}");
                        return Ok(Self {
                            delta_time,
                            models: HashMap::new(),
                        });
                    }
                    Ok(Some(var)) => var,
                    Ok(None) => {
                        debug!("Model training failed for building_id = {building_id}.");
                        return Ok(Self {
                            delta_time,
                            models: HashMap::new(),
                        });
                    }
                },
            );
        }
        info!("Finished model training.");
        debug!("building_id's with models: {:?}.", models.keys());
        Ok(Self { delta_time, models })
    }

    async fn do_timestep(&mut self, mut graph: Graph) -> Result<Graph, SimulationError> {
        info!("Doing timestep!");
        //TODO: NO UNWRAPS AND EXPECTS
        if let Some(_time_component) = graph.get_global_component::<TimeComponent>() {
            let consumer_nodes: Vec<NodeId> = graph
                .get_all_nodes::<SensorLoadNode>()
                .unwrap()
                .map(|n| n.0)
                .collect();
            debug!(
                "amount of consumer nodes in graph = {}.",
                consumer_nodes.len()
            );

            for node_id in consumer_nodes {
                let Some(building_component) = graph.get_node_component::<Building>(node_id) else {
                    debug!(
                        "Could not find building component for node_id = {:?}.",
                        node_id
                    );
                    continue;
                };

                // get building id from consumer node
                let building_id = building_component.building_id;
                debug!("generating predictions for building_id = {}.", building_id);
                // SAFETY: this unwrap is safe because node ids got fetched earlier by using the consumer node as component type
                let component = graph
                    .get_node_component_mut::<SensorLoadNode>(node_id)
                    .unwrap();

                //TODO: use time component and building id to make prediction for consumer node
                // This is naive for now, we expect that the next timedelta is 10s.
                // if no model is found for the building_id, skip the consumer.
                let model = match self.models.get_mut(&building_id) {
                    Some(model) => model,
                    None => {
                        debug!(
                            "NO model found for building_id = {}! Skipping prediction step.",
                            building_id
                        );
                        continue;
                    }
                };
                let prediction = model.get_next_prediction();
                // The energy field is always the first element of the prediction model's result.
                debug!("predicted {}.", prediction[0]);
                component.active_power = prediction[0];
                component.reactive_power = 0.02 * component.active_power;
            }

            let current_irradiance: f64 = match graph.get_global_component::<IrradianceComponent>()
            {
                Some(item) => item.irradiance,
                None => 10.,
            };
            let current_wind_speed: f64 = match graph.get_global_component::<WindSpeedComponent>() {
                Some(wind) => wind.wind_speed,
                None => 10.,
            };

            for (id, _, component) in graph.get_all_nodes_mut::<SensorGeneratorNode>().unwrap() {
                debug!(
                    "predicting producer node type = {:?}.",
                    component.power_type
                );

                match &component.power_type {
                    PowerType::Nuclear => {
                        let mut rng = rand::thread_rng();
                        let efficiency = rng.gen_range(0.99..=1.00);
                        component.active_power =
                            component.active_power * efficiency * self.delta_time.as_secs_f64()
                                / 3600.0;
                    }
                    PowerType::Solar => {
                        // Hash the node id as a pseudo way to give every house random but contestant solar panel area
                        let mut h = DefaultHasher::new();
                        id.hash(&mut h);
                        let solar_panel_area = h.finish() % 10 + 2;

                        component.active_power = solar_panel_area as f64 * current_irradiance;
                    }
                    PowerType::Wind => {
                        // https://thundersaidenergy.com/downloads/wind-power-impacts-of-larger-turbines/
                        // c_p = efficiency percentage. Theoretical maximum * small error factor.
                        let mut rng = rand::thread_rng();
                        let efficiency = rng.gen_range(0.98..=1.00);
                        let c_p = 0.593 * efficiency;
                        // rho = air_density, kg / m^3. source: wikipedia
                        let rho = 1.204;
                        // length of a single blade in meters. range of average lengths.
                        let blade_length: f64 = rng.gen_range(35.0..45.0);
                        component.active_power = 0.5
                            * c_p
                            * rho
                            * PI
                            * blade_length.powi(2)
                            * current_wind_speed.powi(3);
                    }
                    _ => {
                        //TODO: this code was present previously and thus kept for now
                        let delta_capacity = rand::thread_rng().gen_range(-50.0..50.0)
                            * self.delta_time.as_secs() as f64;
                        component.active_power =
                            (component.active_power + delta_capacity).clamp(1000.0, 2000.0)
                    }
                }
            }
        } else {
            error!("No time component was found.");
        };

        let mut num_consumer_nodes = 0;
        let mut num_producer_nodes = 0;
        let mut total_demand = 0.0;
        let mut total_capacity = 0.0;

        let components = graph
            .get_all_nodes::<SensorLoadNode>()
            .into_iter()
            .flatten();
        for (_, _, component) in components {
            num_consumer_nodes += 1;
            total_demand += component.active_power
        }

        let num_edges = 0;

        let mut power_type_percentages: HashMap<PowerType, f64> = HashMap::new();

        let components = graph
            .get_all_nodes::<SensorGeneratorNode>()
            .into_iter()
            .flatten();
        for (_, _, component) in components {
            num_producer_nodes += 1;
            total_capacity += component.active_power;
            let counter = power_type_percentages
                .entry(component.power_type)
                .or_insert(0.0);
            *counter += component.active_power
        }

        for (_, percentage) in power_type_percentages.iter_mut() {
            if total_capacity != 0.0 {
                *percentage /= total_capacity
            }
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
            analytics.transmission_edges_count = num_edges;
            analytics.total_demand = total_demand;
            analytics.total_capacity = total_capacity;
            if total_capacity != 0.0 {
                analytics.utilization = total_demand / total_capacity;
            }
        } else {
            debug!("No analytics component found");
        }
        Ok(graph)
    }
}
