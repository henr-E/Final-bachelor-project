use futures::future::join_all;
use proto::frontend::{
    CreateSimulationParams, CreateSimulationResponse, Simulation, SimulationInterfaceService,
    Simulations, TwinId,
};
use proto::simulation;
use proto::simulation::simulation_manager::{
    ComponentsInfo, PushSimulationRequest, SimulationData, SimulationManagerClient,
};
use proto::simulation::{simulation_manager, Graph};
use std::ffi::c_double;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};
use tracing::debug;
use uuid::Uuid;

pub struct SimulationService {
    simulation_items: Arc<Mutex<Vec<Simulation>>>,
    client: SimulationManagerClient<Channel>,
}

impl SimulationService {
    pub async fn new() -> Self {
        Self {
            simulation_items: Arc::new(Mutex::new(Vec::new())),
            client: SimulationManagerClient::new(
                tonic::transport::Channel::from_static("http://127.0.0.1:8100")
                    .connect()
                    .await
                    .expect("Error could not connect to simulation manager"),
            ),
        }
    }
    async fn create_simulation_manager(
        &self,
        id: String,
        graph: Option<Graph>,
        timesteps: u64,
        timestep_delta: c_double,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .clone()
            .push_simulation(PushSimulationRequest {
                id: Option::from(simulation_manager::SimulationId {
                    uuid: id.to_string(),
                }),
                initial_state: Option::from(simulation::State {
                    graph,
                    global_components: Default::default(),
                }),
                timesteps,
                timestep_delta,
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
}

///Frontend communication
#[tonic::async_trait]
impl SimulationInterfaceService for SimulationService {
    async fn create_simulation(
        &self,
        request: Request<CreateSimulationParams>,
    ) -> Result<Response<CreateSimulationResponse>, Status> {
        let req = request.into_inner();

        //add GRPC request for creating simulationxvc
        debug!(
            "new simulation added {}, {}, {}, {}",
            req.name, req.start_date_time, req.end_date_time, req.twin_id
        );

        let creation_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let id: String = Uuid::new_v4().to_string();

        let time_steps =
            ((req.end_date_time - req.start_date_time) as f64 / req.time_step_delta) as u64;

        let success = self
            .create_simulation_manager(id.clone(), req.graph, time_steps, req.time_step_delta)
            .await
            .is_ok();
        let response = CreateSimulationResponse { success };

        let new_simulation = Simulation {
            name: req.name,
            id: id.clone(),
            start_date_time: req.start_date_time, // Assuming these fields are provided correctly
            end_date_time: req.end_date_time,
            creation_date_time: creation_time.as_secs() as i32, // Assuming this is provided or generated here
            frames_loaded: 0, // Assuming this is calculated or provided
            status: 0,        // Assuming this is set correctly here
        };

        let mut simulations = self.simulation_items.lock().await;
        simulations.push(new_simulation);

        Ok(Response::new(response))
    }

    async fn get_all_simulations(
        &self,
        request: Request<TwinId>,
    ) -> Result<Response<Simulations>, Status> {
        let req = request.into_inner();

        debug!("get all simulations {}", req.twin_id);

        let items = join_all(
            self.simulation_items
                .lock()
                .await
                .iter()
                .map(move |item| async {
                    let id: String = item.id.to_string();
                    let simulation_item = self
                        .get_simulation_manager(id)
                        .await
                        .expect("Failed to get a simulation");

                    let mut item = item.clone();
                    item.status = simulation_item.status;
                    item.frames_loaded = simulation_item.timestep_count as i32;
                    item
                }),
        )
        .await;

        let response = Simulations { item: items };
        Ok(Response::new(response))
    }

    //TODO implement this function
    async fn get_simulation(
        &self,
        request: Request<simulation_manager::SimulationId>,
    ) -> Result<Response<Simulation>, Status> {
        let req = request.into_inner();

        let name = req.uuid;

        debug!("returning simulation {}", name);

        let response = Simulation {
            name: String::from("test name"),
            id: String::from("test id"),
            start_date_time: 0,
            end_date_time: 0,
            creation_date_time: 0,
            frames_loaded: 0,
            status: 0,
        };

        Ok(Response::new(response))
    }
}
