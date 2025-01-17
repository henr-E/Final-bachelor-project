#![allow(dead_code)]
use std::collections::HashMap;

use anyhow::{anyhow, Context, Ok, Result};
use sqlx::pool::PoolConnection;
use sqlx::types::chrono::NaiveDate;
use sqlx::{query, PgConnection, PgPool, Postgres, Transaction};

use database_config::database_url;
use prost_types::Value;
use prost_value::*;
use proto::simulation::{simulation_manager::SimulationStatus, Edge, Node};
use tonic::Status;

type Date = NaiveDate;

#[derive(Debug, sqlx::Type, PartialEq, Clone)]
#[sqlx(type_name = "enum_status")]
pub enum StatusEnum {
    Pending,
    Computing,
    Finished,
    Failed,
}
impl StatusEnum {
    pub fn to_string(status: StatusEnum) -> String {
        match status {
            StatusEnum::Pending => "Pending",
            StatusEnum::Computing => "Computing",
            StatusEnum::Finished => "Finished",
            _ => "Failed",
        }
        .to_string()
    }
    pub fn from_string(status: &str) -> StatusEnum {
        match status {
            "Pending" => StatusEnum::Pending,
            "Computing" => StatusEnum::Computing,
            "Finished" => StatusEnum::Finished,
            _ => StatusEnum::Failed,
        }
    }

    pub fn to_simulation_status(status: StatusEnum) -> SimulationStatus {
        match status {
            StatusEnum::Pending => SimulationStatus::Pending,
            StatusEnum::Computing => SimulationStatus::Computing,
            StatusEnum::Finished => SimulationStatus::Finished,
            _ => SimulationStatus::Failed,
        }
    }
}
pub struct Simulation {
    pub id: i32,
    pub date: Date,
    pub name: String,
    pub step_size_ms: i32,
    pub max_steps: i32,
    pub status: String,
    pub status_info: Option<String>,
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
        let url = database_url("simulation_manager");
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
    pub async fn connection(&mut self) -> Result<&mut PgConnection> {
        if self.transaction.is_some() {
            return Ok(self
                .transaction
                .as_deref_mut()
                .context("missing transaction")?);
        }
        if self.connection.is_none() {
            self.connection = Some(self.pool.acquire().await.map_err(|e| anyhow!(e))?);
        }
        Ok(self
            .connection
            .as_deref_mut()
            .context("missing connection")?)
    }

    /// Begin a database transaction. The changes won't take effect until commit() is called.
    pub async fn begin_transaction(&mut self) -> Result<()> {
        self.transaction = Some(self.pool.begin().await?);
        Ok(())
    }

    /// Commit all changes since the last call to begin()
    pub async fn commit(&mut self) -> Result<()> {
        let t = self.transaction.take().context("missing transaction")?;
        t.commit().await.map_err(|e| anyhow!(e))?;
        Ok(())
    }

    /// get tick step size
    pub async fn get_delta(&mut self, simulation_id: i32) -> Result<i32> {
        let delta = sqlx::query!(
            "SELECT step_size_ms FROM simulations WHERE id = $1",
            simulation_id
        )
        .fetch_one(self.connection().await?)
        .await?
        .step_size_ms;
        Ok(delta)
    }

    /// Get amount of steps to run for given simulation
    pub async fn get_iterations(&mut self, simulation_id: i32) -> Result<i32> {
        let iterations = sqlx::query!(
            "SELECT max_steps FROM simulations WHERE id = $1",
            simulation_id
        )
        .fetch_one(self.connection().await?)
        .await?
        .max_steps;
        Ok(iterations)
    }

    /// Get list of selected simulator names for given simulation.
    pub async fn get_selected_simulators(
        &mut self,
        simulation_id: i32,
    ) -> Result<Option<Vec<String>>> {
        let selection = sqlx::query!(
            "SELECT simulators FROM simulations WHERE id=$1",
            simulation_id
        )
        .fetch_one(self.connection().await?)
        .await?
        .simulators;

        Ok(selection)
    }

    /// Add a simulation to the simlations table.
    pub async fn add_simulation(
        &mut self,
        name: &str,
        step_size_ms: i32,
        max_steps: i32,
        status: StatusEnum,
        selected_simulators: Vec<String>,
    ) -> Result<i32> {
        query!("INSERT INTO simulations (name, step_size_ms, max_steps, status, simulators) VALUES($1, $2, $3, $4, $5) RETURNING id",
            name, step_size_ms, max_steps, status as _, &selected_simulators)

        .fetch_one(self.connection().await?)
            .await
            .map_err(|e| anyhow!(e))
            .map(|s| s.id)
    }

