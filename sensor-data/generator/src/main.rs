use proto::sensor_data_ingest::sensor_data_file::FileFormat;
use proto::sensor_data_ingest::{
    data_ingest_service_client::DataIngestServiceClient, JsonFileFormat, SensorDataFile,
};

use crate::data_generator::SensorDataGenerator;
use crate::sensors::precipitation::SensorPrecipitation;
use crate::sensors::temperature::SensorTemperature;
use crate::sensors::Coordinates;

// mod csv_reader;
mod data_generator;
mod sensors;

const INGEST_SERVICE_URL: &str = "http://0.0.0.0:8080";

#[tokio::main]
async fn main() {
    // TODO: read address and port from environment
    // connect to DataIngestService
    let mut client = DataIngestServiceClient::connect(INGEST_SERVICE_URL)
        .await
        .unwrap();

    // create a SensorDataGenerator object
    let mut sensor_data_generator = SensorDataGenerator::new();

    // Add a temperature sensor
    let sensor1 = Box::new(SensorTemperature {
        coordinates: Coordinates {
            latitude: 23.5,
            longitude: 60.2,
        },
        interval: 30, // set the interval to 30 seconds
    });

    // Add a precipitation sensor
    let sensor2 = Box::new(SensorPrecipitation {
        coordinates: Coordinates {
            latitude: 18.5,
            longitude: 0.2,
        },
        interval: 60, // set the interval to 60 seconds
    });

    // add sensors to data generator
    sensor_data_generator.add_sensor(sensor1);
    sensor_data_generator.add_sensor(sensor2);

    // generate data for a time range
    let begin: u64 = 0;
    let end: u64 = 75;
    let generated_data = sensor_data_generator.generate_data(begin, end);

    // send data to DataIngestService
    let request = tonic::Request::new(SensorDataFile {
        data: generated_data,
        data_format: "".to_string(),
        file_format: Some(FileFormat::Json(JsonFileFormat {})), // TODO: add correct file format
    });

    // print response
    let response = client.test_parse_sensor_data(request).await.unwrap();
    println!("RESPONSE={:?}", response);
}
