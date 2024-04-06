use crate::util::{get_uploads_dir, proto_file_format_to_parser_file_format};
use proto::sensor_data_ingest::{
    DataIngestService, ParseFailure, ParseFailureReason, ParseResult, SensorDataFile,
    SensorDataLines,
};
use sensor_data_validator::Validator;
use sensor_store::SensorStore;
use sqlx::PgPool;
use std::path::{Path, PathBuf};
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};
use tonic::{Request, Response, Status, Streaming};
use tracing::{debug, error, info, info_span, Instrument};
use uuid::Uuid;

/// Struct on which the service contract will be implemented. Can contain data when needed.
#[derive(Debug)]
pub struct DataIngestor {
    /// Connection to the postgresql database.
    pub pool: PgPool,
    /// Connection to the sensor store.
    pub sensor_store: SensorStore,
}

// Implement the service from gRPC contract on a struct.
#[tonic::async_trait]
impl DataIngestService for DataIngestor {
    async fn test_parse_sensor_data(
        &self,
        request: Request<SensorDataFile>,
    ) -> Result<Response<ParseResult>, Status> {
        // Generating an identifier for debugging streams.
        let identifier = DataIngestor::generate_identifier();
        let span = info_span!("", transaction_id=%identifier);

        async move {
            // Get the sensor data from the request.
            let sensor_data = request.into_inner();
            let Ok(sensor_id) = Uuid::try_parse(&sensor_data.sensor_id) else {
                return Ok(tonic::Response::new(
                    ParseFailure::new_string_detail(
                        ParseFailureReason::SensorIdInvalid,
                        "could not parse sensor id as uuid",
                    )
                    .into(),
                ));
            };
            let Ok(sensor) = self.sensor_store.get_sensor(sensor_id).await else {
                return Ok(tonic::Response::new(
                    ParseFailure::new_empty(ParseFailureReason::SensorIdNotFound).into(),
                ));
            };

            // Transform blobs of data into a more manageable binary JSON format. The contents of
            // the data has not been cleaned.
            debug!("converting sensor data to archival format");
            // Convert the protobuf FileFormat field into the
            // [`FileFormat`](sensor_data_parser::FileFormat) the parser can work with.
            let file_format = match proto_file_format_to_parser_file_format(sensor_data.file_format)
            {
                Ok(format) => format,
                Err(detail) => {
                    return Ok(tonic::Response::new(
                        ParseFailure::new_string_detail(
                            ParseFailureReason::DataFormatNotSupported,
                            detail,
                        )
                        .into(),
                    ))
                }
            };
            let sensor_data_bson =
                match sensor_data_parser::from_slice(&sensor_data.data, file_format) {
                    Ok(doc) => doc,
                    Err(err) => return Ok(tonic::Response::new(err.into())),
                };

            debug!("validating sensor data format");
            Ok(tonic::Response::new(
                Validator::from_signals(sensor.signals())
                    .validate(&bson::Bson::Document(sensor_data_bson)),
            ))
        }
        .instrument(span)
        .await
    }