    /// Delete a simulation from all tables of the database using the name of the simulation.
    pub async fn delete_simulation_via_name(&mut self, name: &str) -> Result<bool> {
        query!("DELETE FROM simulations WHERE name = $1", name)
            .execute(self.connection().await?)
            .await
            .map_err(|e| anyhow!(e))?;
        Ok(true)
    }

    /// Get a simulation from the simulations table using the name.
    pub async fn get_simulation_via_name(&mut self, name: &str) -> Result<Simulation> {
        let result = query!("SELECT id, date, name, step_size_ms, max_steps, status as \"enum_status: StatusEnum \", status_info FROM simulations WHERE name = $1", name)
            .fetch_one(self.connection().await?)
            .await?;
        let sim = Simulation {
            id: result.id,
            date: result.date,
            name: result.name,
            step_size_ms: result.step_size_ms,
            max_steps: result.max_steps,
            status: StatusEnum::to_string(result.enum_status.context("missing: `enum_status`")?),
            status_info: result.status_info,
        };
        Ok(sim)
    }

    /// Get a simulation from the simulations table using the id.
    pub async fn get_simulation_via_id(&mut self, id: i32) -> Result<Simulation> {
        let result = query!("SELECT id, date, name, step_size_ms, max_steps, status as \"enum_status: StatusEnum \", status_info FROM simulations WHERE id = $1", id)
            .fetch_one(self.connection().await?)
            .await?;
        let sim = Simulation {
            id: result.id,
            date: result.date,
            name: result.name,
            step_size_ms: result.step_size_ms,
            max_steps: result.max_steps,
            status: StatusEnum::to_string(result.enum_status.context("missing: `enum_status`")?),
            status_info: result.status_info,
        };
        Ok(sim)
    }

