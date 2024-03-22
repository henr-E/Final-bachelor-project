use proto::frontend::proto_twin::{
    CreateTwinRequest, CreateTwinResponse, GetAllTwinsRequest, GetAllTwinsResponse,
    GetBuildingsRequest, GetBuildingsResponse,
};
use proto::frontend::TwinService;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string, Value};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncWriteExt, BufWriter};
use tonic::{Request, Response, Status};

pub struct MyTwinService;

#[derive(Serialize, Deserialize, Clone)] // Makes Twin serializable, deserializable, and cloneable.
struct Twin {
    id: String,
    name: String,
    longitude: f64,
    latitude: f64,
    radius: i64,
}

async fn append_twin_to_file(data: &Twin, file_path: &str) -> Result<(), Status> {
    // Appends a twin to a file, creating or updating the file as needed.
    let mut twins = if Path::new(file_path).exists() {
        // Load existing twins from file, or start with empty list if any error.
        let content = fs::read_to_string(file_path)
            .await
            .map_err(|e| Status::internal(format!("Failed to read twins file: {}", e)))?;
        serde_json::from_str::<Vec<Twin>>(&content).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    };

    twins.push(data.clone()); // Add the new twin to the list.

    let twin_to_write = to_string(&twins)
        .map_err(|e| Status::internal(format!("Failed to serialize twins: {}", e)))?;

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
        .await
        .map_err(|e| Status::internal(format!("Failed to open file for twins: {}", e)))?;
    let mut writer = BufWriter::new(file); // Prepare the file writer.

    writer
        .write_all(twin_to_write.as_bytes())
        .await
        .map_err(|e| Status::internal(format!("Error writing to file for twins: {}", e)))?;
    writer
        .flush()
        .await
        .map_err(|e| Status::internal(format!("Error flushing file for twins: {}", e)))?;

    Ok(())
}

async fn transform_and_save_geojson(data: &str, file_path: &str) -> Result<(), Status> {
    // Converts API response to GeoJSON and saves it to a file.
    let api_response: Value = serde_json::from_str(data)
        .map_err(|e| Status::internal(format!("Failed to parse response data: {}", e)))?;

    let mut node_map: HashMap<i64, (f64, f64)> = HashMap::new(); // Maps node IDs to their lat-lon.
    let mut features: Vec<Value> = vec![]; // Will hold the GeoJSON features.

    if let Some(elements) = api_response["elements"].as_array() {
        for element in elements {
            // Populate node_map with nodes.
            if element["type"] == "node" {
                let id = element["id"].as_i64().unwrap();
                let lat = element["lat"].as_f64().unwrap();
                let lon = element["lon"].as_f64().unwrap();
                node_map.insert(id, (lat, lon));
            }
        }

        for element in elements {
            // Create GeoJSON features for ways using the nodes.
            if let Some("way") = element["type"].as_str() {
                let nodes = element["nodes"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|node_id| node_map.get(&node_id.as_i64().unwrap()).cloned())
                    .map(|(lat, lon)| vec![lon, lat])
                    .collect::<Vec<_>>();

                if !nodes.is_empty() {
                    features.push(json!({
                        "type": "Feature",
                        "geometry": {
                            "type": "Polygon",
                            "coordinates": [nodes],
                        },
                        "properties": element["tags"].clone(),
                    }));
                }
            }
        }
    }

    let geojson = json!({
        "type": "FeatureCollection",
        "features": features,
    }); // Construct the final GeoJSON object.

    let geojson_to_write = to_string(&geojson)
        .map_err(|e| Status::internal(format!("Failed to serialize GeoJSON: {}", e)))?;

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
        .await
        .map_err(|e| Status::internal(format!("Failed to open file for GeoJSON: {}", e)))?;
    let mut writer = BufWriter::new(file); // Prepare the file writer for GeoJSON.

    writer
        .write_all(geojson_to_write.as_bytes())
        .await
        .map_err(|e| Status::internal(format!("Error writing GeoJSON to file: {}", e)))?;
    writer
        .flush()
        .await
        .map_err(|e| Status::internal(format!("Error flushing GeoJSON file: {}", e)))?;

    Ok(())
}

