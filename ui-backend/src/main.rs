use crate::simulation_service::SimulationService;
use proto::frontend::{
    AuthenticationServiceServer, SensorCrudServiceServer, SimulationInterfaceServiceServer,
    TwinServiceServer,
};
use tonic::transport::Server;

use server::MyAuthenticationService;
use std::env;

// sqlx
use sqlx::postgres::PgPool;

pub mod error;
mod hashing;
mod jwt;
mod server;
mod simulation_service;
use crate::sensor::SensorStore;
use tracing::info;
mod sensor;
mod twin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();

    dotenvy::dotenv().ok();

    // Create database connection with provided environment variables
    let database_url = database_config::database_url("ui_backend");
    let pool = PgPool::connect(&database_url).await?;

    let addr = env::var("UI_BACKEND_ADDR")
        .unwrap_or("127.0.0.1:8080".to_string())
        .parse()
        .expect("A valid bind address");

    let twin_service = TwinServiceServer::new(twin::MyTwinService::new(pool.clone()));
    let simulation_service = SimulationInterfaceServiceServer::new(SimulationService::new().await);
    let sensor_crud_service = SensorCrudServiceServer::new(SensorStore::new().await);
    let authentication_service =
        AuthenticationServiceServer::new(MyAuthenticationService::new(pool.clone()));

    info!("Listening on {addr}");

    Server::builder()
        .add_service(simulation_service)
        .add_service(twin_service)
        .add_service(sensor_crud_service)
        .add_service(authentication_service)
        .serve(addr)
        .await?;
    Ok(())
}