    /// Get the first pending simulation from the simulation database.
    pub async fn get_next_simulation(&mut self) -> Result<Option<i32>> {
        let status: StatusEnum = StatusEnum::from_string("Pending");
        let simulation_id = query!(
            "SELECT id FROM simulations WHERE status = $1 ORDER BY id ASC LIMIT 1",
            status as _
        )
        .fetch_optional(self.connection().await?)
        .await?
        .map(|s| s.id);
        if let Some(id) = simulation_id {
            self.update_status(id, StatusEnum::Computing, None).await?;
        }
        Ok(simulation_id)
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
                prost_to_serde_json(component.1).context("invalid component data")?
            )
            .execute(self.connection().await?)
            .await?;
        }
        Ok(node_id)
    }

    /// Get all nodes with their components from the nodes table. If the components field isn't
    /// None, it only returns those components. If it is None, all components are returned. If
    /// components is empty, no components are returned.
    pub async fn get_nodes_filtered(
        &mut self,
        simulation_id: i32,
        time_step: i32,
        components: Option<Vec<String>>,
    ) -> Result<Vec<Node>> {
        // get the nodes
        let mut records: Vec<_> = query!(
            "SELECT * FROM nodes WHERE simulation_id = $1 AND time_step = $2",
            simulation_id,
            time_step
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .collect();

        let mut nodes = Vec::new();
        for n in &mut records {
            // get node components
            let node_components = match components {
                Some(ref comps) => {
                    query!(
                        "SELECT * FROM node_components WHERE node_id = $1 AND name IN (SELECT unnest($2::text[]))",
                        n.id,
                        comps
                    )
                    .fetch_all(self.connection().await?)
                    .await?
                    .into_iter()
                    .map(|c| Some((c.name, serde_json_to_prost(c.component_data)?)))
                    .collect::<Option<_>>()
                    .context("invalid component in db")?
                }

                None => query!("SELECT * FROM node_components WHERE node_id = $1", n.id)
                    .fetch_all(self.connection().await?)
                    .await?
                    .into_iter()
                    .map(|c| Some((c.name, serde_json_to_prost(c.component_data)?)))
                    .collect::<Option<_>>()
                    .context("invalid component in db")?
            };

            nodes.push(Node {
                id: n.node_id as u64,
                longitude: n.longitude,
                latitude: n.latitude,
                components: node_components,
            });
        }
        Ok(nodes)
    }

    /// Get all nodes and their components with a `simulation_id` and `time_step` from the nodes table.
    pub async fn get_nodes(&mut self, simulation_id: i32, time_step: i32) -> Result<Vec<Node>> {
        self.get_nodes_filtered(simulation_id, time_step, None)
            .await
    }

    /// Get one specific node.
    pub async fn get_node(
        &mut self,
        simulation_id: i32,
        time_step: i32,
        node_id: i32,
    ) -> Result<(Node, i32)> {
        let result1 = query!(
            "SELECT * FROM nodes WHERE simulation_id = $1 AND time_step = $2 AND node_id = $3",
            simulation_id,
            time_step,
            node_id
        )
        .fetch_one(self.connection().await?)
        .await?;

        // get node components
        let result2 = query!(
            "SELECT * FROM node_components WHERE node_id = $1",
            result1.node_id as i32
        )
        .fetch_all(self.connection().await?)
        .await?
        .into_iter()
        .map(|c| Some((c.name, serde_json_to_prost(c.component_data)?)))
        .collect::<Option<_>>()
        .context("invalid component in db")?;

        let node: (Node, i32) = (
            Node {
                id: result1.node_id as u64,
                longitude: result1.longitude,
                latitude: result1.latitude,
                components: result2,
            },
            result1.id,
        );
        Ok(node)
    }

    /// Add an edge to the edges table.
    pub async fn add_edge(&mut self, edge: Edge, simulation_id: i32, time_step: i32) -> Result<()> {
        let component_data =
            prost_to_serde_json(edge.component_data.context("missing component data")?)
                .context("invalid component data")?;
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
    pub async fn get_edges(&mut self, simulation_id: i32, time_step: i32) -> Result<Vec<Edge>> {
        let edges = query!(
            "SELECT * FROM edges WHERE simulation_id = $1 AND time_step = $2",
            simulation_id,
            time_step
        )
        .fetch_all(self.connection().await?)
        .await?
        .into_iter()
        .map(|e| {
            Ok(Edge {
                from: e.from_node as u64,
                to: e.to_node as u64,
                component_type: e.component_type,
                component_data: Some(
                    serde_json_to_prost(e.component_data).context("invalid component in db")?,
                ),
                id: e.edge_id as u64,
            })
        })
        .collect::<Result<_>>()?;
        Ok(edges)
    }

    /// Get one specific edge.
    pub async fn get_edge(
        &mut self,
        simulation_id: i32,
        time_step: i32,
        edge_id: i32,
    ) -> Result<(Edge, i32)> {
        let result = query!(
            "SELECT * FROM edges WHERE simulation_id = $1 AND time_step = $2 AND edge_id = $3",
            simulation_id,
            time_step,
            edge_id
        )
        .fetch_one(self.connection().await?)
        .await?;
        let edge = (
            Edge {
                id: result.edge_id as u64,
                from: result.from_node as u64,
                to: result.to_node as u64,
                component_type: result.component_type,
                component_data: Some(
                    serde_json_to_prost(result.component_data)
                        .context("invalid component in db")?,
                ),
            },
            result.id,
        );
        Ok(edge)
    }
    pub async fn get_node_max_timestep(&mut self, simulation_id: i32) -> Result<i32> {
        let node_timestep = sqlx::query!(
            "SELECT time_step FROM nodes WHERE simulation_id = $1 ORDER BY time_step DESC",
            simulation_id
        )
        .fetch_optional(self.connection().await?)
        .await
        .map_err(|err| Status::from_error(Box::new(err)))?
        .map(|t| t.time_step)
        .unwrap_or(0);
        Ok(node_timestep)
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
            time_step, name, simulation_id, prost_to_serde_json(value).context("invalid component data")?
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
        .fetch_all(self.connection().await?)
        .await?
        .into_iter()
        .map(|c| Some((c.name, serde_json_to_prost(c.component_data)?)))
        .collect::<Option<_>>()
        .context("invalid component in db")?)
    }

    /// Get specific global components.
    pub async fn get_specific_global_components(
        &mut self,
        simulation_id: i32,
        time_step: i32,
        name: &str,
    ) -> Result<(String, Value)> {
        let components = query!("SELECT * FROM global_components WHERE simulation_id = $1 AND time_step = $2 AND name = $3",
            simulation_id,
            time_step,
            name
        )
            .fetch_one(self.connection().await?)
            .await?;
        Ok((
            components.name,
            serde_json_to_prost(components.component_data).context("invalid component in db")?,
        ))
    }

    /// Get all global components of a simulation regardless of the time_step.
    pub async fn get_all_global_components(
        &mut self,
        simulation_id: i32,
    ) -> Result<HashMap<String, Value>> {
        Ok(query!(
            "SELECT * FROM global_components WHERE simulation_id = $1",
            simulation_id
        )
        .fetch_all(self.connection().await?)
        .await?
        .into_iter()
        .map(|c| Some((c.name, serde_json_to_prost(c.component_data)?)))
        .collect::<Option<_>>()
        .context("invalid component in db")?)
    }

    /// Return the highest time_step of all global components of a simulation.
    pub async fn get_global_components_max_timestep(&mut self, simulation_id: i32) -> Result<i32> {
        let component_timestep = sqlx::query!(
            "SELECT time_step FROM global_components WHERE simulation_id = $1 ORDER BY time_step DESC",
            simulation_id
        )
            .fetch_optional(self.connection().await?)
            .await
            .map_err(|err| Status::from_error(Box::new(err)))?
            .map(|t| t.time_step)
            .unwrap_or(0);
        Ok(component_timestep)
    }

    /// Get a status of the simulation.
    pub async fn get_status(&mut self, simulation_id: i32) -> Result<StatusEnum> {
        let status = query!(
            "SELECT status as \"status: StatusEnum\" FROM simulations WHERE id = $1",
            simulation_id
        )
        .fetch_one(self.connection().await?)
        .await
        .map_err(|err| Status::from_error(Box::new(err)))?
        .status
        .context("missing status")?;

        Ok(status)
    }
    /// Update the status of the simulation.
    pub async fn update_status(
        &mut self,
        simulation_id: i32,
        status: StatusEnum,
        status_info: Option<&str>,
    ) -> Result<()> {
        let info = status_info.unwrap_or("");
        if info.is_empty() {
            query!(
                "UPDATE simulations SET status = $1 WHERE id = $2",
                status as _,
                simulation_id
            )
            .execute(self.connection().await?)
            .await?;
        } else {
            query!(
                "UPDATE simulations SET status = $1, status_info = $2 WHERE id = $3",
                status as _,
                info,
                simulation_id
            )
            .execute(self.connection().await?)
            .await?;
        }
        Ok(())
    }
}

