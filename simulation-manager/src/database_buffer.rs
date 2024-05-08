use crate::database::{SimulationsDB, StatusEnum};
use anyhow::Context;
use proto::simulation::State;
use sqlx::PgPool;
use tokio::sync::mpsc;

/// The database buffer struct holds a postgres connection pool and an async channel. The postgres
/// connection is used to write every timeframe to the database and update simulation status.
/// The async channel is used by the simulation runner to pass finished timesteps and status to the database buffer
/// which then writes them to the database.
pub struct DatabaseBuffer {
    state_receiver: mpsc::UnboundedReceiver<Transport>,
    connection: SimulationsDB,
}

/// Specifies either simulation state or simulation status
pub enum Transport {
    State(StateTransport),
    Status(StatusTransport),
}

/// Struct to transport simulation data to database buffer
#[derive(Clone)]
pub struct StateTransport {
    pub simulation_id: i32,
    pub iteration: i32,
    pub state: State,
}

/// Struct to transport status information to database buffer
#[derive(Clone)]
pub struct StatusTransport {
    pub simulation_id: i32,
    pub status: StatusEnum,
    pub status_info: String,
}

impl DatabaseBuffer {
    /// Create new database buffer
    pub async fn new(pool: PgPool, state_receiver: mpsc::UnboundedReceiver<Transport>) -> Self {
        Self {
            connection: SimulationsDB::from_pg_pool(pool).await.unwrap(),
            state_receiver,
        }
    }

    /// Start the infinite loop in which the database buffer will check if new messages have arrived
    /// at the async channel.
    /// If there is a new message it will process the timeframe by writing all nodes, node components,
    /// edges and global components to the database. While writing all the different parts of the state
    /// to the database it keeps track of which nodes, edges and components were returned. If this does
    /// not match the state that was sent, the parts that were not returned will be duplicated to the
    /// next timestep.
    pub async fn start(mut self) -> anyhow::Result<()> {
        loop {
            if let Some(transport) = self.state_receiver.recv().await {
                self.connection
                    .begin_transaction()
                    .await
                    .context("while trying to begin transaction")?;

                match transport {
                    // write state to database
                    Transport::State(transport) => {
                        // unpack transport
                        let i = transport.iteration;
                        let simulation_id = transport.simulation_id;
                        let state = transport.state;

                        // multiple checks were done to make sure a graph is present before sending it here
                        let graph = state.graph.context("while trying to extract graph")?;
                        let global = state.global_components;
                        let nodes = graph.nodes;
                        let edges = graph.edge;

                        for node in nodes {
                            // write node to db
                            self.connection
                                .add_node(node, simulation_id, i)
                                .await
                                .context("while trying to add node to database")?;
                        }

                        for edge in edges {
                            // write edge to tb
                            self.connection
                                .add_edge(edge, simulation_id, i)
                                .await
                                .context("while trying to add edge to database")?;
                        }

                        for (key, value) in global {
                            // write global component to db
                            self.connection
                                .add_global_component(&key, value, simulation_id, i)
                                .await
                                .context("while trying to add global component to database")?;
                        }
                    }
                    // update status in database
                    Transport::Status(transport) => {
                        let simulation_id = transport.simulation_id;
                        let status = transport.status;
                        let info = transport.status_info;
                        // write status to db
                        self.connection
                            .update_status(simulation_id, status, Some(&info))
                            .await
                            .context("while trying to update status")?;
                    }
                }

                self.connection
                    .commit()
                    .await
                    .context("while trying to commit transaction")?;
            }
        }
    }
}
