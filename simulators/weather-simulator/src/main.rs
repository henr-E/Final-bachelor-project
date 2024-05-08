use std::{
    collections::HashMap,
    env,
    io::{Error, ErrorKind},
    net::SocketAddr,
    process::ExitCode,
};

use chrono::NaiveDateTime;
use futures::StreamExt;
use itertools::izip;
use tracing::{debug, error, info};

use component_library::global::{
    IrradianceComponent, PrecipitationComponent, TemperatureComponent, TimeComponent,
    WindDirectionComponent, WindSpeedComponent,
};
use predictions::VAR;
use sensor_store::{Quantity, Sensor, SensorStore};
use simulator_communication::{
    simulator::SimulationError, ComponentsInfo, Graph, Server, Simulator,
};
use simulator_utilities::sensor::{average_dataset, values_for_quantity_as_f64};

// 6 * 10 = 60 entries, representing 10 minutes of data.
const AVERAGE_AMT: usize = 6 * 10;

// construct final state from a graph.
// if one of the components is not there, skip.
fn last_global_state(graph: &Graph) -> Option<Vec<f64>> {
    Some(vec![
        graph
            .get_global_component::<PrecipitationComponent>()?
            .precipitation,
        graph
            .get_global_component::<TemperatureComponent>()?
            .current_temp,
        graph
            .get_global_component::<IrradianceComponent>()?
            .irradiance,
        graph
            .get_global_component::<WindSpeedComponent>()?
            .wind_speed,
        graph
            .get_global_component::<WindDirectionComponent>()?
            .wind_direction,
    ])
}

#[tokio::main]
async fn main() -> ExitCode {
    _ = dotenvy::dotenv();
    tracing_subscriber::fmt().init();

    let listen_addr = match env::var("WEATHER_SIMULATOR_ADDR")
        .unwrap_or("127.0.0.1:8103".to_string())
        .parse::<SocketAddr>()
    {
        Ok(v) => v,
        Err(err) => {
            error!("Could not parse bind address: {err}.");
            return ExitCode::FAILURE;
        }
    };

    let server = Server::<WeatherSimulator>::new();

    // Manager address
    let connector_addr =
        env::var("SIMULATOR_CONNECTOR_ADDR").unwrap_or("http://127.0.0.1:8099".to_string());

    info!("Starting weather simulator server on `{listen_addr}`.");
    if let Err(err) = server
        .start(listen_addr, connector_addr, "weather simulator")
        .await
    {
        error!("Server return an error: {err}.");
        return ExitCode::FAILURE;
    }

    info!("Server exited successfully.");
    ExitCode::SUCCESS
}

pub struct WeatherSimulator {
    model: VAR,
    start_time: Option<NaiveDateTime>,
    cache: HashMap<i64, Vec<f64>>,
}

impl Simulator for WeatherSimulator {
    fn get_component_info() -> ComponentsInfo {
        ComponentsInfo::new()
            // Time component is required to make predictions
            .add_required_component::<TimeComponent>()
            // components that the simulator will retrieve and return with adjusted values
            .add_optional_component::<TemperatureComponent>()
            .add_optional_component::<PrecipitationComponent>()
            .add_optional_component::<WindSpeedComponent>()
            .add_optional_component::<WindDirectionComponent>()
            .add_optional_component::<IrradianceComponent>()
            .add_output_component::<TemperatureComponent>()
            .add_output_component::<PrecipitationComponent>()
            .add_output_component::<WindSpeedComponent>()
            .add_output_component::<WindDirectionComponent>()
            .add_output_component::<IrradianceComponent>()
    }

