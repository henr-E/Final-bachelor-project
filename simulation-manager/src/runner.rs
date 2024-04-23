use anyhow::Context;
use futures::future;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tracing::error;

// sqlx
use sqlx::PgPool;
// tokio
use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep;
// tonic
use tonic::transport::Channel;

// proto
use crate::database::{SimulationsDB, StatusEnum};
use crate::database_buffer::Transport;
use proto::simulation::simulator::{
    simulator_client::SimulatorClient, InitialState, IoConfigRequest,
};
use proto::simulation::{Graph, State};

/// The runner contains all the functionality to interface with all the different simulators.
///
/// The runner holds a database connection, a vector of known simulators and a receiver for the
/// asynchronous channel created in main.rs
pub struct Runner {
    db: SimulationsDB,
    simulators: Arc<Mutex<Vec<SimulatorClient<Channel>>>>,
    notif_receiver: mpsc::Receiver<()>,
    state_sender: mpsc::UnboundedSender<Transport>,
}

impl Runner {
    /// Create a new Runner
    pub async fn new(
        pool: PgPool,
        simulators: Arc<Mutex<Vec<SimulatorClient<Channel>>>>,
        notif_receiver: mpsc::Receiver<()>,
        state_sender: mpsc::UnboundedSender<Transport>,
    ) -> Self {
        Self {
            db: SimulationsDB::from_pg_pool(pool).await.unwrap(),
            simulators,
            notif_receiver,
            state_sender,
        }
    }

