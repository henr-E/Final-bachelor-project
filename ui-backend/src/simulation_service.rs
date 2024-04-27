use futures::StreamExt;
use proto::frontend::{
    CreateSimulationParams, CreateSimulationResponse, Simulation, SimulationInterfaceService,
    Simulations, TwinId,
};

use proto::simulation::simulation_manager::{
    ComponentsInfo, PushSimulationRequest, SimulationData, SimulationFrame, SimulationFrameRequest,
    SimulationManagerClient, SimulatorSelection, Simulators,
};
use proto::simulation::{simulation_manager, State};
use sqlx::PgPool;
use std::env;
use std::ffi::c_double;

use std::time::{SystemTime, UNIX_EPOCH};

use std::str::FromStr;
use tonic::transport::Channel;
use tonic::{Request, Response, Status, Streaming};

#[derive(Debug)]
pub struct SimulationDB {
    twin_id: i32,
    name: String,
    start_date_time: i32,
    end_date_time: i32,
    creation_date_time: i32,
}
pub struct SimulationService {
    //TODO set in db
    pool: PgPool,
    client: SimulationManagerClient<Channel>,
}

impl SimulationService {
    pub async fn new(pool: PgPool) -> Self {
        Self {
            pool,
            client: SimulationManagerClient::new(
                tonic::transport::Channel::builder(
                    env::var("SIMULATION_MANAGER_ADDR")
                        .unwrap_or_else(|_| "http://127.0.0.1:8100".to_string())
                        .parse()
                        .expect("A valid simulation manager URI"),
                )
                .connect()
                .await
                .expect("Error could not connect to simulation manager"),
            ),
        }
    }
    async fn create_simulation_manager(
        &self,
        id: String,
        initial_state: Option<State>,
        timesteps: u64,
        timestep_delta: c_double,
        simulator_selection: SimulatorSelection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .clone()
            .push_simulation(PushSimulationRequest {
                id: Option::from(simulation_manager::SimulationId {
                    uuid: id.to_string(),
                }),
                initial_state,
                timesteps,
                timestep_delta,
                selection: Some(simulator_selection),
            })
            .await?;

        Ok(())
    }

    ///Get a simulation by id
    async fn get_simulation_manager(
        &self,
        id: String,
    ) -> Result<SimulationData, Box<dyn std::error::Error>> {
        // sending request and waiting for response
        Ok(self
            .client
            .clone()
            .get_simulation(simulation_manager::SimulationId {
                uuid: id.to_string(),
            })
            .await?
            .into_inner())
    }

    pub async fn _get_components_manager(
        &self,
    ) -> Result<Response<ComponentsInfo>, Box<dyn std::error::Error>> {
        // sending request and waiting for response
        let response = self.client.clone().get_components(()).await?;
        Ok(response)
    }
    pub async fn get_simulators_manager(&self) -> anyhow::Result<Response<Simulators>> {
        let response = self.client.clone().get_simulators(()).await?;
        Ok(response)
    }
}

///Frontend communication
#[tonic::async_trait]
impl SimulationInterfaceService for SimulationService {
    async fn create_simulation(
        &self,
        request: Request<CreateSimulationParams>,
    ) -> Result<Response<CreateSimulationResponse>, Status> {
        let req = request.into_inner();

        let creation_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let time_steps =
            ((req.end_date_time - req.start_date_time) as f64 / req.time_step_delta) as u64;

        let twin_id =
            i32::from_str(&req.twin_id).map_err(|err| Status::invalid_argument(err.to_string()))?;

        // create a simulation to insert into DB, note: twin_id is also stored
        let new_simulation = SimulationDB {
            twin_id,
            name: req.name,
            start_date_time: req.start_date_time, // Assuming these fields are provided correctly
            end_date_time: req.end_date_time,
            creation_date_time: creation_time.as_secs() as i32, // Assuming this is provided or generated here
        };

        let transaction = self
            .pool
            .begin()
            .await
            .map_err(|err| Status::from_error(Box::new(err)))?;

        let simulation_id = sqlx::query!(
            "INSERT INTO simulations (twin_id , name, start_date_time, end_date_time, creation_date_time) VALUES ($1,$2,$3,$4,$5) RETURNING id",
            new_simulation.twin_id,
            new_simulation.name,
            new_simulation.start_date_time,
            new_simulation.end_date_time,
            new_simulation.creation_date_time,
        )
            .fetch_one(&self.pool)
            .await
            .map_err(|err| Status::from_error(Box::new(err)))?
            .id;

        transaction
            .commit()
            .await
            .map_err(|err| Status::from_error(Box::new(err)))?;

        let success = self
            .create_simulation_manager(
                simulation_id.clone().to_string(),
                req.start_state,
                time_steps,
                req.time_step_delta,
                req.simulators.ok_or(Status::invalid_argument(
                    "CreateSimulationParams does not contain selected simulators names",
                ))?,
            )
            .await
            .is_ok();
        let response = CreateSimulationResponse {
            success,
            id: simulation_id,
        };

        Ok(Response::new(response))
    }