    async fn new(_delta_time: std::time::Duration, graph: Graph) -> Result<Self, SimulationError> {
        info!("Started new weather simulator.");
        // try to connect with sensor database
        let sensor_store = match SensorStore::new().await {
            Ok(sensor_store) => sensor_store,
            Err(err) => {
                error!("Database connection failed: {}", err);
                return Err(SimulationError::Internal(Box::new(err)));
            }
        };
        // Retrieve all global sensors
        let mut global_sensors: Vec<Sensor> = Vec::new();
        match sensor_store.get_all_global_sensors().await {
            Ok(sensor_stream) => {
                // Iterate over the stream of sensors
                sensor_stream
                    .for_each(|sensor_result| {
                        match sensor_result {
                            Ok(sensor) => global_sensors.push(sensor),
                            Err(err) => error!("Error retrieving sensor: {}", err),
                        }
                        futures::future::ready(())
                    })
                    .await;
            }
            Err(err) => error!("Error retrieving all sensors: {}", err),
        }
        info!("Retrieved all global sensors.");
        debug!("amount of global_sensors = {}", global_sensors.len());

        // NOTE: This code seems a bit odd. this is because the signals for global sensors in
        // the database are unique. So there are no 2 temperature signals in the database
        // that share the global sensor property.

        // store global sensor values per quantity
        let mut global_sensors_temperatures = Vec::new();
        let mut global_sensors_wind_speed = Vec::new();
        let mut global_sensors_irradiance = Vec::new();
        let mut global_sensors_precipitation = Vec::new();
        let mut global_sensors_wind_direction = Vec::new();
        for sensor in &global_sensors {
            // retrieve corresponding sensor temperature values (if any)
            match values_for_quantity_as_f64(&sensor_store, sensor, Quantity::Temperature).await {
                Err(_) => {}
                Ok(sensor_data_temperature) => {
                    global_sensors_temperatures = sensor_data_temperature;
                }
            }
            // retrieve corresponding sensor irradiance values (if any)
            match values_for_quantity_as_f64(&sensor_store, sensor, Quantity::Irradiance).await {
                Err(_) => {}
                Ok(sensor_data_irradiance) => {
                    global_sensors_irradiance = sensor_data_irradiance;
                }
            }
            // retrieve corresponding sensor wind speed values (if any)
            match values_for_quantity_as_f64(&sensor_store, sensor, Quantity::WindSpeed).await {
                Err(_) => {}
                Ok(sensor_data_wind_speed) => {
                    global_sensors_wind_speed = sensor_data_wind_speed;
                }
            }
            // retrieve corresponding sensor rainfall values (if any)
            match values_for_quantity_as_f64(&sensor_store, sensor, Quantity::Rainfall).await {
                Err(_) => {}
                Ok(sensor_data_precipitation) => {
                    global_sensors_precipitation = sensor_data_precipitation;
                }
            }
            // retrieve corresponding sensor wind direction values (if any)
            match values_for_quantity_as_f64(&sensor_store, sensor, Quantity::WindDirection).await {
                Err(_) => {}
                Ok(sensor_data_wind_direction) => {
                    global_sensors_wind_direction = sensor_data_wind_direction;
                }
            }
        }
        debug!(
            "Amount of global_sensors_precipitation= {}.",
            global_sensors_precipitation.len()
        );
        debug!(
            "Amount of global_sensors_temperatures = {}.",
            global_sensors_temperatures.len()
        );
        debug!(
            "Amount of global_sensors_irradiance = {}.",
            global_sensors_irradiance.len()
        );
        debug!(
            "Amount of global_sensors_wind_speed = {}.",
            global_sensors_wind_speed.len()
        );
        debug!(
            "Amount of global_sensors_wind_direction = {}.",
            global_sensors_wind_direction.len()
        );
        // average values of dataset.
        average_dataset(&mut global_sensors_temperatures, AVERAGE_AMT);
        average_dataset(&mut global_sensors_irradiance, AVERAGE_AMT);
        average_dataset(&mut global_sensors_wind_speed, AVERAGE_AMT);
        average_dataset(&mut global_sensors_precipitation, AVERAGE_AMT);
        average_dataset(&mut global_sensors_wind_direction, AVERAGE_AMT);

        let min_length = global_sensors_precipitation
            .len()
            .min(global_sensors_temperatures.len())
            .min(global_sensors_irradiance.len())
            .min(global_sensors_wind_speed.len())
            .min(global_sensors_wind_direction.len());
        debug!("minimum length of data: {}", min_length);

        global_sensors_temperatures.drain(..global_sensors_temperatures.len() - min_length);
        global_sensors_irradiance.drain(..global_sensors_irradiance.len() - min_length);
        global_sensors_wind_speed.drain(..global_sensors_wind_speed.len() - min_length);
        global_sensors_precipitation.drain(..global_sensors_precipitation.len() - min_length);
        global_sensors_wind_direction.drain(..global_sensors_wind_direction.len() - min_length);

        // making total dataset.
        let mut data: Vec<f64> = Vec::with_capacity(min_length);
        for (&temperature, &irradiance, &wind_speed, &precipitation, &wind_direction) in izip!(
            &global_sensors_temperatures,
            &global_sensors_irradiance,
            &global_sensors_wind_speed,
            &global_sensors_precipitation,
            &global_sensors_wind_direction,
        ) {
            data.append(&mut vec![
                precipitation,
                temperature,
                irradiance,
                wind_speed,
                wind_direction,
            ]);
        }
        if let Some(last_state) = last_global_state(&graph) {
            data.extend(last_state);
        }
        info!("Training model with {} rows", data.len() / 5);
        if data.is_empty() {
            debug!("Data is empty!");
            return Err(SimulationError::Internal(Box::new(Error::new(
                ErrorKind::InvalidData,
                "There is not enough training data available to train the prediction model.",
            ))));
        }

        info!("Training model with {} rows", data.len() / 5);
        match VAR::new(data, 5) {
            Some(model) => {
                info!("Finished model training. Order = {}", model.get_order());
                Ok(Self {
                    model,
                    cache: HashMap::new(),
                    start_time: None,
                })
            }
            None => Err(SimulationError::Internal(Box::new(Error::new(
                ErrorKind::Other,
                "Failed to train prediction model.",
            )))),
        }
    }

