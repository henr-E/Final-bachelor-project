#![doc = include_str!("../README.md")]

use crate::{error::Error, quantity::*, sensor::Sensor, unit::*};
use database_config::database_url;
use sqlx::types::BigDecimal;
use sqlx::PgPool;
use std::{borrow::Cow, collections::HashSet};

pub mod error;
pub mod quantity;
pub mod sensor;
pub mod unit;

/// Sensor database wrapper.
pub struct SensorStore {
    db_pool: PgPool,
}

pub type Signals<'a> = HashSet<Signal<'a>>;

/// Represents a signal field when ingesting sensor data.
#[derive(PartialEq, Eq, Hash)]
pub struct Signal<'a> {
    /// Name of the field in the data.
    pub name: Cow<'a, str>,
    /// quantity of the signal.
    pub quantity: Quantity,
    /// Unit of the field in the data.
    pub unit: Unit,
    /// Prefix of the value compared to the unit.
    pub prefix: BigDecimal,
}

impl SensorStore {
    /// Create a new [`SensorStore`] using environment variables from the `.env` file.
    ///
    /// See [`database_config::database_url`] for more info.
    pub async fn new() -> Result<Self, Error> {
        let db_url = database_url("SENSOR_ARCHIVE", "SENSOR", None, None);
        let db_pool = PgPool::connect(&db_url).await?;
        Ok(Self { db_pool })
    }

    /// Create a [`SensorStore`] from an already created postgres connection.
    pub fn from_pg_pool(db_pool: &PgPool) -> Self {
        Self {
            db_pool: db_pool.clone(),
        }
    }

    /// Get a single [`Sensor`] from the database given its id.
    pub async fn get(&self, sensor_id: uuid::Uuid) -> Result<Sensor<'static>, Error> {
        let sensor = sqlx::query!(
            "SELECT name, description FROM sensors WHERE id = $1::uuid",
            sensor_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        let mut sensor = Sensor::builder(sensor.name, sensor.description);
        for sensor_signal in sqlx::query!(
            r#"
                SELECT
                    alias,
                    quantity AS "quantity!: Quantity",
                    unit AS "unit!: Unit",
                    prefix
                FROM sensor_signals
                WHERE sensor_id = $1::uuid
            "#,
            sensor_id
        )
        .fetch_all(&self.db_pool)
        .await?
        {
            sensor.add_signal(
                sensor_signal.alias,
                sensor_signal.quantity,
                sensor_signal.unit,
                sensor_signal.prefix,
            );
        }

        Ok(sensor.build())
    }
}
