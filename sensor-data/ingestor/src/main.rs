use proto::sensor_data_ingest::{
    sensor_data::FileFormat as ProtoSensorDataFileFormat, DataIngestService,
    DataIngestServiceServer, ParseFailure, ParseFailureReason, ParseResult, SensorData,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{
    env,
    fmt::Debug,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
    sync::OnceCell,
};
use tonic::{Request, Response, Status, Streaming};
use tracing::{debug, error, info, info_span, Instrument};
use ulid::Ulid;

mod error;

// these defaults are taken from the default timescaledb docker container configuration.
const DEFAULT_DATABASE: &str = "sensor_archive";
const DEFAULT_HOST: &str = "localhost";
const DEFAULT_PASSWORD: &str = "password";
const DEFAULT_PORT: u16 = 8080;
const DEFAULT_USER: &str = "postgres";
const MAX_DB_CONNNECTIONS: u32 = 4;
const UPLOAD_DIR: &str = "uploads";

// Singleton to store once and then retrieve the database url.
static DATABASE_URL: OnceCell<String> = OnceCell::const_new();

/// Struct on which the service contract will be implemented. Can contain data when needed.
#[derive(Debug)]
struct DataIngestor {
    /// Connection to the postgresql database.
    pool: PgPool,
}

// Implement the service from gRPC contract on a struct.
#[tonic::async_trait]
impl DataIngestService for DataIngestor {
    async fn test_parse_sensor_data(
        &self,
        request: Request<SensorData>,
    ) -> Result<Response<ParseResult>, Status> {
        // Generating an identifier for debugging streams.
        let identifier = DataIngestor::generate_identifier();
        let span = info_span!("", transaction_id=%identifier);

        async move {
            // Preparing the file where sensor data will be streamed to.
            // The identifier will be used as filename.
            let path = DataIngestor::get_path(&identifier);
            let mut file = BufWriter::new(File::create(&path).await?);

            // Get the sensor data from the request.
            let sensor_data = request.into_inner();

            // Transform blobs of data into a more manageable binary JSON format. The contents of
            // the data has not been cleaned.
            debug!("converting sensor data to archival format");
            // Convert the protobuf FileFormat field into the
            // [`FileFormat`](sensor_data_parser::FileFormat) the parser can work with.
            let file_format = match proto_file_format_to_parser_file_format(sensor_data.file_format) {
                Ok(format) => format,
                Err(detail) => return Ok(tonic::Response::new(ParseFailure::new_string_detail(ParseFailureReason::DataFormatNotSupported, detail).into()))
            };
            let sensor_data_bson = match sensor_data_parser::from_slice(&sensor_data.data, file_format) {
                Ok(doc) => doc,
                Err(err) => return Ok(tonic::Response::new(err.into())),
            };

            debug!("starting ingesting of sensor data.");

            // Write the vector of bytes to the buffered file writer.
            // TODO: `bson` does not support the `tokio::io::BufWriter`, but can write directly to
            // the `std::io::BufWriter`. Should we use the latter then to avoid the string
            // allocation?
            // TODO: Convert this to use `to_vec` when sprint review is over.
            file.write_all(sensor_data_bson.to_string().as_bytes()).await?;

            // Explicit flush of all buffered contents to disk.
            file.flush().await?;
            debug!("successfully flushed content to disk.");

            // Register the sensor data into the archive database and handle any error.
            match self.register_to_database(&identifier, &path).await{
                Ok(()) => {
                    info!("successfully inserted the sensor data file path into the archive database.");
                    Ok(tonic::Response::new(ParseResult::default()))
                },
                Err(e) => {
                    error!("Insertion of sensor data file failed!: {e}");
                    Err(Status::invalid_argument(format!("Insertion query failed: {e}")))
                },
            }
        }.instrument(span).await
    }

    async fn ingest_sensor_data_stream(
        &self,
        request: Request<Streaming<SensorData>>,
    ) -> Result<Response<ParseResult>, Status> {
        // Generating an identifier for debugging streams.
        let identifier = DataIngestor::generate_identifier();
        // Start a span containing the identifier, so that all log entries from the same sensor
        // data all share the same transaction_id.
        let span = info_span!("", transaction_id=%identifier.clone());
        async move {
            // Take the stream from the request.
            let mut stream = request.into_inner();

            // Preparing the file where sensor data will be streamed to.
            let path =  DataIngestor::get_path(&identifier);
            let file = File::create(&path)
                .await
                .expect("The uploads directory is relative to the process's pwd. Please fix me by editing the UPLOAD_DIR constant.");
            let mut file = BufWriter::new(file);

            debug!("starting ingesting streamed sensor data.");

            // Read all chunks of the sensor data stream and write the whole buffer into writer `file`.
            // TODO: handle the errors that might be generated here.
            while let Some(sensor_data) = stream.message().await? {
                // Transform blobs of data into a more manageable binary JSON format. The contents of
                // the data has not been cleaned.
                debug!("converting sensor data to archival format");
                // Convert the protobuf FileFormat field into the
                // [`FileFormat`](sensor_data_parser::FileFormat) the parser can work with.
                let file_format = match proto_file_format_to_parser_file_format(sensor_data.file_format) {
                    Ok(format) => format,
                    Err(detail) => return Ok(tonic::Response::new(ParseFailure::new_string_detail(ParseFailureReason::DataFormatNotSupported, detail).into()))
                };
                let sensor_data_bson = match sensor_data_parser::from_slice(&sensor_data.data, file_format) {
                    Ok(doc) => doc,
                    Err(err) => return Ok(tonic::Response::new(err.into())),
                };

                // Write the vector of bytes to the buffered file writer.
                // TODO: `bson` does not support the `tokio::io::BufWriter`, but can write directly to
                // the `std::io::BufWriter`. Should we use the latter then to avoid the string
                // allocation?
                // TODO: Convert this to use `to_vec` when sprint review is over.
                file.write_all(sensor_data_bson.to_string().as_bytes()).await?;
            }

            // Explicit flush of all buffered contents to disk.
            file.flush().await?;
            info!("successfully flushed content to disk.");

            // Register the sensor data into the archive database and handle any error.
            match self.register_to_database(&identifier, &path).await {
                Ok(()) => {
                    info!("successfully inserted the sensor data file path into the archive database.");
                    Ok(tonic::Response::new(ParseResult::default()))
                },
                Err(e) => {
                    error!("Insertion of sensor data file failed!: {e}");
                    Err(Status::invalid_argument(format!("Insertion query failed: {e}")))
                },
            }
        }.instrument(span).await
    }
}

impl DataIngestor {
    /// Create an unique identifier to identify the current session.
    fn generate_identifier() -> String {
        Ulid::new().to_string()
    }

    /// Get the absolute path of a sensor data file.
    fn get_path(identifier: &str) -> PathBuf {
        env::current_dir()
            .unwrap()
            .join(UPLOAD_DIR)
            .join(identifier)
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
        identifier: &str,
        path: &Path,
    ) -> Result<(), crate::error::DatabaseRegisterError> {
        // check if the file is indeed a valid file path on disk.
        if !path.is_file() {
            return Err(std::io::Error::from(std::io::ErrorKind::NotFound).into());
        }

        // Insert the values in the archive database.
        sqlx::query!("INSERT INTO sensor_data_files (identifier, time, path, metadata) VALUES ($1::text, now()::timestamp, $2::text, $3::text);", identifier, path.to_str().unwrap(), "")
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

/// Converts the file format used in the protobuf definition to the file format used by the parser.
///
/// # Errors
///
/// If the fields used for the configuration of the file format are invalid (e.g., multiple bytes
/// as the CSV record delimiter).
fn proto_file_format_to_parser_file_format(
    file_format: Option<ProtoSensorDataFileFormat>,
) -> Result<sensor_data_parser::FileFormat, impl Into<String>> {
    Ok(match file_format {
        Some(ProtoSensorDataFileFormat::Csv(csv)) => {
            // Check if the delimiter is a single byte. If not return an appropriate
            // error to the user.
            if csv.delimiter.as_bytes().len() != 1 {
                return Err("csv delimiter must be a single byte");
            }

            sensor_data_parser::FileFormat::Csv {
                delimiter: csv.delimiter.as_bytes()[0],
            }
        }
        Some(ProtoSensorDataFileFormat::Json(_)) => sensor_data_parser::FileFormat::Json,
        None => {
            debug!("File format not supplied! Using CSV with `;` as the delimiter.");
            sensor_data_parser::FileFormat::Csv { delimiter: b';' }
        }
    })
}

/// Get or create the connection string for the archival database.
///
/// This function uses enviroment variables by default.
/// If one of them is not defines, constants are used.
async fn database_url() -> &'static str {
    // Set the database url once and return the value. Every following call, no string allocation
    // will be done, instead the previously calculated value is returned.
    &DATABASE_URL
        .get_or_init(|| async {
            // Check if the url is defined in the enviroment variables. This could be the case
            // since sqlx requires such an url to perform compiletime checks on the queries.
            if let Ok(url) = env::var("DATABASE_URL") {
                return url;
            }
            // Getting all configuration data ready to connect to the database.
            let user_name: String =
                env::var("SENSOR_DATA_INGESTOR_USER").unwrap_or(DEFAULT_USER.to_string());
            let password: String =
                env::var("SENSOR_DATA_INGESTOR_PASSWORD").unwrap_or(DEFAULT_PASSWORD.to_string());
            let host: String =
                env::var("SENSOR_DATA_INGESTOR_HOST").unwrap_or(DEFAULT_HOST.to_string());
            let database: String =
                env::var("SENSOR_DATA_INGESTOR_DATABASE").unwrap_or(DEFAULT_DATABASE.to_string());

            format!(
                "postgres://{}:{}@{}/{}",
                user_name, password, host, database
            )
        })
        .await
}

#[tokio::main]
async fn main() -> Result<(), crate::error::DataIngestError> {
    // Register a tracing subscriber that will print tracing events standard out.
    // The default log level is `INFO`. If needed increase to `DEBUG`, `TRACE` using
    // `with_max_level`.
    tracing_subscriber::fmt().init();

    // Load environment variables from .env file in this directory or up. If not present ignore the
    // error.
    dotenvy::dotenv().ok();

    // Get connection url of the archival database.
    let database_url = database_url().await;
    // Connect to the archival database.
    let pool = PgPoolOptions::new()
        .max_connections(MAX_DB_CONNNECTIONS)
        .connect(database_url)
        .await?;

    // Run database migrations and handle any errors that occur. The directory location is relative
    // to the crates `Cargo.toml`.
    sqlx::migrate!("../../migrations/sensor").run(&pool).await?;
    info!("successfully connected to the archival database and ran migrations.");

    // Create upload directory if it doesn't exist. We have to make sure that we always call the
    // the ingest service binary from the same directory, since the upload directory is dependent
    // on the pwd.
    //
    // If the user has incorrect permissions, panic.
    // If the directory already exists, continue.
    // Not handeling the parent path is fine since we know that the present working directory exists.
    if let Err(err) = std::fs::create_dir(std::env::current_dir().unwrap().join(UPLOAD_DIR)) {
        if err.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(err.into());
        }
    }

    // Create a socket address from environment. Use default if environment variable is not set.
    let socket_address = SocketAddr::from((
        [0, 0, 0, 0],
        std::env::var("SENSOR_DATA_INGESTOR_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(DEFAULT_PORT),
    ));

    // Create the data ingestor service.
    let ingestor_service = DataIngestServiceServer::new(DataIngestor { pool });
    // Create a server that will host the service.
    info!("Server should now be listening at `{}`", socket_address);
    let server = tonic::transport::Server::builder().add_service(ingestor_service);
    // Run the server on the specified address.
    server.serve(socket_address).await?;

    Ok(())
}