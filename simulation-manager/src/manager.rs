use prost_types::value::Kind;
use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

use crate::connector::SimulatorsInfo;
use crate::database::StatusEnum::Failed;
use crate::database::{SimulationsDB, StatusEnum};
use proto::simulation::component_structure::ComponentStructure;
use proto::simulation::simulation_manager::DeleteSimulationRequest as DeleteSimulationRequestManager;
use proto::simulation::simulator::{IoConfigRequest, SimulatorClient};
use proto::simulation::{
    simulation_manager::{
        ComponentsInfo, PushSimulationRequest, SimulationData, SimulationFrame,
        SimulationFrameRequest, SimulationId, SimulationManager, SimulatorInfo, Simulators,
    },
    ComponentPrimitive, ComponentSpecification, Graph, State,
};

/// The Manager handles incoming requests from the frontend. It can return all known component types
/// at a certain time, queue new simulations, return info about a simulation and return the state of
/// a simulation at a requested timestep.
///
/// The manager holds a database connection, a vector of all known simulators and a sender for the
/// asynchronous channel created in main.rs. The sender is used to notify the runner that a new
/// simulation has been queued.
pub struct Manager {
    simulators: Arc<Mutex<Vec<SimulatorsInfo>>>,
    db: Arc<Mutex<SimulationsDB>>,
    notif_sender: mpsc::Sender<()>,
}

impl Manager {
    /// Create a new manager
    pub async fn new(
        pool: PgPool,
        simulators: Arc<Mutex<Vec<SimulatorsInfo>>>,
        notif_sender: mpsc::Sender<()>,
    ) -> Self {
        let db: SimulationsDB = SimulationsDB::from_pg_pool(pool).await.unwrap();
        let db: Arc<Mutex<SimulationsDB>> = Arc::new(Mutex::new(db));
        Self {
            simulators,
            db,
            notif_sender,
        }
    }

    async fn get_selected_simulators(
        &self,
        simulation_id: i32,
    ) -> anyhow::Result<Vec<SimulatorClient<Channel>>> {
        let simulators = self.simulators.lock().await;
        let mut db = self.db.lock().await;

        // get only selected simulators
        let selection = db
            .get_selected_simulators(simulation_id)
            .await?
            .unwrap_or(Vec::new());
        let selected = simulators
            .iter()
            .filter(|sim| selection.contains(&sim.name))
            .map(|sim| sim.simulator.clone())
            .collect::<Vec<_>>();
        drop(simulators);
        drop(db);
        Ok(selected)
    }

