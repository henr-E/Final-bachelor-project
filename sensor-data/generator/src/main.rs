use chrono::{NaiveDate, NaiveDateTime};
use tracing::debug;

use environment_config::env;
use proto::sensor_data_ingest::DataIngestServiceClient;
use sensor_store::SensorStore;

use crate::sensor_data_generator::SensorDataGenerator;

mod measurements;
mod sensor_data_generator;
mod virtual_sensor;

const INGEST_SERVICE_URL_ENV: &str = "SENSOR_DATA_INGESTOR_URL";
const DEFAULT_INGEST_SERVICE_URL: &str = "http://0.0.0.0:8080";

/// Generate fake sensor data and send it to the ingest service.
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    // create a SensorDataGenerator object that can be used to generate fake sensor data
    let mut sensor_data_generator: SensorDataGenerator = SensorDataGenerator::new();
    // create an instance of the database wrapper
    let mut sensor_store = SensorStore::new().await.unwrap();
    // retrieve registered sensors from the database
    sensor_data_generator
        .retrieve_sensors_from_db(&mut sensor_store)
        .await;

    // establish the time frame for which sensor data should be generated
    let date_time_begin: NaiveDateTime = NaiveDate::from_ymd_opt(2024, 3, 2)
        .unwrap()
        .and_hms_opt(20, 44, 44)
        .unwrap();
    let timestamp_begin = date_time_begin.and_utc().timestamp();

    let date_time_end: NaiveDateTime = NaiveDate::from_ymd_opt(2024, 3, 3)
        .unwrap()
        .and_hms_opt(1, 0, 0)
        .unwrap();
    let timestamp_end = date_time_end.and_utc().timestamp();

    // generate the sensor data
    let sensor_data_files =
        sensor_data_generator.generate(timestamp_begin as u64, timestamp_end as u64);

    // connect to DataIngestService
    let mut client = DataIngestServiceClient::connect(
        env(INGEST_SERVICE_URL_ENV).unwrap_or(DEFAULT_INGEST_SERVICE_URL),
    )
    .await
    .unwrap();

    // print response
    let response = client
        .ingest_sensor_data_file_stream(tonic::Request::new(tokio_stream::iter(sensor_data_files)))
        .await
        .expect("failed to send request");
    debug!("RESPONSE={:?}", response);
}