    async fn do_timestep(&mut self, mut graph: Graph) -> Result<Graph, SimulationError> {
        if let Some(time_component) = graph.get_global_component::<TimeComponent>() {
            if self.start_time.is_none() {
                self.start_time = Some(time_component.0);
            }
            let pred_n = (time_component.0.and_utc().timestamp()
                - self.start_time.unwrap().and_utc().timestamp())
                / (10 * AVERAGE_AMT) as i64;

            let predictions = match self.cache.get(&pred_n) {
                Some(p) => p,
                None => {
                    let last = self.cache.keys().copied().max().unwrap_or_default();

                    // make sure that the unwrap below never panics.
                    if last == 0 {
                        self.cache.insert(0, self.model.get_next_prediction());
                    }
                    let preds = self.model.get_next_predictions((pred_n - last) as usize);
                    for (ith, pred) in preds.iter().enumerate() {
                        self.cache.insert(last + ith as i64 + 1, pred.to_vec());
                    }
                    self.cache.get(&pred_n).unwrap()
                }
            };
            // let predictions = self.model.get_next_prediction();

            if let Some(temperature_component) =
                graph.get_global_component_mut::<TemperatureComponent>()
            {
                temperature_component.current_temp = predictions[1] * temperature_component.scalar;
            } else {
                error!("No temperature component was found.");
            }

            if let Some(irradiance_component) =
                graph.get_global_component_mut::<IrradianceComponent>()
            {
                irradiance_component.irradiance = predictions[2] * irradiance_component.scalar;
            } else {
                error!("No irradiance component was found.");
            }

            if let Some(wind_speed_component) =
                graph.get_global_component_mut::<WindSpeedComponent>()
            {
                wind_speed_component.wind_speed = predictions[3] * wind_speed_component.scalar;
            } else {
                error!("No wind speed component was found.");
            }

            if let Some(precipitation_component) =
                graph.get_global_component_mut::<PrecipitationComponent>()
            {
                precipitation_component.precipitation =
                    predictions[0] * precipitation_component.scalar;
            } else {
                error!("No precipitation component was found.");
            }

            if let Some(wind_direction_component) =
                graph.get_global_component_mut::<WindDirectionComponent>()
            {
                wind_direction_component.wind_direction = predictions[4];
            } else {
                error!("No wind direction component was found.");
            }
        } else {
            error!("No time component was found.");
        };
        Ok(graph)
    }
}