    async fn get_selected_components(&self, id: i32) -> Result<ComponentsInfo, Status> {
        let mut components: ComponentsInfo = ComponentsInfo::default();

        // clone simulator vec and drop mutex
        let mut simulators = self
            .get_selected_simulators(id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        for server in simulators.iter_mut() {
            let request = tonic::Request::new(IoConfigRequest {});
            let response = server.get_io_config(request).await?.into_inner();
            let response_components = response.components;
            for (key, value) in response_components {
                components.components.insert(key, value);
            }
        }
        Ok(components)
    }

    /// Unpacks ComponentStructure from ComponentSpecification given name and map of specifications for all components
    fn get_component_structure(
        components: &HashMap<String, ComponentSpecification>,
        component_name: &String,
    ) -> Option<ComponentStructure> {
        let component_spec = match components.get(component_name) {
            None => return None,
            Some(component) => component,
        };
        let component_structure = component_spec.clone().structure?.component_structure?;
        Some(component_structure)
    }

    /// Compares simulation.proto ComponentStructure to protobuf Value Kind
    fn compare_component_structure(expected: ComponentStructure, actual: Kind) -> bool {
        match (actual, expected) {
            (Kind::ListValue(a), ComponentStructure::List(e)) => {
                for value in a.values {
                    let Some(actual_structure) = value.kind else {
                        return false;
                    };
                    let Some(expected_structure) = e.component_structure.clone() else {
                        return false;
                    };
                    if !Manager::compare_component_structure(expected_structure, actual_structure) {
                        return false;
                    }
                }
                true
            }
            (Kind::StructValue(a), ComponentStructure::Struct(e)) => {
                for (item_name, actual_item_value) in a.fields {
                    let Some(actual_item_kind) = actual_item_value.kind else {
                        return false;
                    };
                    let Some(expected_item_value) = e.data.get(&item_name) else {
                        return false;
                    };
                    let Some(expected_item_structure) =
                        expected_item_value.clone().component_structure
                    else {
                        return false;
                    };
                    if !Manager::compare_component_structure(
                        expected_item_structure,
                        actual_item_kind,
                    ) {
                        return false;
                    }
                }
                true
            }
            (a, ComponentStructure::Primitive(e)) => {
                let Ok(e): Result<ComponentPrimitive, _> = (e).try_into() else {
                    return false;
                };
                matches!(
                    (a, e),
                    (Kind::NumberValue(_), ComponentPrimitive::F32)
                        | (Kind::NumberValue(_), ComponentPrimitive::F64)
                        | (Kind::NumberValue(_), ComponentPrimitive::U8)
                        | (Kind::NumberValue(_), ComponentPrimitive::U16)
                        | (Kind::NumberValue(_), ComponentPrimitive::U32)
                        | (Kind::NumberValue(_), ComponentPrimitive::U64)
                        | (Kind::NumberValue(_), ComponentPrimitive::U128)
                        | (Kind::NumberValue(_), ComponentPrimitive::I8)
                        | (Kind::NumberValue(_), ComponentPrimitive::I16)
                        | (Kind::NumberValue(_), ComponentPrimitive::I32)
                        | (Kind::NumberValue(_), ComponentPrimitive::I64)
                        | (Kind::NumberValue(_), ComponentPrimitive::I128)
                        | (Kind::StringValue(_), ComponentPrimitive::String)
                        | (Kind::BoolValue(_), ComponentPrimitive::Bool)
                )
            }
            _ => false,
        }
    }
}

#[tonic::async_trait]
impl SimulationManager for Manager {
    /// Return all currently known component types.
    ///
    /// This request is passed to the different simulators which then respond with their components.
    async fn get_components(&self, _: Request<()>) -> Result<Response<ComponentsInfo>, Status> {
        let mut components: ComponentsInfo = ComponentsInfo::default();

        // clone simulator vec and drop mutex
        let mut simulators = self.simulators.lock().await;

        for server in simulators.iter_mut() {
            let request = tonic::Request::new(IoConfigRequest {});
            let response = server.simulator.get_io_config(request).await?.into_inner();
            let response_components = response.components;
            for (key, value) in response_components {
                components.components.insert(key, value);
            }
        }
        drop(simulators);
        Ok(Response::new(components))
    }

    /// Delete an old simulation
    ///
    /// Based on the name of the given simulation all tables in the database will be cleared of the
    /// records that are coupled to this simulation.
    async fn delete_simulation(
        &self,
        request: Request<DeleteSimulationRequestManager>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let simulation_id = req.id.expect("Request should provide an ID.").uuid;

        // Start transaction
        let mut db = self.db.lock().await;
        db.begin_transaction().await.map_err(|err| {
            Status::internal(format!(
                "delete_simulation could not begin transaction with message: {:?}",
                err.to_string()
            ))
        })?;

        // Delete the simulation
        db.delete_simulation_via_name(&simulation_id)
            .await
            .map_err(|err| {
                Status::internal(format!(
                    "delete_simulation could not delete the simulation with message: {:?}",
                    err.to_string()
                ))
            })?;

        // Commit transaction
        db.commit().await.map_err(|err| {
            Status::internal(format!(
                "delete_simulation could not commit the deletion with message: {:?}",
                err.to_string()
            ))
        })?;
        Ok(Response::new(()))
    }

