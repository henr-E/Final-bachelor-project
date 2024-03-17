use std::net::SocketAddr;
use tonic::{transport::Server};
use proto::frontend::{SimulationInterfaceServiceServer, TwinServiceServer};
use crate::simulation_service::SimulationService;
mod simulation_service;
use crate::twin::MyTwinService;
mod twin;

const PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();

    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));

    let simulation_service = SimulationInterfaceServiceServer::new(SimulationService::new().await);
    let twin_service = TwinServiceServer::new(MyTwinService);

    Server::builder()
        .add_service(simulation_service)
        .add_service(twin_service)
        .serve(addr)
        .await?;
    Ok(())
}