#[tonic::async_trait] // Indicates an async trait (for gRPC service).
impl TwinService for MyTwinService {
    // Implementation of the TwinService trait.
    async fn create_twin(
        &self,
        request: Request<CreateTwinRequest>,
    ) -> Result<Response<CreateTwinResponse>, Status> {
        // Handles creation of a digital twin.

        let req = request.into_inner(); // Extracts the inner request object.

        let twin = Twin {
            // Constructs a Twin instance from the request.
            id: req.id,
            name: req.name,
            latitude: req.latitude,
            longitude: req.longitude,
            radius: req.radius,
        };

        let query = format!(
            // Builds the Overpass API query.
            "[out:json][timeout:25];(\
                    way(around:{radius},{lat},{lon})[\"building\"];\
                    relation(around:{radius},{lat},{lon})[\"building\"];\
            );out body; >; out skel qt;",
            radius = req.radius,
            lat = req.latitude,
            lon = req.longitude
        );

        let url = "https://overpass-api.de/api/interpreter"; // The Overpass API endpoint.
        let client = reqwest::Client::new(); // Instantiates a new HTTP client.
        let response = client
            .post(url) // Sends the query to the Overpass API.
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!("data={}", query))
            .send()
            .await;

        match response {
            Ok(res) if res.status().is_success() => {
                let response_data = res
                    .text()
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?; // Extracts response body.

                let dir_path = "ui-backend/temp-database/"; // Directory for saving files.
                fs::create_dir_all(dir_path).await.map_err(|e| {
                    Status::internal(format!("Failed to create directory for data: {}", e))
                })?;
                let geojson_file_path = format!("{}{}.buildings.geojson", dir_path, twin.id); // Path for the GeoJSON file.

                transform_and_save_geojson(&response_data, &geojson_file_path).await?; // Transforms and saves GeoJSON.

                let all_twins_file_path = "ui-backend/temp-database/all_twins.json"; // Path for the file with all twins.
                append_twin_to_file(&twin, all_twins_file_path).await?; // Appends the twin to the file.

                Ok(Response::new(CreateTwinResponse { created_twin: true }))
            }
            Ok(res) => Err(Status::internal(format!(
                "Failed to get data from OSM: HTTP {}",
                res.status()
            ))), // Handles non-success HTTP responses.
            Err(e) => {
                println!("TESTING");
                Err(Status::internal(e.to_string()))
            } // Handles request failures.
        }
    }

    async fn get_all_twins(
        &self,
        _request: Request<GetAllTwinsRequest>,
    ) -> Result<Response<GetAllTwinsResponse>, Status> {
        let file_path = "ui-backend/temp-database/all_twins.json"; // The file with all twins.

        let all_twins = match fs::read_to_string(file_path).await {
            Ok(content) => {
                if content.trim().is_empty() {
                    "".to_string()
                } else {
                    content
                }
            }
            Err(e) => {
                return Err(Status::internal(format!("Failed to read file: {:?}", e)));
                // Handles file read errors.
            }
        };

        let response = GetAllTwinsResponse { twins: all_twins };

        Ok(Response::new(response))
    }

    async fn get_buildings(
        &self,
        request: Request<GetBuildingsRequest>,
    ) -> Result<Response<GetBuildingsResponse>, Status> {
        // Returns buildings data for a specific twin.
        let req = request.into_inner(); // Extracts the inner request object.

        let id = req.id; // Twin ID.

        let file_path_basic = "ui-backend/temp-database/"; // Base directory for data files.
        let file_path = format!("{}{}.buildings.geojson", file_path_basic, id); // Specific file path.

        let buildings_data = match fs::read_to_string(file_path).await {
            Ok(content) => content, // Successfully reads the file content.
            Err(e) => {
                // Handles file read errors.
                return Err(Status::internal(format!("Failed to read file: {:?}", e)));
            }
        };

        let response = GetBuildingsResponse {
            buildings: buildings_data, // Sets the read content as the response.
        };

        Ok(Response::new(response)) // Returns the response.
    }
}
