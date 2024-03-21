use crate::simulation_service::SimulationService;
use crate::twin::MyTwinService;
use proto::frontend::SimulationInterfaceServiceServer;
use proto::frontend::TwinServiceServer;
use std::env;
use tonic::transport::Server;
use tracing::info;

mod simulation_service;
mod twin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();

    dotenvy::dotenv().ok();

    let addr = env::var("UI_BACKEND_ADDR")
        .unwrap_or("127.0.0.1:8080".to_string())
        .parse()
        .expect("A valid bind address");

    let simulation_service = SimulationInterfaceServiceServer::new(SimulationService::new().await);
    let twin_service = TwinServiceServer::new(MyTwinService);

    info!("Listening on {addr}");

    Server::builder()
        .add_service(simulation_service)
        .add_service(twin_service)
        .serve(addr)
        .await?;
    Ok(())
}
