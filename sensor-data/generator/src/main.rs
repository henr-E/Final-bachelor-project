use chrono::{NaiveDate, NaiveDateTime};
use tracing::debug;

use proto::sensor_data_ingest::DataIngestServiceClient;
use sensor_store::SensorStore;

use crate::sensor_data_generator::SensorDataGenerator;

mod measurements;
mod sensor_data_generator;
mod virtual_sensor;

const INGEST_SERVICE_URL: &str = "http://0.0.0.0:8080";

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

    // TODO: read address and port from environment
    // connect to DataIngestService
    let mut client = DataIngestServiceClient::connect(INGEST_SERVICE_URL)
        .await
        .unwrap();

    // send the sensor data to the ingest service one by one
    for sensor_data in sensor_data_files {
        // send one sensor message to the ingest service
        let request = tonic::Request::new(sensor_data);
        // print response
        let response = client.test_parse_sensor_data(request).await.unwrap();
        debug!("RESPONSE={:?}", response);
    }
}