    /// Queue a new simulation
    ///
    /// The manager starts by decomposing the request into the needed components to populate the database
    /// with the simulations initial state.
    /// It then proceeds by adding the simulation to the database. Then the manager checks if the
    /// needed components are present in the database and if their values have the correct structure.
    /// If this is not the case the simulation will automatically be set to Failed.
    /// If all the required components are present and valid the manager will queue the simulation.
    /// It then places every needed component into the database at timestep 0.
    /// This is all done using a transaction so that it can be committed in one go.
    /// After this the manager will use the asynchronous channel to notify the runner that a new
    /// simulation has been queued.
    async fn push_simulation(
        &self,
        request: Request<PushSimulationRequest>,
    ) -> Result<Response<()>, Status> {
        let simulation = request.into_inner();
        let simulation_id = simulation.id.expect("Request should provide an ID.").uuid;
        let initial_state = simulation
            .initial_state
            .expect("Request should have an initial state.");
        let graph = initial_state
            .graph
            .expect("Initial state should have a graph.");
        let nodes = graph.nodes;
        let edges = graph.edge;

        let global = initial_state.global_components;
        let selection = simulation.selection.ok_or(Status::invalid_argument(
            "Invalid grpc, no selection present",
        ))?;
        let simulators = selection.name;
        // Start transaction
        let mut db = self.db.lock().await;
        db.begin_transaction().await.map_err(|err| {
            Status::internal(format!(
                "push_simulation could not begin transaction with message: {:?}",
                err.to_string()
            ))
        })?;

        let mut status = StatusEnum::Pending;
        if simulation.timesteps == 0 {
            status = StatusEnum::Finished;
        }
        //add a new simulation to the simulation table
        let simulation_index = db
            .add_simulation(
                simulation_id.as_str(),
                (simulation.timestep_delta * 1000.0) as i32,
                simulation.timesteps as i32,
                status,
                simulators,
            )
            .await
            .map_err(|err| {
                Status::internal(format!(
                    "push_simulation could not insert the simulation with message: {:?}",
                    err.to_string()
                ))
            })?;
        db.commit()
            .await
            .map_err(|err| Status::internal(err.to_string()))?;
        drop(db);

        // check if all components have valid structure
        let components = self
            .get_selected_components(simulation_index)
            .await?
            .components;
        db = self.db.lock().await;
        db.begin_transaction()
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        // check if all required node components are valid
        for node in &nodes {
            for (component_name, component_value) in &node.components {
                // get expected component structure from io_config
                let expected_structure =
                    match Manager::get_component_structure(&components, component_name) {
                        None => {
                            break;
                        }
                        Some(s) => s,
                    };

                // get actual structure
                let actual_structure = match component_value.clone().kind {
                    None => {
                        db.update_status(
                            simulation_index,
                            Failed,
                            "Component value has no Kind".into(),
                        )
                        .await
                        .map_err(|err| Status::internal(err.to_string()))?;
                        db.commit()
                            .await
                            .map_err(|err| Status::internal(err.to_string()))?;
                        return Ok(Response::new(()));
                    }
                    Some(s) => s,
                };

                // compare
                if !Manager::compare_component_structure(expected_structure, actual_structure) {
                    db.update_status(simulation_index, Failed, "Provided node component structures do not match structures expected by simulators".into()).await.map_err(|err| Status::internal(err.to_string()))?;
                    db.commit()
                        .await
                        .map_err(|err| Status::internal(err.to_string()))?;
                    return Ok(Response::new(()));
                }
            }
        }

        // check if all required edge components are valid
        for edge in &edges {
            // get edge component name and value
            let component_name = &edge.component_type;
            let component_value = &edge.component_data;

            // get expected structure
            let expected_structure =
                match Manager::get_component_structure(&components, component_name) {
                    None => {
                        continue;
                    }
                    Some(s) => s,
                };

            // get actual structure
            let actual_value = match component_value.clone() {
                None => {
                    db.update_status(simulation_index, Failed, "Component has no value".into())
                        .await
                        .map_err(|err| Status::internal(err.to_string()))?;
                    db.commit()
                        .await
                        .map_err(|err| Status::internal(err.to_string()))?;
                    return Ok(Response::new(()));
                }
                Some(s) => s,
            };

            let actual_structure = match actual_value.clone().kind {
                None => {
                    db.update_status(
                        simulation_index,
                        Failed,
                        "Component value has no Kind".into(),
                    )
                    .await
                    .map_err(|err| Status::internal(err.to_string()))?;
                    db.commit()
                        .await
                        .map_err(|err| Status::internal(err.to_string()))?;
                    return Ok(Response::new(()));
                }
                Some(s) => s,
            };

            // compare
            if !Manager::compare_component_structure(expected_structure, actual_structure) {
                db.update_status(simulation_index, Failed, "Provided edge component structures do not match structures expected by simulators".into()).await.map_err(|err| Status::internal(err.to_string()))?;
                db.commit()
                    .await
                    .map_err(|err| Status::internal(err.to_string()))?;
                return Ok(Response::new(()));
            }
        }

        // check if all global components are valid
        for (component_name, component_value) in &global.clone() {
            // get expected structure
            let expected_structure =
                match Manager::get_component_structure(&components, component_name) {
                    None => {
                        continue;
                    }
                    Some(s) => s,
                };

            // get actual structure
            let actual_structure = match component_value.clone().kind {
                None => {
                    db.update_status(
                        simulation_index,
                        Failed,
                        "Component value has no Kind".into(),
                    )
                    .await
                    .map_err(|err| Status::internal(err.to_string()))?;
                    db.commit()
                        .await
                        .map_err(|err| Status::internal(err.to_string()))?;
                    return Ok(Response::new(()));
                }
                Some(s) => s,
            };

            // compare
            if !Manager::compare_component_structure(expected_structure, actual_structure) {
                db.update_status(simulation_index, Failed, "Provided global component structures do not match structures expected by simulators".into()).await.map_err(|err| Status::internal(err.to_string()))?;
                db.commit()
                    .await
                    .map_err(|err| Status::internal(err.to_string()))?;
                return Ok(Response::new(()));
            }
        }

        // Store graph in database
        for node in nodes {
            db.add_node(node, simulation_index, 0)
                .await
                .map_err(|err| {
                    Status::internal(format!(
                        "push_simulation could not add the nodes with message: {:?}",
                        err.to_string()
                    ))
                })?;
        }
        for edge in edges {
            db.add_edge(edge, simulation_index, 0)
                .await
                .map_err(|err| {
                    Status::internal(format!(
                        "push_simulation could not add the edges with message: {:?}",
                        err.to_string()
                    ))
                })?;
        }
        for (key, value) in global {
            db.add_global_component(&key, value, simulation_index, 0)
                .await
                .map_err(|err| {
                    Status::internal(format!(
                        "push_simulation could not add global variable {} with message: {:?}",
                        key,
                        err.to_string()
                    ))
                })?;
        }
        // Commit transaction
        db.commit().await.map_err(|err| {
            Status::internal(format!(
                "push_simulation could not commit the new simulation with message: {:?}",
                err.to_string()
            ))
        })?;
        self.notif_sender.try_send(()).ok();
        Ok(Response::new(()))
    }