#[cfg(feature = "db_test")]
#[cfg(test)]
mod database_test {
    use crate::database::*;
    use prost_types::value::Kind;
    use prost_types::Value;

    #[sqlx::test(migrations = "../migrations/simulator/")]
    async fn test_add_edge(pool: sqlx::PgPool) {
        let mut db = SimulationsDB::from_pg_pool(pool).await.unwrap();
        db.begin_transaction().await.unwrap();
        let simulation_id = db
            .add_simulation("sim", 42000, 10, StatusEnum::Pending, vec![])
            .await
            .unwrap();
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
        assert_eq!(edges.len(), 1);
    }

    #[sqlx::test(migrations = "../migrations/simulator/")]
    async fn test_nodes_filtered(pool: sqlx::PgPool) {
        let mut db = SimulationsDB::from_pg_pool(pool).await.unwrap();
        let simulation_id = db
            .add_simulation("sim", 42000, 10, StatusEnum::Pending, vec![])
            .await
            .unwrap();
        db.add_node(
            Node {
                id: 3,
                latitude: 3.14,
                longitude: 6.28,
                components: [
                    (
                        "first".to_string(),
                        Value {
                            kind: Some(Kind::NumberValue(1.0)),
                        },
                    ),
                    (
                        "second".to_string(),
                        Value {
                            kind: Some(Kind::NumberValue(2.0)),
                        },
                    ),
                    (
                        "third".to_string(),
                        Value {
                            kind: Some(Kind::NumberValue(3.0)),
                        },
                    ),
                    (
                        "fourth".to_string(),
                        Value {
                            kind: Some(Kind::NumberValue(4.0)),
                        },
                    ),
                ]
                .into(),
            },
            simulation_id,
            5,
        )
        .await
        .unwrap();
        let unfiltered = db.get_nodes_filtered(simulation_id, 5, None).await.unwrap();
        let filtered = db
            .get_nodes_filtered(
                simulation_id,
                5,
                Some(vec!["second".to_string(), "third".to_string()]),
            )
            .await
            .unwrap();
        assert_eq!(unfiltered[0].components.len(), 4);
        assert_eq!(filtered[0].components.len(), 2);
        assert!(filtered[0].components.contains_key("second"));
        assert!(!filtered[0].components.contains_key("first"));
        assert!(unfiltered[0].components.contains_key("first"));
    }
}