    async fn get_all_simulations(
        &self,
        request: Request<TwinId>,
    ) -> Result<Response<Simulations>, Status> {
        let req = request.into_inner();

        let twin_id =
            i32::from_str(&req.twin_id).map_err(|err| Status::invalid_argument(err.to_string()))?;

        // get all simulation matching the twin id in the request
        let items_database = sqlx::query!("SELECT * FROM simulations WHERE twin_id=$1", twin_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Failed to fetch simualations: {}", e)))?;

        // put all the found simulations in a vector
        let mut all_simulations: Vec<Simulation> = Vec::new();
        for item in items_database {
            let simulation_id = item.id.to_string();

            let simulation_item =
                self.get_simulation_manager(simulation_id)
                    .await
                    .map_err(|source| {
                        Status::internal(format!("Failed to get a simulation: {source}"))
                    })?;

            all_simulations.push(Simulation {
                id: item.id,
                name: item.name,
                start_date_time: item.start_date_time,
                end_date_time: item.end_date_time,
                creation_date_time: item.creation_date_time,
                frames_loaded: simulation_item.timestep_count as i32,
                status: simulation_item.status,
                status_info: simulation_item.status_info,
            });
        }

        let response = Simulations {
            item: all_simulations,
        };

        Ok(Response::new(response))
    }

    async fn get_simulation(
        &self,
        request: Request<simulation_manager::SimulationId>,
    ) -> Result<Response<Simulation>, Status> {
        let req = request.into_inner();

        let simulation_id: String = req.uuid;

        let database_simulation_id = i32::from_str(&simulation_id)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let simulation_item = self
            .get_simulation_manager(simulation_id.clone())
            .await
            .map_err(|err| Status::internal(format!("Failed to get a simulation: {err}")))?;

        // select the simulation that matches the uuid provided in the request
        let simulation_db = sqlx::query!(
            "SELECT * FROM simulations WHERE id=$1",
            database_simulation_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Status::internal(format!("Failed to fetch simualations: {}", e)));

        let item = match simulation_db {
            Ok(e) => e,
            Err(_) => return Err(Status::not_found("simulation not found")),
        };

        // create a simulation object to be wrapped in a response, note: no twin_id
        let simulation_found = Simulation {
            id: item.id,
            name: item.name,
            start_date_time: item.start_date_time,
            end_date_time: item.end_date_time,
            creation_date_time: item.creation_date_time,
            frames_loaded: simulation_item.timestep_count as i32,
            status: simulation_item.status,
            status_info: simulation_item.status_info,
        };

        Ok(Response::new(simulation_found))
    }

    type GetSimulationFramesStream = Streaming<SimulationFrame>;

    async fn get_simulation_frames(
        &self,
        request: Request<Streaming<SimulationFrameRequest>>,
    ) -> Result<Response<Self::GetSimulationFramesStream>, Status> {
        self.client
            .clone()
            .get_simulation_frames(request.into_inner().map(|frame| frame.unwrap()))
            .await
    }

    async fn get_components(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ComponentsInfo>, Status> {
        return Ok(self
            ._get_components_manager()
            .await
            .unwrap_or(Response::new(ComponentsInfo::default())));
    }

    async fn get_simulators(&self, _request: Request<()>) -> Result<Response<Simulators>, Status> {
        let result = match self.get_simulators_manager().await {
            Ok(response) => response,
            Err(err) => {
                let status = Status::internal(format!("Failed to fetch the simulators: {}", err));
                return Err(status);
            }
        };
        Ok(result)
    }
}
