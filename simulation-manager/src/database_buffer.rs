use crate::database::SimulationsDB;
use anyhow::Context;
use proto::simulation::State;
use sqlx::PgPool;
use tokio::sync::mpsc;

/// The database buffer struct holds a postgres connection pool and an async channel. The postgres
/// connection is used to write every timeframe to the database.
/// The async channel is used by the simulation runner to pass finished timesteps to the database buffer
/// which then writes them to the database.
pub struct DatabaseBuffer {
    state_receiver: mpsc::UnboundedReceiver<Transport>,
    connection: SimulationsDB,
}

/// Struct to transport data to database buffer
#[derive(Clone)]
pub struct Transport {
    pub simulation_id: i32,
    pub iteration: i32,
    pub state: State,
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

                // unpack transport
                let i = transport.iteration;
                let simulation_id = transport.simulation_id;
                let state = transport.state.clone();
                let graph = state.graph.unwrap();
                let global = transport.state.clone().global_components;
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

                self.connection
                    .commit()
                    .await
                    .context("while trying to commit transaction")?;
            }
        }
    }
}
