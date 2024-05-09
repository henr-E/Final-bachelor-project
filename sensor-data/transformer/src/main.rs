use async_job::{async_trait, Job, Runner, Schedule};
use bigdecimal::ToPrimitive;
use bson::{doc, Bson};
use database_config::database_url;
use rink_core::*;
use sensor_store::{Quantity, SensorStore, Signal};
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;
use sqlx::types::BigDecimal;
use sqlx::{postgres::PgPoolOptions, types::chrono::Utc, PgPool};
use std::{collections::HashSet, fs::File, str::FromStr};
use tokio::time::Duration;
use tracing::{debug, error, info};
use uuid::Uuid;

const CRON_TIME: Duration = Duration::from_secs(5);
const BATCH_SIZE: usize = 1024;

struct TransformerJob {
    pool: PgPool,
    value_buffer: Vec<SensorValue>,
    unit_conversion_context: Context,
}

#[async_trait]
impl Job for TransformerJob {
    fn schedule(&self) -> Option<Schedule> {
        Some(
            format!("1/{} * * * * *", CRON_TIME.as_secs())
                .parse()
                .unwrap(),
        )
    }
    async fn handle(&mut self) {
        info!("Checking for new entries in the archive database...");
        match self.read_sensor_data().await {
            Ok(ReadSensorDataResponse::EntriesAdded) => {
                match self.push_to_database().await {
                    Ok(amt) => {
                        info!("Successfully pushed {amt} sensor value chunks to the database.")
                    }
                    Err(error) => error!("{}", error),
                };
            }
            Ok(ReadSensorDataResponse::NoNewData) => return,
            Err(error) => match error {
                ReadSensorDataError::FileNotFound(path) => error!("File `{}` was not found.", path),
                ReadSensorDataError::InvalidBson(bson_err) => error!("{}", bson_err),
                ReadSensorDataError::InvalidMeasurement(measurement_error) => {
                    error!("{}", measurement_error)
                }
            },
        }
    }
}

#[derive(Debug)]
pub enum ReadSensorDataError {
    FileNotFound(String),
    InvalidBson(String),
    InvalidMeasurement(String),
}

pub enum ReadSensorDataResponse {
    NoNewData,
    EntriesAdded,
}

#[derive(Debug, Serialize)]
struct SensorValue {
    timestamp: String,
    value: String,
    sensor_signal_id: i32,
}

#[derive(Debug, Deserialize)]
struct SensorDataFile {
    sensor_id: Uuid,
    path: String,
}

impl TransformerJob {
    /// Fetch all new data files from database.
    async fn new_data_files(&self) -> Vec<SensorDataFile> {
        let interval = PgInterval::try_from(CRON_TIME).unwrap();
        sqlx::query_as!(
            SensorDataFile,
            "SELECT sensor_id, path FROM archive_sensor_data_files AS sdf WHERE sdf.time > NOW() - $1::interval AND sdf.time < now();",
            interval
        )
            .fetch_all(&self.pool)
            .await.expect("archive_sensor_data_files query failed.")
    }

    /// Create sensor value for a signal from a measurement.
    fn sensor_value(
        &mut self,
        signal: &Signal,
        timestamp_signal: &Signal,
        measurement: &Bson,
    ) -> SensorValue {
        let data_unit = &signal.unit;
        // unwrapping here is ok since there is a foreign key constraint on `signal_type`.
        let base_unit = data_unit.base_unit();
        // get the prefix
        let prefix: &f64 = &signal.prefix.to_f64().expect("prefix is not a valid f64.");
        let value = measurement
            .as_document()
            .unwrap()
            .get_f64(&signal.name)
            .unwrap();
        // multiply value by prefix
        let scaled_value =
            prefix * value * data_unit.rink_multiplier() / base_unit.rink_multiplier();
        // transform the value from `data_unit` to `base_unit`.
        // it might need some
        let convert_string = format!(
            "{} {} -> {}",
            scaled_value,
            data_unit.to_rink(),
            base_unit.to_rink()
        );
        let normalized_value =
            one_line(&mut self.unit_conversion_context, &convert_string).unwrap();
        // extract timestamp from measurement
        let timestamp_name = &timestamp_signal.name;
        let timestamp = measurement
            .as_document()
            .unwrap()
            .get_f64(timestamp_name)
            .unwrap()
            .to_string();

        // create sensor value instance.
        SensorValue {
            value: normalized_value,
            sensor_signal_id: signal.id,
            timestamp,
        }
    }

    /// Read a bson file given a path from the database.
    fn parse_bson(&self, path: &str) -> Result<Bson, ReadSensorDataError> {
        let path = std::path::Path::new(path);
        if !path.exists() {
            return Err(ReadSensorDataError::FileNotFound(
                path.to_str().unwrap().to_string(),
            ));
        }

        // unwrap is ok since we check if the path exists in previous if statement.
        let file = File::open(path).unwrap();
        match bson::from_reader(&file) {
            Ok(bson) => Ok(bson),
            Err(e) => Err(ReadSensorDataError::InvalidBson(e.to_string())),
        }
    }