    /// Start the runner.
    ///
    /// The runner is currently implemented to use busy waiting to poll the database for new simulations.
    /// It will then set up every simulator and start the simulation. Currently, simulations are handled
    /// one by one.
    /// If no new simulation is found the runner waits 30 sec before checking the database again except
    /// if during this wait time a message is received over the asynchronous channel.
    pub async fn start(&mut self) -> anyhow::Result<()> {
        loop {
            let top = self
                .db
                .dequeue()
                .await
                .context("could not dequeue simulation")?;
            if let Some(simulation_id) = top {
                self.db
                    .begin_transaction()
                    .await
                    .context("could not begin transaction")?;
                // check that the simulators do not change the same information
                let mut output_components: HashSet<String> = HashSet::default();
                let guard = self.simulators.lock().await;
                let simulators = guard.clone();
                drop(guard);

                for server in &mut simulators.clone() {
                    let request = tonic::Request::new(IoConfigRequest {});
                    let response = server.get_io_config(request).await?.into_inner();
                    for name in response.output_components {
                        if !output_components.insert(name) {
                            self.db
                                .update_status(simulation_id, StatusEnum::Failed)
                                .await
                                .context("could not update status")?;
                            break;
                        }
                    }
                }
                let status = self
                    .db
                    .get_status(simulation_id)
                    .await
                    .context("could not get status")?;
                if status != StatusEnum::Failed {
                    // Make error handling easier by putting the two functions below into one async
                    // block. This allows errors from both to be handled by the same code.
                    let do_simulation = async {
                        self.set_up(simulation_id).await.context("in `set_up`")?;
                        self.start_simulation(simulation_id)
                            .await
                            .context("in `start_simulation`")?;
                        anyhow::Ok(())
                    };
                    if let Err(err) = do_simulation.await {
                        error!("Simulation `{simulation_id}` failed: {err:?}");
                        self.db
                            .update_status(simulation_id, StatusEnum::Failed)
                            .await
                            .context("could not update status after failed simulation")?;
                    }
                }
                self.db
                    .commit()
                    .await
                    .context("could not commit transaction")?;
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
    /// The setup for a simulation consists of creating an initial state.
    /// The runner assumes all data needed to run a simulation is present in the database. This means
    /// every node and edge along with its components and the global components should be
    /// present with time_step == 0.
    /// The runner will get all these nodes, edges and components, compose a proto::simulation::Graph
    /// and then put this graph along with the step size into a state. This state is then sent to the
    /// simulators.
    async fn set_up(&mut self, simulation_id: i32) -> anyhow::Result<()> {
        // clone simulator vec and drop mutex
        let guard = self.simulators.lock().await;
        let simulators = guard.clone();
        drop(guard);

        // get tick delta
        let delta = self
            .db
            .get_delta(simulation_id)
            .await
            .context("error getting delta")?;
        let mut graph = Graph {
            nodes: vec![],
            edge: vec![],
        };

        // get current simulation nodes at timestep 0 and add to graph
        let mut nodes = self
            .db
            .get_nodes(simulation_id, 0)
            .await
            .context("error getting nodes")?;
        graph.nodes.append(&mut nodes);

        // get current simulation edges at timestep 0 and add to graph
        let mut edges = self
            .db
            .get_edges(simulation_id, 0)
            .await
            .context("error getting edges")?;
        graph.edge.append(&mut edges);

        // add all global components to the graph
        let globals = self
            .db
            .get_global_components(simulation_id, 0)
            .await
            .context("error getting global components")?;

        // create initial state
        let initial_state = InitialState {
            initial_state: Some(State {
                graph: Some(graph),
                global_components: globals,
            }),
            timestep_delta: delta as u64,
        };

        // // setup of simulators
        future::try_join_all(
            simulators
                .clone()
                .into_iter()
                .map(|server| (initial_state.clone(), server))
                .map(|(initial_state, mut server)| async move {
                    let setup_request = tonic::Request::new(initial_state);
                    server.setup(setup_request).await
                }),
        )
        .await?;
        Ok(())
    }

    /// Run a full simulation
    ///
    /// For each tick in the simulation the runner will execute a timestep for each simulator.
    /// The runner will get all the needed components (nodes, edges and components) from the previous
    /// timestep and send this state to the simulators using grpc and wait for a response. This now
    /// happens in parallel to improve performance and decrease the time it takes to run a full simulation.
    /// When the runner receives a response from a simulator, it will create a Transport object. Once
    /// All simulators have responded. These different transport objects are returned as a vector.
    /// The runner then combines all these responses into one big state and sends this to the
    /// database buffer using the async channel. The buffer will then write the timestep to the database.
    /// Writing to the database is done on a seperate thread so that the runner does not need to spend
    /// time waiting on the database. The async channel has an unbounded queue of messages so the
    /// runner can keep queueing finished timeframes if it takes a long time for the database buffer
    /// to process them.
    /// In the case that a simulator does not return all components the components that weren't sent
    /// back will be duplicated into the next timestep so that they are available in the next tick
    async fn start_simulation(&mut self, simulation_id: i32) -> anyhow::Result<()> {
        // get amount of iterations to run the simulation for
        let iterations = self
            .db
            .get_iterations(simulation_id)
            .await
            .context("error getting iterations")?;

        // get initial state
        let mut graph = Graph {
            nodes: vec![],
            edge: vec![],
        };
        let mut nodes_send = self
            .db
            .get_nodes(simulation_id, 0)
            .await
            .context("error getting nodes")?;
        graph.nodes.append(&mut nodes_send);

        let mut edges_send = self
            .db
            .get_edges(simulation_id, 0)
            .await
            .context("error getting edges")?;
        graph.edge.append(&mut edges_send);

        let globals = self
            .db
            .get_global_components(simulation_id, 0)
            .await
            .context("error getting glboal components")?;

        let mut prev: State = State {
            graph: Some(graph),
            global_components: globals,
        };

        // clone simulator vec and drop mutex
        let guard = self.simulators.lock().await;
        let simulators = guard.clone();
        drop(guard);
        for i in 0..iterations {
            // parallel execution of the simulators in the simulation for the current time step
            let results = future::try_join_all(
                simulators
                    .clone()
                    .into_iter()
                    .map(|server| (prev.clone(), server))
                    .map(|(prev, mut server)| async move {
                        // get values from state
                        let graph = prev.clone().graph.unwrap();
                        let grpc_global = prev.clone().global_components;
                        let nodes = graph.nodes.clone();
                        let edges = graph.edge.clone();

                        let mut edge_ids_sent = Vec::new();
                        for edge in edges {
                            edge_ids_sent.push(edge.id as i32);
                        }
                        let mut node_ids_sent = Vec::new();
                        for node in nodes {
                            node_ids_sent.push(node.id as i32);
                        }
                        let mut global_ids_sent = Vec::new();
                        for (key, _value) in grpc_global.clone() {
                            global_ids_sent.push(key);
                        }

                        // send to server and do time step
                        let do_time_step_request = tonic::Request::new(State {
                            graph: Some(graph.clone()),
                            global_components: grpc_global.clone(),
                        });
                        let do_time_step_response = server
                            .do_timestep(do_time_step_request)
                            .await
                            .context("in call `do_timestep`")?;

                        // read out results of simulator
                        let output_state = do_time_step_response
                            .into_inner()
                            .output_state
                            .context("no output state found")?;

                        // place results into Transport struct
                        anyhow::Ok(Transport {
                            simulation_id,
                            iteration: i + 1,
                            state: output_state,
                        })
                    }),
            )
            .await?;

            // make copy of previous state
            let mut new_state = prev.clone();

            // merge previous state with all output states
            for result in results {
                // Replace previous node with the version that has been returned by the simulator.
                // Since simulators can not edit the same nodes, this will always work.
                // Nodes that were not returned by any simulator will also still be present
                let result_graph = result.state.graph.unwrap();
                for result_node in result_graph.nodes {
                    let node = new_state
                        .graph
                        .as_mut()
                        .unwrap()
                        .nodes
                        .iter_mut()
                        .find(|n| n.id == result_node.id)
                        .unwrap();
                    for (name, c) in result_node.components.into_iter() {
                        node.components.insert(name, c);
                    }
                }

                // Idem for edges
                for result_edge in result_graph.edge {
                    let edge = new_state
                        .graph
                        .as_mut()
                        .unwrap()
                        .edge
                        .iter_mut()
                        .find(|e| e.id == result_edge.id)
                        .unwrap();
                    *edge = result_edge;
                }

                // Idem for global components
                for (key, value) in result.state.global_components.clone() {
                    new_state.global_components.insert(key, value);
                }
            }

            // create transport and send to database buffer
            let transport = Transport {
                simulation_id,
                iteration: i + 1,
                state: new_state.clone(),
            };
            self.state_sender.send(transport)?;
            let status = match i {
                0 => StatusEnum::Pending,
                i if i < iterations - 1 => StatusEnum::Computing,
                i if i == iterations - 1 => StatusEnum::Finished,
                _ => StatusEnum::Failed,
            };
            self.db
                .update_status(simulation_id, status)
                .await
                .context("error updating status")?;
            // set previous state to new state
            prev = new_state.clone();
        }
        Ok(())
    }
}
