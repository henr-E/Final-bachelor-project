pub mod string_service {
    tonic::include_proto!("users.string_list");
}
pub mod map_data_service {
    tonic::include_proto!("users.map_data");
}

// Re-export the necessary items to make them easily accessible
pub use map_data_service::map_data_service_server::{MapDataService, MapDataServiceServer};
pub use string_service::string_service_server::{StringService, StringServiceServer};

pub use map_data_service::MapDataResponse;
pub use string_service::ListStringsResponse;
