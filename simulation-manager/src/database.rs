#![allow(dead_code)]
use std::collections::HashMap;

use anyhow::{anyhow, Ok, Result};
use sqlx::pool::PoolConnection;
use sqlx::types::chrono::NaiveDate;
use sqlx::{query, query_as, PgConnection, PgPool, Postgres, Transaction};

use database_config::database_url;
use prost_types::Value;
use prost_value::*;
use proto::simulation::{Edge, Node};

type Date = NaiveDate;

pub struct Simulation {
    pub id: i32,
    pub date: Date,
    pub name: String,
    pub step_size_ms: i32,
    pub max_steps: i32,
}

/// An API abstraction over the simulations database.
pub struct SimulationsDB {
    pool: PgPool,
    transaction: Option<Transaction<'static, Postgres>>,
    connection: Option<PoolConnection<Postgres>>,
}

impl SimulationsDB {
    /// Create a new connection to the database. Uses the credentials from the `SIMULATIONS`
    /// environment variable.
    pub async fn connect() -> Result<Self> {
        let url = database_url("SIMULATIONS", "SIMULATIONS", None, None);
        let pool = PgPool::connect(&url).await?;
        Ok(Self {
            pool,
            transaction: None,
            connection: None,
        })
    }

    /// Create a new connection to the database using a `sqlx::PgPool`.
    pub async fn from_pg_pool(pool: PgPool) -> Result<Self> {
        Ok(Self {
            pool,
            transaction: None,
            connection: None,
        })
    }

    /// Gets the right connection for queries. If a transaction is started we use that, otherwise
    /// we use a connection from the pool.
    async fn connection(&mut self) -> Result<&mut PgConnection> {
        if self.transaction.is_some() {
            return Ok(self.transaction.as_deref_mut().unwrap());
        }
        if self.connection.is_none() {
            self.connection = Some(self.pool.acquire().await.map_err(|e| anyhow!(e))?);
        }
        return Ok(self.connection.as_deref_mut().unwrap());
    }

    /// Begin a database transaction. The changes won't take effect until commit() is called.
    pub async fn begin_transaction(&mut self) -> Result<()> {
        self.transaction = Some(self.pool.begin().await?);
        Ok(())
    }

    /// Commit all changes since the last call to begin()
    pub async fn commit(&mut self) -> Result<()> {
        let t = self.transaction.take().unwrap();
        t.commit().await.map_err(|e| anyhow!(e))?;
        Ok(())
    }

    /// Add a simulation to the simlations table.
    pub async fn add_simulation(
        &mut self,
        name: &str,
        step_size_ms: i32,
        max_steps: i32,
    ) -> Result<i32> {
        query!("INSERT INTO simulations (name, step_size_ms, max_steps) VALUES($1, $2, $3) RETURNING id",
            name, step_size_ms, max_steps)

        .fetch_one(self.connection().await?)
            .await
            .map_err(|e| anyhow!(e))
            .map(|s| s.id)
    }

    /// Get a simulation from the simulations table.
    pub async fn get_simulation(&mut self, name: &str) -> Result<Simulation> {
        query_as!(
            Simulation,
            "SELECT * FROM simulations WHERE name = $1",
            name
        )
        .fetch_one(self.connection().await?)
        .await
        .map_err(|err| anyhow!(err))
    }

    /// Add a simulation to the queue table.
    pub async fn enqueue(&mut self, simulation_id: i32) -> Result<i32> {
        query!(
            "INSERT INTO queue (simulation_id) VALUES($1) RETURNING id",
            simulation_id
        )
        .fetch_one(self.connection().await?)
        .await
        .map_err(|e| anyhow!(e))
        .map(|s| s.id)
    }