    async fn ingest_sensor_data_file_stream(
        &self,
        request: Request<Streaming<SensorDataFile>>,
    ) -> Result<Response<ParseResult>, Status> {
        debug!("starting ingesting streamed sensor data.");

        // Start a span containing the identifier, so that all log entries from the same sensor
        // data all share the same transaction_id.
        let span_identifier = DataIngestor::generate_identifier();
        let span = info_span!("data_stream", transaction_id=%span_identifier.clone());

        // Take the stream from the request.
        let mut stream = request.into_inner();

        let mut sensor: Option<sensor_store::Sensor> = None;
        let mut failures = Vec::new();

        async {
            // Read all chunks of the sensor data stream and write the whole buffer into writer `file`.
            while let Some(sensor_data) = stream.message().await? {
                // Generating an identifier for debugging streams.

                let identifier = DataIngestor::generate_identifier();

                // Preparing the file where sensor data will be streamed to.
                let path = DataIngestor::get_path(identifier).await;
                let file = match File::create(&path).await {
                    Ok(f) => f,
                    Err(e) => {
                        failures.push(ParseFailure::new_string_detail(
                            ParseFailureReason::Unknown,
                            format!(
                                "Could not create file for sensor data (permission error?): {}",
                                e
                            ),
                        ));
                        continue;
                    }
                };
                let mut file = BufWriter::new(file);

                let Ok(sensor_id) = Uuid::try_parse(&sensor_data.sensor_id) else {
                    failures.push(ParseFailure::new_string_detail(
                        ParseFailureReason::SensorIdInvalid,
                        "could not parse sensor id as uuid",
                    ));
                    continue;
                };

                let sensor = match sensor {
                    Some(ref s) if s.id == sensor_id => s,
                    _ => {
                        let Ok(new_sensor) = self.sensor_store.get_sensor(sensor_id).await else {
                            failures.push(ParseFailure::new_empty(
                                ParseFailureReason::SensorIdNotFound,
                            ));
                            continue;
                        };
                        sensor = Some(new_sensor);
                        // SAFETY: `unwrap` is called directly after setting the `Some` variant.
                        sensor.as_ref().unwrap()
                    }
                };

                // Transform blobs of data into a more manageable binary JSON format. The contents of
                // the data has not been cleaned.
                debug!("converting sensor data to archival format");
                // Convert the protobuf FileFormat field into the
                // [`FileFormat`](sensor_data_parser::FileFormat) the parser can work with.
                let file_format =
                    match proto_file_format_to_parser_file_format(sensor_data.file_format) {
                        Ok(format) => format,
                        Err(detail) => {
                            failures.push(ParseFailure::new_string_detail(
                                ParseFailureReason::DataFormatNotSupported,
                                detail,
                            ));
                            continue;
                        }
                    };
                let sensor_data_bson =
                    match sensor_data_parser::from_slice(&sensor_data.data, file_format) {
                        Ok(doc) => doc,
                        Err(err) => {
                            failures.push(err);
                            continue;
                        }
                    };

                debug!("validating sensor data format");
                let parse_result = Validator::from_signals(sensor.signals())
                    .validate_from_document(&sensor_data_bson);
                if !parse_result.ok() {
                    failures.extend(parse_result.failures.into_iter());
                    continue;
                }

                // Write the vector of bytes to the buffered file writer.
                // TODO: `bson` does not support the `tokio::io::BufWriter`, but can write directly to
                // the `std::io::BufWriter`. Should we use the latter then to avoid the string
                // allocation?
                // TODO: Convert this to use `to_vec` when sprint review is over.
                file.write_all(sensor_data_bson.to_string().as_bytes())
                    .await?;

                // Explicit flush of all buffered contents to disk.
                file.flush().await?;
                info!("successfully flushed content to disk.");

                // Register the sensor data into the archive database and handle any error.
                if let Err(e) = self
                    .register_to_database(identifier, sensor.id, &path)
                    .await
                {
                    error!("Insertion of sensor data file failed!: {e}");
                    return Err(tonic::Status::from_error(e.into()));
                };
            }
            Ok(())
        }
        .instrument(span)
        .await?;

        if !failures.is_empty() {
            return Ok(tonic::Response::new(ParseResult { failures }));
        }
        info!("successfully inserted the sensor data file path into the archive database.");
        Ok(tonic::Response::new(ParseResult::new_ok()))
    }

    async fn ingest_sensor_data_stream(
        &self,
        _request: Request<Streaming<SensorDataLines>>,
    ) -> Result<Response<ParseResult>, Status> {
        unimplemented!("sensor data streams are not supported yet");
    }
}

impl DataIngestor {
    /// Create an unique identifier to identify the current session.
    fn generate_identifier() -> Uuid {
        Uuid::now_v7()
    }

    /// Get the absolute path of a sensor data file.
    async fn get_path(identifier: Uuid) -> PathBuf {
        get_uploads_dir().await.join(identifier.to_string())
    }

    /// Register a sensor data file to the archival database.
    ///
    /// # Errors
    ///
    /// * If `identifier` is longer than 26 characters, the query will fail on the side of the
    /// database.
    /// * If `path` doesn't point to a file on the server's filesystem, we will return an error.
    ///
    /// # Example
    ///
    /// ```
    /// // ulid is a string of 26 characters. If it is any longer the query will fail.
    /// // abs_path should be a valid Path. If this path doesn't exist, we will return an io error.
    /// register_to_database(&ulid, &abs_path);
    /// ```
    async fn register_to_database(
        &self,
        identifier: Uuid,
        sensor_id: Uuid,
        path: &Path,
    ) -> Result<(), crate::error::DatabaseRegisterError> {
        // check if the file is indeed a valid file path on disk.
        if !path.is_file() {
            return Err(std::io::Error::from(std::io::ErrorKind::NotFound).into());
        }

        // Insert the values in the archive database.
        sqlx::query!(
            "
                INSERT INTO archive_sensor_data_files (identifier, time, path, metadata, sensor_id)
                VALUES ($1::uuid, now()::timestamp, $2::text, $3::text, $4::uuid)
            ",
            identifier,
            path.to_str().unwrap(),
            "",
            sensor_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
