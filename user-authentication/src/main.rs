pub mod filehandling;
pub mod hashing;
pub mod jwt;
pub mod myerror;

mod server;

use std::net::SocketAddr;
// main.rs
use tonic::transport::Server;

use crate::server::MyAuthenticationService;
use proto::auth::AuthenticationServiceServer;

const PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));

    let authentication_service = AuthenticationServiceServer::new(MyAuthenticationService {});

    Server::builder()
        .add_service(authentication_service)
        .serve(addr)
        .await?;
    Ok(())
}