    /// Pop the first simulation from the queue table.
    pub async fn dequeue(&mut self) -> Result<Option<i32>> {
        let simulation_id = query!("SELECT simulation_id FROM queue ORDER BY id ASC",)
            .fetch_optional(&self.pool)
            .await?
            .map(|s| s.simulation_id);
        if let Some(id) = simulation_id {
            query!("DELETE FROM queue WHERE simulation_id = $1", id)
                .execute(self.connection().await?)
                .await
                .map_err(|e| anyhow!(e))?;
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    /// Add a node to the nodes table and its components to the node_components table.
    pub async fn add_node(
        &mut self,
        node: Node,
        simulation_id: i32,
        time_step: i32,
    ) -> Result<i32> {
        // add the nodes
        let node_id = query!("INSERT INTO nodes (node_id, simulation_id, time_step, longitude, latitude) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            node.id as i32, simulation_id, time_step, node.longitude, node.latitude)
        .fetch_one(self.connection().await?)
            .await
            .map_err(|e| anyhow!(e))
            .map(|n| n.id)?;
        for component in node.components {
            // add the component
            query!(
                "INSERT INTO node_components (name, node_id, component_data) VALUES ($1, $2, $3)",
                component.0,
                node_id,
                prost_to_serde_json(component.1)
            )
            .execute(self.connection().await?)
            .await?;
        }
        Ok(node_id)
    }

    /// Get all nodes and their components with a `simulation_id` and `time_step` from the nodes table.
    pub async fn get_nodes(&mut self, simulation_id: i32, time_step: i32) -> Result<Vec<Node>> {
        // get the nodes
        let mut nodes: Vec<Node> = query!(
            "SELECT * FROM nodes WHERE simulation_id = $1 AND time_step = $2",
            simulation_id,
            time_step
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|n| Node {
            id: n.node_id as u64,
            longitude: n.longitude,
            latitude: n.latitude,
            components: [].into(),
        })
        .collect();
        for node in &mut nodes {
            // get node components
            node.components = query!(
                "SELECT * FROM node_components WHERE node_id = $1",
                node.id as i32
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|c| (c.name, serde_json_to_prost(c.component_data)))
            .collect();
        }
        Ok(nodes)
    }

    /// Add an edge to the edges table.
    pub async fn add_edge(&mut self, edge: Edge, simulation_id: i32, time_step: i32) -> Result<()> {
        let component_data = prost_to_serde_json(edge.component_data.unwrap());
        let rows_affected = query!(
            "INSERT INTO edges (edge_id, simulation_id, time_step, from_node, to_node, component_data, component_type) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            edge.id as i32, simulation_id, time_step, edge.from as i32, edge.to as i32, component_data, edge.component_type
        ).execute(self.connection().await?).await?.rows_affected();
        if rows_affected == 1 {
            Ok(())
        } else {
            Err(anyhow!("failed to add edge"))
        }
    }

    /// Get all edges with a `simulation_id` and `time_step` from the edges table.
    pub async fn get_edges(&self, simulation_id: i32, time_step: i32) -> Result<Vec<Edge>> {
        let edges = query!(
            "SELECT * FROM edges WHERE simulation_id = $1 AND time_step = $2",
            simulation_id,
            time_step
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|e| Edge {
            from: e.from_node as u64,
            to: e.to_node as u64,
            component_type: e.component_type,
            component_data: Option::from(serde_json_to_prost(e.component_data)),
            id: e.edge_id as u64,
        })
        .collect();
        Ok(edges)
    }

    /// Add a global component to the global_components table.
    pub async fn add_global_component(
        &mut self,
        name: &str,
        value: Value,
        simulation_id: i32,
        time_step: i32,
    ) -> Result<()> {
        query!(
            "INSERT INTO global_components (time_step, name, simulation_id, component_data) VALUES ($1, $2, $3, $4)",
            time_step, name, simulation_id, prost_to_serde_json(value)
        ).execute(self.connection().await?).await?;
        Ok(())
    }

    /// Get all global components with a `simulation_id` and `time_step` from the global_components table.
    pub async fn get_global_components(
        &mut self,
        simulation_id: i32,
        time_step: i32,
    ) -> Result<HashMap<String, Value>> {
        Ok(query!(
            "SELECT * FROM global_components WHERE simulation_id = $1 AND time_step = $2",
            simulation_id,
            time_step
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|c| (c.name, serde_json_to_prost(c.component_data)))
        .collect())
    }
}

#[cfg(feature = "db_test")]
#[cfg(test)]
mod database_test {
    use crate::database::*;
    use prost_types::value::Kind;
    use prost_types::Value;

    #[sqlx::test(migrations = "../migrations/simulator/")]
    async fn test_queue(pool: sqlx::PgPool) {
        let mut db = SimulationsDB::from_pg_pool(pool).await.unwrap();
        let id_1 = db.add_simulation("sim1", 1000, 100).await.unwrap();
        let id_2 = db.add_simulation("sim2", 2000, 200).await.unwrap();
        db.enqueue(id_1).await.unwrap();
        db.enqueue(id_2).await.unwrap();
        let q1 = db.dequeue().await.unwrap();
        let q2 = db.dequeue().await.unwrap();
        let q3 = db.dequeue().await.unwrap();
        assert_eq!(q1, Some(id_1));
        assert_eq!(q2, Some(id_2));
        assert_eq!(q3, None);
    }
    #[sqlx::test(migrations = "../migrations/simulator/")]
    async fn test_add_edge(pool: sqlx::PgPool) {
        let mut db = SimulationsDB::from_pg_pool(pool).await.unwrap();
        db.begin_transaction().await.unwrap();
        let simulation_id = db.add_simulation("sim", 42000, 10).await.unwrap();
        db.add_node(
            Node {
                id: 3,
                latitude: 3.14,
                longitude: 6.28,
                components: [(
                    "first".to_string(),
                    Value {
                        kind: Some(Kind::NumberValue(1.0)),
                    },
                )]
                .into(),
            },
            simulation_id,
            5,
        )
        .await
        .unwrap();
        db.add_node(
            Node {
                id: 4,
                latitude: 6.28,
                longitude: 1.44,
                components: [(
                    "second".to_string(),
                    Value {
                        kind: Some(Kind::NumberValue(2.0)),
                    },
                )]
                .into(),
            },
            simulation_id,
            5,
        )
        .await
        .unwrap();
        db.add_edge(
            Edge {
                from: 3,
                to: 4,
                component_type: "Sometype".to_string(),
                component_data: Some(Value {
                    kind: Some(Kind::NumberValue(42.0)),
                }),
                id: 5,
            },
            simulation_id,
            5,
        )
        .await
        .unwrap();

        db.commit().await.unwrap();

        let edges = db.get_edges(simulation_id, 5).await.unwrap();
        let edges2 = db.get_edges(simulation_id, 5).await.unwrap();
        assert_eq!(edges.len(), 1);
    }
}