    /// Process signals and store the results in the buffer.
    fn process_signals<'a>(
        &mut self,
        signals: impl Iterator<Item = &'a Signal<'a>>,
        timestamp_signal: &Signal,
        measurement: &Bson,
    ) {
        for signal in signals {
            let sensor_value = self.sensor_value(signal, timestamp_signal, measurement);
            debug!("created sensor value: {:?}", &sensor_value);
            self.value_buffer.push(sensor_value);
        }
    }

    /// Reads new sensor data files from the database, parses the data, and stores the results in a buffer.
    async fn read_sensor_data(&mut self) -> Result<ReadSensorDataResponse, ReadSensorDataError> {
        // fetch files within the last 5 minutes.
        let data_files: Vec<SensorDataFile> = self.new_data_files().await;

        // early exit.
        if data_files.is_empty() {
            return Ok(ReadSensorDataResponse::NoNewData);
        }

        // list all the new data files
        debug!("rows: {:?}", data_files);

        for data_file in data_files.into_iter() {
            // read data file form path provided by the database.
            let data_bson: Result<Bson, ReadSensorDataError> = self.parse_bson(&data_file.path); // get sensor signals that are registered to this sensor.
            let sensor_store = SensorStore::from_pg_pool(&self.pool);
            let sensor = sensor_store
                .get_sensor(data_file.sensor_id)
                .await
                .expect("Could not get sensor signals from the database.");
            let sensor_signals = sensor.signals();

            let timestamp_signal = sensor_signals
                .iter()
                .find(|signal| signal.quantity == Quantity::Timestamp)
                .expect("Could not find a signal `timestamp`");

            let sensor_signals = sensor_signals
                .iter()
                .filter(|signal| signal.quantity != Quantity::Timestamp)
                .collect::<HashSet<_>>();

            match data_bson {
                Ok(bson) => {
                    match &bson {
                        Bson::Document(document) => {
                            let top_level_key = document
                                .keys()
                                .next()
                                .expect("document should contain at least one key");
                            let inner_value = document
                                .get(top_level_key)
                                .expect("key should have a value");
                            if let Some(measurements) = inner_value.as_array() {
                                // Multiple measurements
                                for measurement in measurements.iter() {
                                    self.process_signals(
                                        sensor_signals.iter().copied(),
                                        timestamp_signal,
                                        measurement,
                                    );
                                }
                            } else {
                                // Single measurement
                                self.process_signals(
                                    sensor_signals.iter().copied(),
                                    timestamp_signal,
                                    &bson,
                                );
                            }
                        }
                        Bson::Array(measurements) => {
                            for measurement in measurements.iter() {
                                self.process_signals(
                                    sensor_signals.iter().copied(),
                                    timestamp_signal,
                                    measurement,
                                );
                            }
                        }
                        _ => {
                            error!("Invalid BSON format.");
                        }
                    }
                }
                Err(e) => {
                    error!("Error parsing BSON: {:?}", e);
                }
            }
        }
        Ok(ReadSensorDataResponse::EntriesAdded)
    }

    fn clear_buffer(&mut self) {
        self.value_buffer.clear();
    }

    /// Asynchronously pushes sensor values stored in `value_buffer` to the database.
    async fn push_to_database(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        // divide sensor_values into chunks.
        let chunks: Vec<_> = self.value_buffer.chunks(BATCH_SIZE).collect();
        let mut amt_chunks = 0;
        for chunk in chunks {
            let timestamps: Vec<chrono::DateTime<Utc>> = chunk
                .iter()
                .map(|item| {
                    sqlx::types::chrono::DateTime::<Utc>::from_timestamp(
                        item.timestamp.parse::<i64>().unwrap(),
                        0,
                    )
                    .unwrap()
                })
                .collect();
            let values: Vec<BigDecimal> = chunk
                .iter()
                .map(|item| {
                    let value = item
                        .value
                        .split_whitespace()
                        .find(|item| item.parse::<f64>().is_ok())
                        .unwrap();

                    BigDecimal::from_str(value).unwrap()
                })
                .collect();
            let ids: Vec<i32> = chunk.iter().map(|item| item.sensor_signal_id).collect();
            if let Err(e) = sqlx::query!(
                "INSERT INTO sensor_values (timestamp, value, sensor_signal_id) SELECT * FROM UNNEST($1::timestamptz[], $2::decimal[], $3::int[])",
                &timestamps[..],
                &values[..],
                &ids[..]
            ).execute(&self.pool).await{
                // If the query fails, clear duplicate entries.
                self.clear_buffer();
                return Err(Box::new(e));
            }

            amt_chunks += 1;
        }
        self.clear_buffer();
        Ok(amt_chunks)
    }
}

async fn run() {
    let mut runner = Runner::new();
    let database_url = database_url("sensor_archive");

    // connect to the database.
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Could not open database connection.");

    // initialize the unit conversion library.
    let unit_conversion_context =
        simple_context().expect("Could not initialize unit conversion context.");

    runner = runner.add(Box::new(TransformerJob {
        pool,
        value_buffer: Vec::new(),
        unit_conversion_context,
    }));

    info!("Starting the runner");
    runner.run().await;

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    run().await;
}