    /// Return all relevant info about a simulation. This includes the simulations status
    /// (finished/running/pending), how many frames have been processed, time step delta and total
    /// amount of time steps.
    async fn get_simulation(
        &self,
        request: Request<SimulationId>,
    ) -> Result<Response<SimulationData>, Status> {
        let simulation_id = request.into_inner().uuid;

        // Start transaction
        let mut db = self.db.lock().await;
        db.begin_transaction().await.map_err(|err| {
            Status::internal(format!(
                "get_simulation could not begin transaction with message: {:?}",
                err.to_string()
            ))
        })?;

        let simulation = db
            .get_simulation_via_name(&simulation_id)
            .await
            .map_err(|err| {
                Status::internal(format!(
                    "get_simulation could not fetch the simulation with message: {:?}",
                    err.to_string()
                ))
            })?;

        // Get current timestep
        let node_timestep = db
            .get_node_max_timestep(simulation.id)
            .await
            .map_err(|err| {
                Status::internal(format!(
                    "get_simulation could not fetch the max node timestep with message: {:?}",
                    err.to_string()
                ))
            })?;

        let component_timestep = db
            .get_global_components_max_timestep(simulation.id)
            .await
            .map_err(|err| Status::internal(format!("get_simulation could not fetch the max global variable timestep with message: {:?}", err.to_string())))?;

        let timestep = node_timestep.max(component_timestep);

        let sim_status = db.get_status(simulation.id).await.map_err(|err| {
            Status::internal(format!(
                "get_simulation could not fetch the simulation status with message: {:?}",
                err.to_string()
            ))
        })?;

        // Create response
        let simulation_data = SimulationData {
            id: Some(SimulationId {
                uuid: simulation_id,
            }),
            status: StatusEnum::to_simulation_status(sim_status).into(),
            timestep_count: timestep as u64,
            max_timestep_count: simulation.max_steps as u64,
            timestep_delta: simulation.step_size_ms as f64 / 1000.0,
            status_info: simulation.status_info,
        };

        // Commit transaction
        db.commit().await.expect("Commit transaction has failed");

        Ok(Response::new(simulation_data))
    }

