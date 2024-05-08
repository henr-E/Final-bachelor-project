use crate::simulation_service::SimulationService;
use proto::frontend::{
    sensor_data_fetching::SensorDataFetchingServiceServer, AuthenticationServiceServer,
    SensorCrudServiceServer, SimulationInterfaceServiceServer, TwinServiceServer,
};

use tonic::transport::Server;

use auth::MyAuthenticationService;
use std::env;

// sqlx
use sqlx::postgres::PgPool;

mod auth;
mod hashing;
mod jwt;
mod simulation_service;
use crate::auth::auth_interceptor;
use crate::sensor::SensorStore;
use tracing::info;

mod sensor;
mod twin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("ui_backend=DEBUG,INFO")
        .init();

    dotenvy::dotenv().ok();

    // Create database connection with provided environment variables
    let database_url = database_config::database_url("ui_backend");
    let pool = PgPool::connect(&database_url).await?;

    let addr = env::var("UI_BACKEND_ADDR")
        .unwrap_or("127.0.0.1:8080".to_string())
        .parse()
        .expect("A valid bind address");

    let simulation_service = SimulationService::new(pool.clone()).await;
    let simulation_service_server = SimulationInterfaceServiceServer::with_interceptor(
        simulation_service.clone(),
        auth_interceptor,
    );

    let sensor_crud_service = SensorStore::new().await;
    let sensor_crud_service_server =
        SensorCrudServiceServer::with_interceptor(sensor_crud_service.clone(), auth_interceptor);

    let sensor_data_fetching_service = SensorDataFetchingServiceServer::with_interceptor(
        sensor_crud_service.clone(),
        auth_interceptor,
    );

    let twin_service = TwinServiceServer::with_interceptor(
        twin::MyTwinService::new(pool.clone(), simulation_service, sensor_crud_service),
        auth_interceptor,
    );

    let authentication_service =
        AuthenticationServiceServer::new(MyAuthenticationService::new(pool.clone()));

    info!("Listening on {addr}");

    Server::builder()
        .add_service(simulation_service_server)
        .add_service(twin_service)
        .add_service(sensor_crud_service_server)
        .add_service(sensor_data_fetching_service)
        .add_service(authentication_service)
        .serve(addr)
        .await?;
    Ok(())
}
