use crate::{ingestor::DataIngestor, util::get_uploads_dir};
use database_config::database_url;
use environment_config::env;
use proto::sensor_data_ingest::DataIngestServiceServer;
use sensor_store::SensorStore;
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing::info;

mod error;
mod ingestor;
mod util;

pub const DEFAULT_UPLOADS_DIR: &str = "./uploads";
pub const UPLOADS_DIR_ENV_NAME: &str = "SENSOR_DATA_INGESTOR_UPLOADS_DIR";

/// Default port the application is run on.
const DEFAULT_PORT: u16 = 8084;

#[tokio::main]
async fn main() -> Result<(), crate::error::DataIngestError> {
    // Register a tracing subscriber that will print tracing events standard out.
    // The default log level is `INFO`. If needed increase to `DEBUG`, `TRACE` using
    // `with_max_level`.
    tracing_subscriber::fmt().init();

    // Initialize the uploads directory and get an early panic if any.
    let _ = get_uploads_dir().await;

    // Get connection url of the archival database.
    let database_url = database_url("sensor_archive");
    // Connect to the archival database.
    let pool = PgPool::connect(&database_url).await?;
    info!("successfully connected to the archival database.");

    // Create a socket address from environment. Use default if environment variable is not set.
    let socket_address = SocketAddr::from((
        [0, 0, 0, 0],
        env("SENSOR_DATA_INGESTOR_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(DEFAULT_PORT),
    ));

    let sensor_store = SensorStore::from_pg_pool(&pool);
    // Create the data ingestor service.
    let ingestor_service = DataIngestServiceServer::new(DataIngestor { pool, sensor_store });
    // Create a server that will host the service.
    info!("Server should now be listening at `{}`", socket_address);
    let server = tonic::transport::Server::builder().add_service(ingestor_service);
    // Run the server on the specified address.
    server.serve(socket_address).await?;

    Ok(())
}