    type GetSimulationFramesStream = std::pin::Pin<
        Box<dyn tokio_stream::Stream<Item = Result<SimulationFrame, Status>> + Send + 'static>,
    >;

    /// Return time frame using tonic stream
    ///
    /// The manager will send simulation frames back to the frontend using streams. As long as there
    /// are incoming frame_requests it will get all data from the database at the requested frame and
    /// compose a state to send to the frontend. Then it will send a response to the current request
    /// using the stream.
    async fn get_simulation_frames(
        &self,
        request: Request<tonic::Streaming<SimulationFrameRequest>>,
    ) -> Result<Response<Self::GetSimulationFramesStream>, Status> {
        // input stream
        let mut stream: tonic::Streaming<SimulationFrameRequest> = request.into_inner();
        let db_clone = self.db.clone();
        // output stream
        let output = async_stream::stream! {
            let db = db_clone;
            while let Some(frame_request) = stream.next().await {
                let frame_request = frame_request?;
                let simulation_name;
                if let Some(id) = frame_request.simulation_id.clone() {
                    simulation_name = id.uuid;
                } else {
                    yield Err(Status::invalid_argument("No simulation id was provided"));
                    continue;
                };

                // Start transaction
                let mut db = db.lock().await;

                db.begin_transaction().await.map_err(|err| Status::internal(format!("get_simulation_frames could not begin transaction with message: {:?}", err.to_string())))?;

                // Simulation index
                let simulation_id = db.get_simulation_via_name(&simulation_name).await.map_err(|err| Status::internal(format!("get_simulation_frames could not get the simulation id with message: {:?}", err.to_string())))?.id;

                let frame_index = frame_request.frame_nr as i32;

                let mut graph = Graph {
                    nodes: vec![],
                    edge: vec![],
                };

                let nodes_send = db.get_nodes(simulation_id, frame_index).await.map_err(|err| Status::internal(format!("get_simulation_frames could get the nodes with message: {:?}", err.to_string())))?;

                for node in nodes_send {
                    graph.nodes.push(node);
                }

                let edges_send = db.get_edges(simulation_id, frame_index).await.map_err(|err| Status::internal(format!("get_simulation_frames could not get the edges with message: {:?}", err.to_string())))?;

                for edge in edges_send {
                    graph.edge.push(edge);
                }

                let globals = db.get_global_components(simulation_id, frame_index).await.map_err(|err| Status::internal(format!("get_simulation_frames could not get the global variables with message: {:?}", err.to_string())))?;

                // Commit transaction
                db.commit().await.map_err(|err| Status::internal(format!("get_simulation_frames could not commit the changes with message: {:?}", err.to_string())))?;

                // output
                yield Ok(SimulationFrame {
                    request: Some(frame_request),
                    state: Some(State {
                        graph: Some(graph),
                        global_components: globals
                    })
                });
            }
        };
        Ok(Response::new(Box::pin(output)))
    }

