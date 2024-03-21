use std::env;

// main.rs
use tonic::transport::Server;

use crate::server::MyAuthenticationService;
use proto::auth::AuthenticationServiceServer;

pub mod filehandling;
pub mod hashing;
pub mod jwt;
pub mod myerror;

mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();

    dotenvy::dotenv().ok();

    let addr = env::var("USER_AUTHENTICATION_ADDR")
        .unwrap_or("127.0.0.1:8080".to_string())
        .parse()
        .expect("A valid bind address");

    let authentication_service = AuthenticationServiceServer::new(MyAuthenticationService {});

    Server::builder()
        .add_service(authentication_service)
        .serve(addr)
        .await?;
    Ok(())
}
