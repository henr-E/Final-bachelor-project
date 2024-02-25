// Import Empty from prost_types instead of core::iter
use std::net::SocketAddr;
use tonic::{transport::Server, Request, Response, Status};

mod proto;

use proto::{ListStringsResponse, StringService, StringServiceServer};
use proto::{MapDataResponse, MapDataService, MapDataServiceServer};

const PORT: u16 = 8080;

struct MyStringService;
struct MyMapDataService;

#[tonic::async_trait]
impl StringService for MyStringService {
    async fn list_strings(
        &self,
        _request: Request<()>, // Note: The parameter name is prefixed with an underscore to indicate it's unused
    ) -> Result<Response<ListStringsResponse>, Status> {
        println!("list_strings method was called");
        let strings = vec![
            "string1".to_string(),
            "string2".to_string(),
            "string3".to_string(),
        ];
        Ok(Response::new(ListStringsResponse { strings }))
    }
}

#[tonic::async_trait]
impl MapDataService for MyMapDataService {
    async fn get_map_data(
        &self,
        _request: Request<()>,
    ) -> Result<Response<MapDataResponse>, Status> {
        println!("get_map_data method was called");
        let streets = vec!["Street 1".to_string(), "Street 2".to_string()];
        let houses = vec!["House A".to_string(), "House B".to_string()];
        Ok(Response::new(MapDataResponse { streets, houses }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));
    println!("StringService server listening on {}", addr);

    let string_service = StringServiceServer::new(MyStringService);
    let map_data_service = MapDataServiceServer::new(MyMapDataService);

    Server::builder()
        .add_service(string_service)
        .add_service(map_data_service)
        .serve(addr)
        .await?;

    Ok(())
}