    /// Get information about the simulators
    ///
    /// Gives the name and the output components of each simulator
    async fn get_simulators(&self, _request: Request<()>) -> Result<Response<Simulators>, Status> {
        let mut components: Simulators = Default::default();
        // clone simulator vec and drop mutex
        let mut simulators = self.simulators.lock().await;

        for simulator in &mut simulators.iter_mut() {
            let request = tonic::Request::new(IoConfigRequest {});
            let response = simulator
                .simulator
                .clone()
                .get_io_config(request)
                .await?
                .into_inner();
            let info: SimulatorInfo = SimulatorInfo {
                output_components: response.output_components,
                name: simulator.name.to_string(),
            };
            components.simulator.push(info);
        }
        drop(simulators);
        Ok(Response::new(components))
    }
}

// Uses ports 8005-8008 in localhost
#[cfg(test)]
mod manager_grpc_test {
    use std::net::SocketAddr;
    use tokio::time::{sleep, Duration};
    use tonic::transport::Server;

    use proto::simulation::simulation_manager::{SimulationManagerClient, SimulationManagerServer};

    use super::*;

    /// Tests pushing a simulation. This test uses sqlx::test so as not to
    /// affect the main simulations database. This test only runs if the db_test feature is enabled.
    #[cfg(feature = "db_test")]
    #[sqlx::test(migrations = "../migrations/simulator/")]
    async fn test_push_simulation(pool: PgPool) {
        use prost_types::value::Kind;
        use prost_types::Value;
        use proto::simulation::simulation_manager::SimulatorSelection;
        use proto::simulation::{Edge, Node};
        //set up
        let simulators: Arc<Mutex<Vec<SimulatorsInfo>>> = Arc::new(Mutex::new(Vec::default()));
        let (send, _recv) = mpsc::channel(1);
        let manager = Manager::new(pool.clone(), simulators, send).await;

        let node0 = Node {
            longitude: 11.11,
            latitude: 11.11,
            id: 0,
            components: [(
                "key1".to_string(),
                Value {
                    kind: Some(Kind::NumberValue(1.0)),
                },
            )]
            .into(),
        };
        let node1 = Node {
            longitude: 10.11,
            latitude: 10.11,
            id: 1,
            components: [(
                "key2".to_string(),
                Value {
                    kind: Some(Kind::NumberValue(2.0)),
                },
            )]
            .into(),
        };
        let edge1 = Edge {
            from: 0,
            to: 1,
            component_type: "Edge".to_string(),
            component_data: Some(Value {
                kind: Some(Kind::NumberValue(42.0)),
            }),
            id: 0,
        };
        let request1 = PushSimulationRequest {
            id: Some(SimulationId {
                uuid: "sim1".to_string(),
            }),
            timestep_delta: 30.0,
            timesteps: 3,
            selection: Some(SimulatorSelection { name: vec![] }),
            initial_state: Some(State {
                graph: Some(Graph {
                    nodes: vec![node0.clone(), node1.clone()],
                    edge: vec![edge1.clone()],
                }),
                global_components: [(
                    "key3".to_string(),
                    Value {
                        kind: Some(Kind::NumberValue(3.0)),
                    },
                )]
                .into(),
            }),
        };

        Manager::push_simulation(&manager, Request::new(request1))
            .await
            .expect("");

        // Check if the data in the database is correct
        let simulation = manager
            .db
            .lock()
            .await
            .get_simulation_via_name("sim1")
            .await
            .unwrap();

        assert_eq!(simulation.name, "sim1");
        assert_eq!(simulation.max_steps, 3);
        assert_eq!(simulation.step_size_ms, 30000);

        // Check if the nodes are added correctly to the database
        let nodes = manager
            .db
            .lock()
            .await
            .get_nodes(simulation.id, 0)
            .await
            .unwrap();

        assert_eq!(nodes[0], node0);
        assert_eq!(nodes[1], node1);

        // Check if the edges are added correctly to the database
        let edges = manager
            .db
            .lock()
            .await
            .get_edges(simulation.id, 0)
            .await
            .unwrap();

        assert_eq!(edges[0], edge1);

        // Check if the global components are added correctly to the database
        let globals = manager
            .db
            .lock()
            .await
            .get_global_components(simulation.id, 0)
            .await
            .unwrap();

        assert_eq!(
            globals["key3"],
            Value {
                kind: Some(Kind::NumberValue(3.0)),
            }
        );
    }

