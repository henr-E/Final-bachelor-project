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
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;
    let pool_clone = pool.clone();

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

    tokio::spawn(async move {
        let mut runner = Runner::new(pool_clone, server_connections_clone, notif_receiver);
        let _err = runner.start().await;
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
