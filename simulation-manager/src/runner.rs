use std::collections::HashMap;
use std::time::Duration;

use prost_types::Value;
use sqlx::PgPool;
use tokio::sync::mpsc;
use tokio::time::sleep;
// tonic
use tonic::transport::Channel;

use prost_value::*;
//sqlx
use proto::simulation::{Edge, Graph, Node, State};
// proto
use proto::simulation::simulator::{
    simulator_client::SimulatorClient, InitialState, IoConfigRequest,
};

/// The runner contains all the functionality to interface with all the different simulators.
///
/// The runner holds a database connection, a vector of known simulators and a receiver for the
/// asynchronous channel created in main.rs
pub struct Runner {
    pool: PgPool,
    simulators: Vec<SimulatorClient<Channel>>,
    notif_receiver: mpsc::Receiver<()>,
}

impl Runner {
    /// Create a new Runner
    pub fn new(
        pool: PgPool,
        simulators: Vec<SimulatorClient<Channel>>,
        notif_receiver: mpsc::Receiver<()>,
    ) -> Self {
        Self {
            pool,
            simulators,
            notif_receiver,
        }
    }

    /// Start the runner.
    ///
    /// The runner is currently implemented to use busy waiting to poll the database for new simulations.
    /// It will then set up every simulator and start the simulation. Currenly Simulations are handled
    /// one by one.
    /// If no new simulation is found the runner waits 30 sec before checking the database again except
    /// if during this wait time a message is received over the asynchronous channel.
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            if let Some(top) = sqlx::query!("SELECT simulation_id FROM queue ORDER BY id ASC")
                .fetch_optional(&self.pool)
                .await?
            {
                let simulation_id = top.simulation_id;
                let mut transaction = self.pool.begin().await?;
                sqlx::query!("DELETE FROM queue WHERE simulation_id = $1", simulation_id)
                    .execute(&mut *transaction)
                    .await?;
                self.set_up(simulation_id).await?;
                self.start_simulation(simulation_id).await?;
                transaction.commit().await?;
            } else {
                tokio::select! {
                    _ = sleep(Duration::from_secs(30)) => {},
                    _ = self.notif_receiver.recv() => {},
                }
            }
        }
    }

    /// Set up a simulation based on simulation id.
    ///
    /// The setup for a simulation consists of creating an initial state for every simulator.
    /// The runner assumes all data needed to run a simulation is present in the database. This means
    /// every node and edge along with its components and the global components should be
    /// present with time_step == 0.
    /// The runner will get all these nodes, edges and components, compose a proto::simulation::Graph
    /// and then put this graph along with the step size into a state. This state is then sent to the
    /// simulators.
    async fn set_up(&self, simulation_id: i32) -> Result<(), Box<dyn std::error::Error>> {
        // get tick delta
        let delta = sqlx::query!(
            "SELECT step_size_ms FROM simulations WHERE id = $1",
            simulation_id
        )
        .fetch_one(&self.pool)
        .await?
        .step_size_ms;

        // setup of simulators
        for server in &mut self.simulators.clone() {
            let io_config_request = tonic::Request::new(IoConfigRequest {});
            let _io_config_response = server.get_io_config(io_config_request).await?.into_inner();
            // use io_config_response to figure out which data to get from the database and create an initial state
            let mut graph = Graph {
                nodes: vec![],
                edge: vec![],
            };

            // select current simulation nodes at timestep 0
            let nodes = sqlx::query!(
                "SELECT * FROM nodes WHERE simulation_id = $1 AND time_step = 0",
                simulation_id
            )
            .fetch_all(&self.pool)
            .await?;

            // for every node get its components and add to graph
            for node in nodes {
                let id = node.id; // id of the node in the database for given time step and simulation
                let node_id = node.node_id; // the actual id of the node
                let longitude = node.longitude;
                let latitude = node.latitude;
                let mut grpc_node = Node {
                    longitude,
                    latitude,
                    components: Default::default(),
                    id: node_id as u64,
                };

                // look for node components based on the database id of the node
                let components =
                    sqlx::query!("SELECT * FROM node_components Where node_id = $1", id)
                        .fetch_all(&self.pool)
                        .await?;

                for component in components {
                    let name = component.name;
                    let data = component.component_data;
                    grpc_node.components.insert(name, serde_json_to_prost(data));
                }

                graph.nodes.push(grpc_node);
            }

            // for each edge, add it to graph
            let edges = sqlx::query!(
                "SELECT * FROM edges WHERE simulation_id = $1 AND time_step = 0",
                simulation_id
            )
            .fetch_all(&self.pool)
            .await?;

            for edge in edges {
                let edge_id = edge.edge_id; // the actual id of the edge
                let from: i32 = edge.from_node;
                let to: i32 = edge.to_node;
                let component_type = edge.component_type;
                let component_data = edge.component_data;

                let grpc_edge = Edge {
                    id: edge_id as u64,
                    from: from as u64,
                    to: to as u64,
                    component_type,
                    component_data: Option::from(prost_value::serde_json_to_prost(component_data)),
                };
                graph.edge.push(grpc_edge);
            }

            // add all global components to the graph
            let globals = sqlx::query!(
                "SELECT * FROM global_components where simulation_id = $1 and time_step = 0",
                simulation_id
            )
            .fetch_all(&self.pool)
            .await?;

            let mut global: HashMap<String, Value> = HashMap::new();
            for component in globals {
                let component_name = component.name;
                let component_data = serde_json_to_prost(component.component_data);

                global.insert(component_name, component_data);
            }

            // create initial state
            let initial_state = State {
                graph: Some(graph),
                global_components: global,
            };

            // send it to the server
            let setup_request = tonic::Request::new(InitialState {
                timestep_delta: delta as u64,
                initial_state: Option::from(initial_state),
            });
            let _setup_response = server.setup(setup_request).await?;
        }
        Ok(())
    }

    /// Run a full simulation
    ///
    /// For each tick in the simulation the runner will execute a timestep for each simulator.
    /// The runner will get all the needed components (nodes, edges and components) from the previous
    /// timestep from the database and compose a state from them. Then it will send this state to
    /// the simulator using grpc and wait for a response. When the runner receives a response from the
    /// simulator, it will decompose the received state and put all the components back into the database
    /// at timestep +1
    /// In the case that a simulator does not return all components the components that weren't sent
    /// back will be duplicated into the next timestep so that they are available in the next tick
    async fn start_simulation(&self, simulation_id: i32) -> Result<(), Box<dyn std::error::Error>> {
        // get amount of iterations to run the simulation for
        let iterations = sqlx::query!(
            "SELECT max_steps FROM simulations WHERE id = $1",
            simulation_id
        )
        .fetch_one(&self.pool)
        .await?
        .max_steps;

        for i in 0..iterations {
            let mut edge_ids_send: Vec<i32> = Vec::new();
            let mut global_ids_send: Vec<String> = Vec::new();
            let mut node_ids_send: Vec<i32> = Vec::new();
            let mut edge_ids_received: Vec<i32> = Vec::new();
            let mut global_ids_received: Vec<String> = Vec::new();
            let mut node_ids_received: Vec<i32> = Vec::new();
            let mut transaction = self.pool.begin().await?;
            for server in &mut self.simulators.clone() {
                // get needed data from database
                let mut graph = Graph {
                    nodes: vec![],
                    edge: vec![],
                };
                let nodes_send = sqlx::query!(
                    "SELECT * FROM nodes WHERE simulation_id = $1 AND time_step = $2",
                    simulation_id,
                    i
                )
                .fetch_all(&mut *transaction)
                .await?;

                for node in nodes_send {
                    let id = node.id; // id of the node for the given time step and simulation
                    let node_id = node.node_id; // the actual id of the node
                    let longitude = node.longitude;
                    let latitude = node.latitude;
                    let mut grpc_node = Node {
                        longitude,
                        latitude,
                        components: Default::default(),
                        id: node_id as u64,
                    };
                    // look for the node components based on the database id of the node
                    let components =
                        sqlx::query!("SELECT * FROM node_components Where node_id = $1", id)
                            .fetch_all(&mut *transaction)
                            .await?;

                    for component in components {
                        let name = component.name;
                        let data = component.component_data;
                        grpc_node.components.insert(name, serde_json_to_prost(data));
                    }

                    graph.nodes.push(grpc_node);
                    // keep track of all send nodes
                    if !node_ids_send.contains(&node_id) {
                        node_ids_send.push(node_id);
                    }
                }

                let edges_send = sqlx::query!(
                    "SELECT * FROM edges WHERE simulation_id = $1 AND time_step = $2",
                    simulation_id,
                    i
                )
                .fetch_all(&mut *transaction)
                .await?;

                for edge in edges_send {
                    let edge_id = edge.edge_id; // the actual id of the edge
                    let from = edge.from_node;
                    let to = edge.to_node;
                    let component_type = edge.component_type;
                    let component_data = edge.component_data;

                    let grpc_edge = Edge {
                        from: from as u64,
                        to: to as u64,
                        component_type,
                        component_data: Option::from(serde_json_to_prost(component_data)),
                        id: edge_id as u64,
                    };
                    graph.edge.push(grpc_edge);
                    // keep track of all send edges
                    if !edge_ids_send.contains(&edge_id) {
                        edge_ids_send.push(edge_id);
                    }
                }

                let globals = sqlx::query!(
                    "SELECT * FROM global_components WHERE simulation_id = $1",
                    simulation_id
                )
                .fetch_all(&mut *transaction)
                .await?;

                let mut grpc_global: HashMap<String, Value> = HashMap::new();

                for component in globals {
                    let component_name = component.name;
                    let component_data = component.component_data;
                    global_ids_send.push(component_name.clone());
                    grpc_global.insert(component_name, serde_json_to_prost(component_data));
                }

                // send to server and do time step
                let do_time_step_request = tonic::Request::new(State {
                    graph: Some(graph),
                    global_components: grpc_global,
                });
                let do_time_step_response = server.do_timestep(do_time_step_request).await?;

                // write results back to database
                let output_state = do_time_step_response.into_inner().output_state.unwrap();
                let graph = output_state.graph.unwrap();
                let global = output_state.global_components;
                let nodes = graph.nodes;
                let edges = graph.edge;

                for node in nodes {
                    // write node to db
                    let node_id = node.id as i32;
                    let id = sqlx::query!("INSERT INTO nodes (node_id, simulation_id, time_step, longitude, latitude) VALUES($1, $2, $3, $4, $5) RETURNING id", node_id, simulation_id, i+1, node.longitude, node.latitude)
                        .map(|row| { row.id })
                        .fetch_one(&mut *transaction).await?;

                    // keep track of which nodes have been returned from the simulation
                    node_ids_received.push(node.id as i32);

                    // write node components to db
                    for (component_name, component_value) in node.components {
                        sqlx::query!("INSERT INTO node_components (name, node_id, component_data) Values ($1, $2, $3)", component_name, id, prost_to_serde_json(component_value))
                            .execute(&mut *transaction).await?;
                    }
                }

                for edge in edges {
                    let from_node = edge.from as i32;
                    let to_node = edge.to as i32;
                    let edge_id = edge.id as i32;
                    // write edge to db
                    sqlx::query!("INSERT INTO edges (edge_id, simulation_id, time_step, from_node, to_node, component_data, component_type) VALUES ($1, $2, $3, $4, $5, $6, $7)", edge_id, simulation_id, i+1, from_node, to_node,prost_to_serde_json(edge.component_data.unwrap()), edge.component_type)
                        .execute(&mut *transaction).await?;

                    // keep track of which edges have been returned from the simulation
                    edge_ids_received.push(edge.id as i32);
                }

                for (key, value) in global {
                    global_ids_received.push(key.clone());
                    sqlx::query!("INSERT INTO global_components (name, simulation_id, time_step, component_data) VALUES ($1, $2, $3, $4)", key, simulation_id, i+1, prost_to_serde_json(value))
                        .execute(&mut *transaction).await?;
                }
            }
            // duplicate not returned nodes from the simulator to the next time step
            for node_id in node_ids_send {
                if !node_ids_received.contains(&node_id) {
                    // get node from current time step
                    let duplicate_node = sqlx::query!(
                        "SELECT * FROM nodes WHERE node_id = $1 AND simulation_id = $2 AND time_step = $3",
                        node_id, simulation_id, i
                    )
                        .fetch_all(&mut *transaction)
                        .await?;
                    let dup_node = &duplicate_node[0]; //TODO: check if this is actually unique

                    // duplicate node to new time step
                    let id = sqlx::query!("INSERT INTO nodes (node_id, simulation_id, time_step, longitude, latitude) VALUES($1, $2, $3, $4, $5) RETURNING id",
                        dup_node.node_id, simulation_id, i+1, dup_node.longitude, dup_node.latitude)
                        .map(|row| { row.id })
                        .fetch_one(&mut *transaction).await?;

                    let duplicate_comp =
                        sqlx::query!("SELECT * FROM node_components WHERE node_id = $1", node_id)
                            .fetch_all(&mut *transaction)
                            .await?;

                    // write node components to db
                    for comp in duplicate_comp {
                        sqlx::query!("INSERT INTO node_components (name, node_id, component_data) Values ($1, $2, $3)", comp.name, id, comp.component_data)
                            .execute(&mut *transaction).await?;
                    }
                }
            }

            // duplicate not returned edges from the simulator to the next time step
            for edge_id in edge_ids_send {
                if !edge_ids_received.contains(&edge_id) {
                    // get edge from current time step
                    let duplicate_edge = sqlx::query!(
                        "SELECT * FROM edges WHERE edge_id = $1 AND simulation_id = $2 AND time_step = $3",
                        edge_id, simulation_id, i
                    )
                        .fetch_all(&mut *transaction)
                        .await?;
                    let dup_edge = &duplicate_edge[0]; //TODO: check if this is actually unique

                    // duplicate edge to new time step
                    sqlx::query!("INSERT INTO edges (edge_id, simulation_id, time_step, from_node, to_node, component_data, component_type) VALUES($1, $2, $3, $4, $5, $6, $7)",
                        dup_edge.edge_id, simulation_id, i+1, dup_edge.from_node, dup_edge.to_node, dup_edge.component_data, dup_edge.component_type)
                        .execute(&mut *transaction).await?;
                }
            }

            // duplicate not returned global components from the simulator to the next time step
            for global in global_ids_send {
                if !global_ids_received.contains(&global) {
                    let duplicate_global = sqlx::query!(
                        "SELECT * FROM global_components WHERE name = $1 AND simulation_id = $2 AND time_step = $3",
                        global, simulation_id, i
                    )
                        .fetch_all(&mut *transaction)
                        .await?;
                    let dup_global = &duplicate_global[0]; //TODO: check if this is actually unique

                    // duplicate global component to new time step
                    sqlx::query!("INSERT INTO global_components (time_step, name, simulation_id, component_data) VALUES ($1, $2, $3, $4)",
                        i+1, dup_global.name, simulation_id, dup_global.component_data)
                        .execute(&mut *transaction).await?;
                }
            }

            transaction.commit().await?;
        }
        Ok(())
    }
}