    pub struct ManagerTest {}

    impl ManagerTest {
        pub fn new() -> Self {
            Self {}
        }
    }

    #[tonic::async_trait]
    impl SimulationManager for ManagerTest {
        async fn get_components(&self, _: Request<()>) -> Result<Response<ComponentsInfo>, Status> {
            unreachable!()
        }

        async fn push_simulation(
            &self,
            _request: Request<PushSimulationRequest>,
        ) -> Result<Response<()>, Status> {
            unreachable!()
        }

        async fn delete_simulation(
            &self,
            _request: Request<DeleteSimulationRequestManager>,
        ) -> Result<Response<()>, Status> {
            unreachable!()
        }

        // requests an ID and returns the same ID
        async fn get_simulation(
            &self,
            request: Request<SimulationId>,
        ) -> Result<Response<SimulationData>, Status> {
            let simulation_id = request.into_inner().uuid;

            // return error
            if simulation_id == "error" {
                return Err(Status::from_error(Box::new(sqlx::Error::PoolTimedOut)));
            }

            let simulation_data = SimulationData {
                id: Some(SimulationId {
                    uuid: simulation_id,
                }),
                status: Default::default(),
                timestep_count: Default::default(),
                max_timestep_count: Default::default(),
                timestep_delta: Default::default(),
                status_info: Default::default(),
            };
            Ok(Response::new(simulation_data))
        }

        // type GetSimulationFramesStream = Streaming<SimulationFrame>;
        type GetSimulationFramesStream = std::pin::Pin<
            Box<dyn tokio_stream::Stream<Item = Result<SimulationFrame, Status>> + Send + 'static>,
        >;
        async fn get_simulation_frames(
            &self,
            request: Request<tonic::Streaming<SimulationFrameRequest>>,
        ) -> Result<Response<Self::GetSimulationFramesStream>, Status> {
            let mut stream: tonic::Streaming<SimulationFrameRequest> = request.into_inner();

            // output stream
            let output = async_stream::stream! {
                while let Some(frame_request) = stream.next().await {
                    let frame_request = frame_request?;

                    if frame_request.clone().simulation_id.unwrap().uuid == "error" {
                        Err(Status::from_error(Box::new(sqlx::Error::PoolTimedOut)))?;
                    }

                    yield Ok(SimulationFrame {
                        request: Some(frame_request),
                        state: Default::default()
                    });
                }
            };
            Ok(Response::new(Box::pin(output)))
        }

        async fn get_simulators(
            &self,
            _request: Request<()>,
        ) -> Result<Response<Simulators>, Status> {
            Ok(Response::new(Default::default()))
        }
    }

    // tests a simple connection between server and client.
    #[tokio::test]
    async fn test_connection() -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = "0.0.0.0:8005".parse().unwrap();
        let manager = ManagerTest::new();

