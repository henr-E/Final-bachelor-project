use async_job::{async_trait, Job, Runner, Schedule};
use chrono::Utc;
use tokio::time::Duration;
use tracing::{debug, error, info};

use environment_config::env;
use proto::sensor_data_ingest::DataIngestServiceClient;
use sensor_store::SensorStore;

use crate::sensor_data_generator::SensorDataGenerator;

mod measurements;
mod sensor_data_generator;
mod virtual_sensor;

const INGEST_SERVICE_URL_ENV: &str = "SENSOR_DATA_INGESTOR_URL";
const DEFAULT_INGEST_SERVICE_URL: &str = "http://0.0.0.0:8084";

struct GeneratorJob {
    sensor_store: SensorStore,
}

#[async_trait]
impl Job for GeneratorJob {
    fn schedule(&self) -> Option<Schedule> {
        Some("*/10 * * * * *".parse().unwrap())
    }
    async fn handle(&mut self) {
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
        let sensor_data_files = sensor_data_generator.generate(timestamp_begin, timestamp_end);

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

async fn run() {
    let mut runner = Runner::new();
    runner = runner.add(Box::new(GeneratorJob {
        sensor_store: SensorStore::new().await.unwrap(),
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
    tracing_subscriber::fmt()
        .with_env_filter("sensor_data_generator=DEBUG,INFO")
        .init();
    run().await;
}
