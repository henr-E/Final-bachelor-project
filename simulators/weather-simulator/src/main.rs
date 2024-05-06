use std::{env, net::SocketAddr, process::ExitCode};

use futures::stream::StreamExt;
use itertools::izip;
use rand_distr::num_traits::ToPrimitive;
use sqlx::types::BigDecimal;
use thiserror::Error;
use tracing::{debug, error, info, Level};

use component_library::global::{
    IrradianceComponent, PrecipitationComponent, TemperatureComponent, TimeComponent,
    WindDirectionComponent, WindSpeedComponent,
};
use predictions::VAR;
use sensor_store::{Quantity, Sensor, SensorStore};
use simulator_communication::{
    simulator::SimulationError, ComponentsInfo, Graph, Server, Simulator,
};

/// Errors that can occur
#[derive(Debug, Error)]
pub enum WeatherError {
    #[error("Unable to convert a vector of BigDecimal values to a vector of f64 values.")]
    FailedConversion(),
}

/// Convert a vector of BigDecimal values to a vector of f64 values
fn big_decimals_to_floats(values: Vec<BigDecimal>) -> Result<Vec<f64>, WeatherError> {
    let floats: Result<Vec<f64>, WeatherError> = values
        .into_iter()
        .map(|bd| bd.to_f64().ok_or(WeatherError::FailedConversion()))
        .collect();
    floats
}

async fn get_sensor_data_for_quantity_and_sensor(
    sensor_store: &SensorStore,
    sensor: &Sensor<'_>,
    quantity: Quantity,
) -> Option<Vec<f64>> {
    let values_as_big_decimals = match sensor
        .signal_values_for_quantity(sensor_store, quantity)
        .await
    {
        Ok(values_as_big_decimal) => values_as_big_decimal,
        Err(err) => {
            error!("Error retrieving the signal values: {}", err);
            return None;
        }
    };
    let values_as_floats = match big_decimals_to_floats(values_as_big_decimals) {
        Ok(values_as_floats) => values_as_floats,
        Err(err) => {
            error!(
                "Failed to convert vector of big decimal values to vector of float values.: {}",
                err
            );
            return None;
        }
    };
    if values_as_floats.is_empty() {
        return None;
    }
    Some(values_as_floats)
}

#[tokio::main]
async fn main() -> ExitCode {
    _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

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
    model: Option<VAR>,
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

    async fn new(_delta_time: std::time::Duration, _graph: Graph) -> Result<Self, SimulationError> {
        info!("Started new weather simulator.");
        // try to connect with sensor database
        let sensor_store = match SensorStore::new().await {
            Ok(sensor_store) => sensor_store,
            Err(err) => {
                error!("Database connection failed: {}", err);
                return Ok(Self { model: None });
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
            match get_sensor_data_for_quantity_and_sensor(
                &sensor_store,
                sensor,
                Quantity::Temperature,
            )
            .await
            {
                None => {}
                Some(sensor_data_temperature) => {
                    global_sensors_temperatures = sensor_data_temperature;
                }
            }
            // retrieve corresponding sensor irradiance values (if any)
            match get_sensor_data_for_quantity_and_sensor(
                &sensor_store,
                sensor,
                Quantity::Irradiance,
            )
            .await
            {
                None => {}
                Some(sensor_data_irradiance) => {
                    global_sensors_irradiance = sensor_data_irradiance;
                }
            }
            // retrieve corresponding sensor wind speed values (if any)
            match get_sensor_data_for_quantity_and_sensor(
                &sensor_store,
                sensor,
                Quantity::WindSpeed,
            )
            .await
            {
                None => {}
                Some(sensor_data_wind_speed) => {
                    global_sensors_wind_speed = sensor_data_wind_speed;
                }
            }
            // retrieve corresponding sensor rainfall values (if any)
            match get_sensor_data_for_quantity_and_sensor(&sensor_store, sensor, Quantity::Rainfall)
                .await
            {
                None => {}
                Some(sensor_data_precipitation) => {
                    global_sensors_precipitation = sensor_data_precipitation;
                }
            }
            // retrieve corresponding sensor wind direction values (if any)
            match get_sensor_data_for_quantity_and_sensor(
                &sensor_store,
                sensor,
                Quantity::WindDirection,
            )
            .await
            {
                None => {}
                Some(sensor_data_wind_direction) => {
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

        let mut data: Vec<f64> = Vec::new();
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
        info!("Training model with {} rows", data.len() / 5);
        if data.is_empty() {
            debug!("Data is empty!");
            return Ok(Self { model: None });
        }
        let model = VAR::new(data, 5);
        info!("Finished model training.");
        debug!("model training succeeded = {}", model.is_some());
        Ok(Self { model })
    }

    async fn do_timestep(&mut self, mut graph: Graph) -> Result<Graph, SimulationError> {
        info!("Doing timestep!");
        if let Some(_time_component) = graph.get_global_component::<TimeComponent>() {
            let predictions = match self.model.as_mut() {
                None => {
                    error!("Failed to create prediction model.");
                    return Ok(graph);
                }
                Some(model) => model.get_next_prediction(),
            };

            if let Some(temperature_component) =
                graph.get_global_component_mut::<TemperatureComponent>()
            {
                temperature_component.current_temp = predictions[1];
            } else {
                error!("No temperature component was found.");
            }

            if let Some(irradiance_component) =
                graph.get_global_component_mut::<IrradianceComponent>()
            {
                irradiance_component.irradiance = predictions[2];
            } else {
                error!("No irradiance component was found.");
            }

            if let Some(wind_speed_component) =
                graph.get_global_component_mut::<WindSpeedComponent>()
            {
                wind_speed_component.wind_speed = predictions[3];
            } else {
                error!("No wind speed component was found.");
            }

            if let Some(precipitation_component) =
                graph.get_global_component_mut::<PrecipitationComponent>()
            {
                precipitation_component.precipitation = predictions[0];
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