        tokio::spawn(async move {
            let server = SimulationManagerServer::new(manager);

            let build = Server::builder().add_service(server).serve(addr).await;
            assert!(build.is_ok());
        });

        // Only works by waiting a little.
        // As this test takes around 0.02s, 1 nanosecond isn't much
        sleep(Duration::from_nanos(1)).await;

        let mut client = SimulationManagerClient::connect("http://0.0.0.0:8005")
            .await
            .unwrap();

        let simulation_id = SimulationId {
            uuid: "test".to_string(),
        };
        let res = client
            .get_simulation(Request::new(simulation_id.clone()))
            .await;

        // See if response is valid
        assert_eq!(
            simulation_id.uuid,
            res.unwrap().into_inner().id.unwrap().uuid
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_db_error() -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = "0.0.0.0:8006".parse().unwrap();
        let manager = ManagerTest::new();

        tokio::spawn(async move {
            let server = SimulationManagerServer::new(manager);

            let build = Server::builder().add_service(server).serve(addr).await;
            assert!(build.is_ok());
        });

        sleep(Duration::from_nanos(1)).await;

        let mut client = SimulationManagerClient::connect("http://0.0.0.0:8006")
            .await
            .unwrap();

        let simulation_id = SimulationId {
            uuid: "error".to_string(),
        };
        let res = client
            .get_simulation(Request::new(simulation_id.clone()))
            .await;

        assert_eq!(res.unwrap_err().code(), tonic::Status::unknown("").code());

        Ok(())
    }

    #[tokio::test]
    async fn test_grpc_stream() -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = "0.0.0.0:8007".parse().unwrap();
        let manager = ManagerTest::new();

        tokio::spawn(async move {
            let server = SimulationManagerServer::new(manager);

            let build = Server::builder().add_service(server).serve(addr).await;
            assert!(build.is_ok());
        });

        sleep(Duration::from_nanos(1)).await;

        let mut client = SimulationManagerClient::connect("http://0.0.0.0:8007")
            .await
            .unwrap();

        let simulation_id = SimulationId {
            uuid: "test".to_string(),
        };
        let mut frames = vec![];

        let frame_count = 5;
        for i in 0..frame_count {
            frames.push(SimulationFrameRequest {
                simulation_id: Some(simulation_id.clone()),
                frame_nr: i,
            })
        }

        let mut res = client
            .get_simulation_frames(Request::new(tokio_stream::iter(frames)))
            .await
            .unwrap()
            .into_inner();

        let mut i = 0;
        while let Some(frame_response) = res.next().await {
            assert!(frame_response.is_ok());
            i += 1;
        }
        assert_eq!(i, frame_count);

        Ok(())
    }

    #[tokio::test]
    async fn test_stream_error() -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = "0.0.0.0:8008".parse().unwrap();
        let manager = ManagerTest::new();

        tokio::spawn(async move {
            let server = SimulationManagerServer::new(manager);

            let build = Server::builder().add_service(server).serve(addr).await;
            assert!(build.is_ok());
        });

        sleep(Duration::from_nanos(1)).await;

        let mut client = SimulationManagerClient::connect("http://0.0.0.0:8008")
            .await
            .unwrap();

        let simulation_id = SimulationId {
            uuid: "test".to_string(),
        };
        let mut frames = vec![];

        for i in 0..9 {
            frames.push(SimulationFrameRequest {
                simulation_id: Some(simulation_id.clone()),
                frame_nr: i,
            });
        }

        frames.push(SimulationFrameRequest {
            simulation_id: Some(SimulationId {
                uuid: "error".to_string(),
            }),
            frame_nr: 10,
        });

        let mut res = client
            .get_simulation_frames(Request::new(tokio_stream::iter(frames)))
            .await
            .unwrap()
            .into_inner();

        // error during stream
        while let Some(frame_response) = res.next().await {
            assert!(frame_response.is_err());
        }

        Ok(())
    }
}
