use std::collections::HashMap;

use async_job::{async_trait, Job, Runner, Schedule};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use tokio::time::Duration;
use tracing::{debug, error, info, Level};
use uuid::Uuid;

use environment_config::env;
use proto::sensor_data_ingest::{
    sensor_data_file::FileFormat, DataIngestServiceClient, JsonFileFormat, SensorDataFile,
};
use sensor_store::SensorStore;

use crate::{
    sensor_data_generator::{SensorDataGenerator, VIRTUAL_SENSOR_GENERATION_INTERVAL},
    virtual_sensor::VirtualSensor,
};

mod measurements;
mod sensor_data_generator;
mod virtual_sensor;

const INGEST_SERVICE_URL_ENV: &str = "SENSOR_DATA_INGESTOR_URL";
const DEFAULT_INGEST_SERVICE_URL: &str = "http://0.0.0.0:8084";
const DAYS_OF_EXTRA_DATA: u64 = 14;

struct GeneratorJob {
    sensor_store: SensorStore,
    sensor_found_map: HashMap<Uuid, DateTime<Utc>>,
}

#[async_trait]
impl Job for GeneratorJob {
    fn schedule(&self) -> Option<Schedule> {
        Some("*/10 * * * * *".parse().unwrap())
    }
    async fn handle(&mut self) {
        let mut sensor_data_files = Vec::new();
        // If we are in the first hour of generating sensor data
        if let Some(data) = generate_last_n_days(
            DAYS_OF_EXTRA_DATA,
            &self.sensor_store,
            &mut self.sensor_found_map,
        )
        .await
        {
            sensor_data_files = data;
        };
        debug!("running sensor data generator");

        let mut sensor_data_generator = SensorDataGenerator::new();
        // retrieve registered sensors from the database
        sensor_data_generator
            .retrieve_sensors_from_db(&self.sensor_store)
            .await;

        // establish the time frame for which sensor data should be generated
        let timestamp_begin = (Utc::now() - Duration::from_secs(3600)).timestamp() as u64;
        let timestamp_end = Utc::now().timestamp() as u64;

        // generate the sensor data
        sensor_data_files.extend(sensor_data_generator.generate(timestamp_begin, timestamp_end));

        // connect to DataIngestService
        let mut client = match DataIngestServiceClient::connect(
            env(INGEST_SERVICE_URL_ENV).unwrap_or(DEFAULT_INGEST_SERVICE_URL),
        )
        .await
        {
            Ok(client) => client,
            Err(err) => {
                error!("Failed to connect to the ingest service: {err}");
                return;
            }
        };

        let response = client
            .ingest_sensor_data_file_stream(tonic::Request::new(tokio_stream::iter(
                sensor_data_files,
            )))
            .await
            .expect("failed to send request");
        debug!("RESPONSE={:?}", response);
    }
}

async fn generate_last_n_days(
    n: u64,
    sensor_store: &SensorStore,
    sensor_found_map: &mut HashMap<Uuid, DateTime<Utc>>,
) -> Option<Vec<SensorDataFile>> {
    let mut sensors = HashMap::new();

    // get all sensors, register first time that a sensor is seen in a hashmap.
    // this time is used later to determine if a sensor needs more historical data.
    let amount_per_signal: HashMap<Uuid, u64> =
        match sensor_store.get_sensor_signal_value_count().await {
            Ok(value) => value,
            Err(_) => return None,
        };
    match sensor_store.get_all_sensors().await {
        Ok(a) => {
            a.for_each(|sensor_result| {
                match sensor_result {
                    Ok(sensor) => {
                        // check if the sensor already has a lot of values.
                        // this is implemented in case a user has pushed their own data to the system.
                        let sensor_id: Uuid = sensor.id;
                        // no unwrap due to timing issue
                        if let Some(&amt) = amount_per_signal.get(&sensor_id) {
                            // if there is already some data for this sensor, don't register it.
                            if amt < 1000 {
                                sensor_found_map.entry(sensor_id).or_insert(Utc::now());
                            }
                            sensors.insert(sensor_id, sensor);
                        }
                    }
                    Err(err) => error!("Error retrieving sensor: {}", err),
                }
                futures::future::ready(())
            })
            .await;
        }
        Err(err) => error!("Error retrieving all sensors: {}", err),
    };
    // make list of sensors that need extra historical data.
    let mut virtual_sensors = Vec::new();
    for (senssor_id, time) in sensor_found_map.iter() {
        if Utc::now().timestamp() - time.timestamp() < 3600 {
            if let Some(sensor) = sensors.get(senssor_id) {
                let virtual_sensor = VirtualSensor::new(
                    sensor.clone(),
                    VIRTUAL_SENSOR_GENERATION_INTERVAL.as_secs(),
                    FileFormat::Json(JsonFileFormat {}),
                );
                virtual_sensors.push(virtual_sensor);
            }
        }
    }
    if virtual_sensors.is_empty() {
        return None;
    }
    debug!("Adding historical data.");
    // generate data for these sensors.
    let mut sensor_data_generator = SensorDataGenerator::from(virtual_sensors);

    // establish the time frame for which sensor data should be generated.
    // from (now-1h) to (now - n days).
    let timestamp_begin = (Utc::now() - Duration::from_secs(3600 * 24 * n)).timestamp() as u64;
    let timestamp_end = (Utc::now() - Duration::from_secs(3600)).timestamp() as u64;

    // generate the sensor data
    Some(sensor_data_generator.generate(timestamp_begin, timestamp_end))
}

async fn run() {
    let mut runner = Runner::new();
    runner = runner.add(Box::new(GeneratorJob {
        sensor_store: SensorStore::new().await.unwrap(),
        sensor_found_map: HashMap::new(),
    }));

    info!("Starting the runner");
    runner.run().await;

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");
}

/// Generate fake sensor data and send it to the ingest service.
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    run().await;
}
