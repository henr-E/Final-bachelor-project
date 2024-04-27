use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use futures::stream::FuturesUnordered;
use futures::StreamExt;
use prost_types::value::Kind;
use prost_types::Value as ProstValue;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use tonic::{Request, Response, Status};

use proto::frontend::proto_twin::{
    BuildingObject, CreateTwinRequest, CreateTwinResponse, DeleteBuildingRequest,
    DeleteTwinRequest, GetAllTwinsRequest, GetAllTwinsResponse, GetBuildingsRequest,
    GetBuildingsResponse, TwinObject, UndoDeleteBuildingRequest,
};
use proto::frontend::{DeleteSensorRequest, GetSensorsRequest, TwinId, TwinService};

use proto::frontend::DeleteSimulationRequest as DeleteSimulationRequestFrontend;

use crate::sensor::SensorStore;
use crate::simulation_service::SimulationService;

pub struct MyTwinService {
    pool: PgPool,
    simulation_service: SimulationService,
    sensor_service: SensorStore,
}

impl MyTwinService {
    pub fn new(
        pool: PgPool,
        simulation_service: SimulationService,
        sensor_service: SensorStore,
    ) -> Self {
        Self {
            pool,
            simulation_service,
            sensor_service,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Twin {
    name: String,
    longitude: f64,
    latitude: f64,
    //radius in meters
    radius: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Building {
    street: String,
    house_number: String,
    postcode: String,
    city: String,
    // A list of coordinate pairs (longitude, latitude).
    coordinates: Vec<(f64, f64)>,
    visible: bool,
}

async fn request_and_process_twin_data(
    twin_id: i32,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    bottom_left: (f64, f64),
    top_right: (f64, f64),
) -> Result<(), Status> {
    // Request the overpass api for the chunk of data.
    let response = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .unwrap()
        .post("https://overpass-api.de/api/interpreter")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!(
            "\
                data=[out:json];\
                (\
                    node[\"building\"]({},{},{},{}); \
                    way[\"building\"]({},{},{},{});\
                    relation[\"building\"]({},{},{},{});\
                );\
                out body;>;out skel qt;\
            ",
            bottom_left.0,
            bottom_left.1,
            top_right.0,
            top_right.1,
            bottom_left.0,
            bottom_left.1,
            top_right.0,
            top_right.1,
            bottom_left.0,
            bottom_left.1,
            top_right.0,
            top_right.1,
        ))
        .send()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    if !response.status().is_success() {
        return Err(Status::internal(format!(
            "Failed to get data from OSM: HTTP {}",
            response.status(),
        )));
    }

    let response_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let mut buildings = Vec::new();
    let mut node_map: HashMap<i64, Vec<f64>> = HashMap::new();
    let mut default_city = String::new();
    let mut default_postcode = String::new();

    if let Some(elements) = response_data["elements"].as_array() {
        for element in elements {
            if element["type"] == "node" {
                let id = element["id"].as_i64().unwrap();
                let lat = element["lat"].as_f64().unwrap();
                let lon = element["lon"].as_f64().unwrap();
                node_map.insert(id, vec![lat, lon]);
            }
        }
        for element in elements {
            if let Some(tags) = element["tags"].as_object() {
                if tags.contains_key("addr:city") && default_city.is_empty() {
                    default_city = tags["addr:city"].as_str().unwrap_or_default().to_string();
                }
                if tags.contains_key("addr:postcode") && default_postcode.is_empty() {
                    default_postcode = tags["addr:postcode"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                }
                // Break early if both defaults are found
                if !default_city.is_empty() && !default_postcode.is_empty() {
                    break;
                }
            }
        }

        for element in elements {
            if let Some("way") = element["type"].as_str() {
                let nodes = element["nodes"].as_array().unwrap();
                let coordinates = nodes
                    .iter()
                    .filter_map(|node_id| node_map.get(&node_id.as_i64().unwrap()))
                    .map(|coords| (coords[0], coords[1]))
                    .collect::<Vec<(f64, f64)>>();

                let street = element["tags"]["addr:street"]
                    .as_str()
                    .unwrap_or("no street")
                    .to_string();
                let house_number = element["tags"]["addr:housenumber"]
                    .as_str()
                    .unwrap_or("no house number")
                    .to_string();
                let mut postcode = element["tags"]["addr:postcode"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                let mut city = element["tags"]["addr:city"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                let visible = true;

                if postcode.is_empty() {
                    postcode = default_postcode.clone()
                }
                if city.is_empty() {
                    city = default_city.clone()
                }
                buildings.push(Building {
                    street,
                    house_number,
                    postcode,
                    city,
                    coordinates,
                    visible,
                });
            }
        }
    }

    for building in buildings.iter() {
        // Transform coordinates from Vec<(f64, f64)> to Vec<Vec<f64>>
        let coordinates_transformed = building
            .coordinates
            .iter()
            .map(|&(lat, lon)| vec![lat, lon])
            .collect::<Vec<Vec<f64>>>();

        sqlx::query!("INSERT INTO buildings (street, house_number, postcode, city, coordinates, visible, twin_id) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                    building.street,
                    building.house_number,
                    building.postcode,
                    building.city,
                    json!(&coordinates_transformed), // Use the transformed coordinates for insertion
                    building.visible,
                    twin_id
                )
            .execute(transaction.as_mut())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
    }

    Ok(())
}

#[tonic::async_trait]
impl TwinService for MyTwinService {
    async fn create_twin(
        &self,
        request: Request<CreateTwinRequest>,
    ) -> Result<Response<CreateTwinResponse>, Status> {
        let req = request.into_inner();

        //add twin to the database
        let twin = Twin {
            name: req.name,
            latitude: req.latitude,
            longitude: req.longitude,
            radius: req.radius,
        };

        //calculate creation time
        let creation_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let twin_id = sqlx::query!("INSERT INTO twins (name, longitude, latitude, radius, creation_date_time) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            twin.name,
            twin.longitude,
            twin.latitude,
            twin.radius,
            creation_time.as_secs() as i32
        )
            .fetch_one(&self.pool)
            .await
            .map_err(|err| Status::from_error(Box::new(err)))?
            .id;

        let radius_km = twin.radius / 1000.0;
        // Calculate degrees of latitude per kilometer
        let delta_lat = 1.0 / 111.0;
        // Calculate degrees of longitude per kilometer at the given latitude
        // Make sure to specify the type for numerical literals and perform operations on the correct type
        let delta_lon = 1.0 / (111.0 * twin.latitude.to_radians().cos());

        //todo this is the code to divide 1 api request to overpass into 2 (a left part and a right part)
        // the database team will make these requests happen at the same time

        // Chunk the request per square kilometer.

        // Get the bounding box of the area.
        let bottom_left = (
            twin.latitude - delta_lat * radius_km,
            twin.longitude - delta_lon * radius_km,
        );
        let top_right = (
            twin.latitude + delta_lat * radius_km,
            twin.longitude + delta_lon * radius_km,
        );

        dbg!(bottom_left, top_right);

        let requests = FuturesUnordered::new();
        let mut current_bottom_left = bottom_left;
        while current_bottom_left.0 < top_right.0 {
            while current_bottom_left.1 < top_right.1 {
                let mut transaction = self
                    .pool
                    .begin()
                    .await
                    .expect("could not create transaction");
                requests.push(tokio::spawn(async move {
                    // Create a request for this chunk only.
                    request_and_process_twin_data(
                        twin_id,
                        &mut transaction,
                        (current_bottom_left.0, current_bottom_left.1),
                        (
                            (current_bottom_left.0 + delta_lat).min(top_right.0),
                            (current_bottom_left.1 + delta_lon).min(top_right.1),
                        ),
                    )
                    .await?;

                    transaction.commit().await.map_err(|e| {
                        Status::internal(format!("transaction failed to commit: {}", e))
                    })
                }));
                current_bottom_left.1 += delta_lon;
            }
            current_bottom_left.1 = bottom_left.1;
            current_bottom_left.0 += delta_lat;
            // dbg!(current_bottom_left);
        }
        // Make sure all requests are resolved.
        let results = requests.collect::<Vec<Result<_, _>>>().await;

        if let Some(err) = results.into_iter().find_map(|r| match r {
            Ok(Ok(_)) => None,
            Ok(Err(err)) => Some(err),
            Err(err) => Some(Status::internal(format!("join error: {}", err))),
        }) {
            return Err(Status::internal(err.to_string()));
        }

        Ok(Response::new(CreateTwinResponse {
            id: twin_id,
            creation_date_time: creation_time.as_secs() as i32,
        }))
    }

    async fn get_all_twins(
        &self,
        _request: Request<GetAllTwinsRequest>,
    ) -> Result<Response<GetAllTwinsResponse>, Status> {
        let twins_result = sqlx::query!(
            "SELECT id, name, longitude, latitude, radius, creation_date_time FROM twins"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Status::internal(format!("Failed to fetch twins: {}", e)))?;

        let mut twins = Vec::new();

        for twin in twins_result {
            let simulation_amount = sqlx::query!(
                "SELECT COUNT(*) as count FROM simulations WHERE twin_id = $1",
                twin.id
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                Status::internal(format!(
                    "Failed to count simulations for twin_id {}: {}",
                    twin.id, e
                ))
            })?
            .count
            .unwrap_or(0);

            twins.push(TwinObject {
                id: twin.id,
                name: twin.name,
                longitude: twin.longitude,
                latitude: twin.latitude,
                radius: twin.radius,
                creation_date_time: twin.creation_date_time,
                simulation_amount,
            });
        }

        Ok(Response::new(GetAllTwinsResponse { twins }))
    }

    async fn get_buildings(
        &self,
        request: Request<GetBuildingsRequest>,
    ) -> Result<Response<GetBuildingsResponse>, Status> {
        let twin_id = request.into_inner().id;
        let buildings_result = sqlx::query!(
        "SELECT id, street, house_number, postcode, city, coordinates, visible FROM buildings WHERE twin_id = $1",
        twin_id as i32
    )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Failed to fetch buildings: {}", e)))?;

        let mut buildings: Vec<BuildingObject> = Vec::new();
        for b in buildings_result {
            let coordinates_serde_value = b.coordinates.unwrap_or_else(|| json!([]));
            let coordinates: Vec<Vec<f64>> =
                serde_json::from_value(coordinates_serde_value).unwrap();

            let coordinates_value = ProstValue {
                kind: Some(Kind::ListValue(prost_types::ListValue {
                    values: coordinates
                        .into_iter()
                        .map(|pair| ProstValue {
                            kind: Some(Kind::ListValue(prost_types::ListValue {
                                values: pair
                                    .into_iter()
                                    .map(|coord| ProstValue {
                                        kind: Some(Kind::NumberValue(coord)),
                                    })
                                    .collect(),
                            })),
                        })
                        .collect(),
                })),
            };

            buildings.push(BuildingObject {
                id: b.id,
                street: b.street.unwrap_or_default(),
                house_number: b.house_number.unwrap_or_default(),
                postcode: b.postcode.unwrap_or_default(),
                city: b.city.unwrap_or_default(),
                coordinates: Some(coordinates_value),
                visible: b.visible.unwrap_or_default(),
            });
        }

        Ok(Response::new(GetBuildingsResponse { buildings }))
    }

    async fn delete_building(
        &self,
        request: Request<DeleteBuildingRequest>,
    ) -> Result<Response<()>, Status> {
        let building_id = request.into_inner().id;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        sqlx::query!(
            "UPDATE buildings SET visible = false WHERE id = $1",
            building_id as i64
        )
        .execute(&mut *transaction)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        transaction
            .commit()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }
    async fn undo_delete_building(
        &self,
        request: Request<UndoDeleteBuildingRequest>,
    ) -> Result<Response<()>, Status> {
        let building_id = request.into_inner().id;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        sqlx::query!(
            "UPDATE buildings SET visible = true WHERE id = $1",
            building_id as i64
        )
        .execute(&mut *transaction)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        transaction
            .commit()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn delete_twin(
        &self,
        request: Request<DeleteTwinRequest>,
    ) -> Result<Response<()>, Status> {
        use proto::frontend::SensorCrudService;
        use proto::frontend::SimulationInterfaceService;
        let twin_id = request.into_inner().id;

        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        //delete simulation
        let simulations = self
            .simulation_service
            .get_all_simulations(Request::new(TwinId {
                twin_id: twin_id.to_string(),
            }))
            .await?;
        for simulation in simulations.into_inner().item {
            self.simulation_service
                .delete_simulation(Request::new(DeleteSimulationRequestFrontend {
                    id: simulation.id,
                }))
                .await?;
        }

        // Delete all sensors associated with the twin
        let mut sensors = self
            .sensor_service
            .get_sensors(Request::new(GetSensorsRequest { twin_id }))
            .await?
            .into_inner();
        while let Some(sensor) = sensors.next().await {
            let get_sensor_request = sensor?;
            let id: String = get_sensor_request.sensor.unwrap().id;
            self.sensor_service
                .delete_sensor(Request::new(DeleteSensorRequest { uuid: id }))
                .await?;
        }

        // Delete the twin
        sqlx::query!("DELETE FROM twins WHERE id = $1", twin_id)
            .execute(&mut *transaction)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        transaction
            .commit()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }
}
