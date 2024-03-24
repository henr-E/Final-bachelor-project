use std::env;
use std::net::SocketAddr;

// sqlx
use sqlx::postgres::PgPool;
use tokio::sync::mpsc;
// tonic
use tonic::transport::{Channel, Server};

// proto
use proto::simulation::simulation_manager::SimulationManagerServer;
use proto::simulation::simulator::simulator_client::SimulatorClient;
use runner::Runner;

use database_buffer::{DatabaseBuffer, Transport};

pub mod database;
mod database_buffer;
pub mod manager;
pub mod runner;

/// Main function that spawns runner and manager to handle requests for new simulations and
/// manage currently running simulations
///
/// First this function sets up its needed databse connection based on the values provided in .env
/// Then the runner is set up on a seperate thread
/// Lastly the manager will be setup on its own thread to handle requests from the frontend.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    // Create database connection with provided environment variables
    let database_url = database_config::database_url("simulation_manager");
    let pool = PgPool::connect(&database_url).await?;
    let pool_clone1 = pool.clone();
    let pool_clone2 = pool.clone();

    // Set up runner on a seperate thread
    // TODO: add simulators, change to mut
    let mut server_connections: Vec<SimulatorClient<Channel>> = Vec::default();
    server_connections.push(
        SimulatorClient::connect("http://127.0.0.1:8101")
            .await
            .unwrap(),
    );

    let server_connections_clone = server_connections.clone();

    let (notif_sender, notif_receiver) = mpsc::channel(1);
    let (state_sender, state_receiver) = mpsc::unbounded_channel::<Transport>();

    tokio::spawn(async move {
        let mut runner = Runner::new(
            pool_clone1,
            server_connections_clone,
            notif_receiver,
            state_sender,
        )
        .await;
        runner.start().await.unwrap();
    });

    tokio::spawn(async move {
        let database_buffer = DatabaseBuffer::new(pool_clone2, state_receiver).await;
        database_buffer.start().await.unwrap();
    });

    // Set up GRPC server listening on provided address or default localhost:8100
    let listen_addr = env::var("SIMULATOR_MANAGER_ADDR")
        .unwrap_or("127.0.0.1:8100".to_string())
        .parse::<SocketAddr>()?;
    let server = SimulationManagerServer::new(manager::Manager::new(
        pool.clone(),
        server_connections,
        notif_sender,
    ));
    Server::builder()
        .add_service(server)
        .serve(listen_addr)
        .await?;

    Ok(())
}
