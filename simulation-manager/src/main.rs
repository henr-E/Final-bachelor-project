use anyhow::Context;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

// imports
use connector::SimulatorConnector;
use database_buffer::{DatabaseBuffer, Transport};
use proto::simulation::{
    simulation_manager::SimulationManagerServer, simulator::simulator_client::SimulatorClient,
    simulator_connection::SimulatorConnectionServer,
};
use runner::Runner;
use sqlx::postgres::PgPool;
use tokio::sync::{mpsc, Mutex};
use tonic::transport::{Channel, Server};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

// modules
pub mod connector;
pub mod database;
mod database_buffer;
pub mod manager;
pub mod runner;

/// Main function that spawns runner and manager to handle requests for new simulations and
/// manage currently running simulations
///
/// First this function sets up its needed database connection based on the values provided in .env
/// Then the runner is set up on a separate thread
/// Lastly the manager will be setup on its own thread to handle requests from the frontend.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Create database connection with provided environment variables
    let database_url = database_config::database_url("simulation_manager");
    let pool = PgPool::connect(&database_url).await?;
    let pool_clone1 = pool.clone();
    let pool_clone2 = pool.clone();

    // Initialize vector with simulator connections
    // Simulators are automatically connected when calling start and providing the connector address (in env)
    let server_connections: Arc<Mutex<Vec<SimulatorClient<Channel>>>> =
        Arc::new(Mutex::new(Vec::default()));
    let server_connections_clone = server_connections.clone();

    // Set up GRPC server for simulators to connect to, listening on provided address or default localhost:8099
    let connector_listen_addr = env::var("SIMULATOR_CONNECTOR_ADDR")
        .unwrap_or("127.0.0.1:8099".to_string())
        .parse::<SocketAddr>()?;

    let connector = SimulatorConnector::new(server_connections.clone());
    let connector_server = SimulatorConnectionServer::new(connector);

    // mpsc channel
    let (notif_sender, notif_receiver) = mpsc::channel(1);
    let (state_sender, state_receiver) = mpsc::unbounded_channel::<Transport>();

    // Set up GRPC server listening on provided address or default localhost:8100
    let listen_addr = env::var("SIMULATOR_MANAGER_ADDR")
        .unwrap_or("127.0.0.1:8100".to_string())
        .parse::<SocketAddr>()?;
    info!("Listening on {listen_addr}");

    let manager =
        manager::Manager::new(pool.clone(), server_connections.clone(), notif_sender).await;
    let server = SimulationManagerServer::new(manager);

    // Set up simulation runner
    let mut runner = Runner::new(
        pool_clone1,
        server_connections_clone,
        notif_receiver,
        state_sender,
    )
    .await
    .context("Failed to set up the runner")?;

    // Database thread
    let task1 = tokio::spawn(async move {
        let database_buffer = DatabaseBuffer::new(pool_clone2, state_receiver).await;
        database_buffer.start().await
    });

    // Connector thread
    let task2 = tokio::spawn(
        Server::builder()
            .add_service(connector_server)
            .serve(connector_listen_addr),
    );

    // Runner thread
    let task3 = tokio::spawn(async move {
        // Infinitely loop in order to retry if the runner encounters an error.
        loop {
            if let Err(err) = runner.start().await {
                error!("Error encountered in runner: {err:?}");
            };
        }
    });

    // Manager
    let task4 = Server::builder().add_service(server).serve(listen_addr);

    // Handle errors
    tokio::select! {
        err = task1 => { err??; },
        err = task2 => { err??; },
        err = task3 => { err?; },
        err = task4 => { err?; },
    }

    Ok(())
}
