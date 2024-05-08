#![doc = include_str!("../README.md")]

use std::collections::HashMap;

use futures::stream::Stream;
use sqlx::{Error as SqlxError, PgPool, Postgres, Transaction};
use uuid::Uuid;

use database_config::database_url;
use sensor::SensorBuilder;

pub use crate::{
    error::Error,
    quantity::Quantity,
    sensor::Sensor,
    signal::{Signal, Signals},
    unit::Unit,
};

pub mod error;
pub mod quantity;
pub mod sensor;
pub mod signal;
pub mod unit;

/// Sensor database wrapper.
#[derive(Debug, Clone)]
pub struct SensorStore {
    db_pool: PgPool,
}

impl SensorStore {
    /// Create a new [`SensorStore`] using environment variables from the `.env` file.
    ///
    /// See [`database_url`] for more info.
    pub async fn new() -> Result<Self, Error> {
        let db_url = database_url("sensor_archive");
        let db_pool = PgPool::connect(&db_url).await?;
        Ok(Self { db_pool })
    }

    /// Create a [`SensorStore`] from an already created postgres connection.
    pub fn from_pg_pool(db_pool: &PgPool) -> Self {
        Self {
            db_pool: db_pool.clone(),
        }
    }

    /// Get a single [`Sensor`] from the database given its [`Uuid`].
    pub async fn get_sensor(&self, sensor_id: Uuid) -> Result<Sensor<'_>, Error> {
        let sensor = match sqlx::query!(
            "SELECT name, description, location[0]::float as lon, location[1]::float as lat, twin_id, building_id FROM sensors WHERE id = $1::uuid",
            sensor_id
        )
        .fetch_one(&self.db_pool)
        .await {
                Ok(s) => s,
                Err(e) => return Err(match e{
                    SqlxError::RowNotFound => Error::SensorIdNotFound,
                    e => e.into(),
                })
            };

        // NOTE: unwrapping on location is ok since there is a `NOT NULL` constraint.
        let sensor = Sensor::builder(
            sensor_id,
            sensor.name,
            sensor.description,
            (sensor.lon.unwrap(), sensor.lat.unwrap()),
            sensor.twin_id,
            sensor.building_id,
        );
        Ok(self.add_signals_to_builder(sensor).await?.build())
    }

    /// Get a single [`Sensor`] from the database given its building id.
    pub async fn get_sensor_for_building(&self, building_id: i32) -> Result<Sensor<'_>, Error> {
        let sensor = sqlx::query!(
            "SELECT id::uuid, name, description, location[0]::float as lon, location[1]::float as lat, twin_id, building_id FROM sensors WHERE building_id = $1::int",
            building_id
        )
            .fetch_one(&self.db_pool)
            .await?;

        // NOTE: unwrapping on location is ok since there is a `NOT NULL` constraint.
        let sensor = Sensor::builder(
            sensor.id,
            sensor.name,
            sensor.description,
            (sensor.lon.unwrap(), sensor.lat.unwrap()),
            sensor.twin_id,
            sensor.building_id,
        );
        Ok(self.add_signals_to_builder(sensor).await?.build())
    }

    /// Set a single [`Sensor`] into the database, returning its [`Uuid`].
    pub async fn store_sensor(&self, sensor: Sensor<'_>) -> Result<Uuid, Error> {
        // create a transaction. if any error occurs,
        // no commits are performed.
        let mut transaction = self
            .db_pool
            .begin()
            .await
            .expect("Couldn't start transaction");
        // generate new uuid for this sensor.
        let sensor_id = sensor.id;
        // insert sensor into the database.
        let _sensor = sqlx::query!(
            "INSERT INTO sensors (id, name, description, location, user_id, twin_id, building_id) values ($1::uuid, $2::text, $3::text, POINT($4::float, $5::float), $6::int, $7::int, $8)",
            sensor_id, &sensor.name, &sensor.description.clone().unwrap_or_default(), sensor.location.0, sensor.location.1, 1, sensor.twin_id, sensor.building_id
        )
        .execute(&mut *transaction)
        .await?;
        // add the signals of this sensor to the transaction.
        self.store_signals(sensor.signals(), &mut transaction, sensor_id)
            .await?;
        // commit transaction to the database.
        transaction.commit().await?;

        // if everything happens correctly,
        // return the generated sensor_id to the caller.
        Ok(sensor_id)
    }

    /// Commit all signals of a single [`Sensor`] to the database.
    ///
    /// This function is called from the [`SensorStore::set_sensor`] method.
    async fn store_signals(
        &self,
        signals: &Signals<'_>,
        transaction: &mut Transaction<'_, Postgres>,
        sensor_id: Uuid,
    ) -> Result<(), SqlxError> {
        let signals: Vec<Signal> = Vec::from_iter(signals.iter().cloned());
        // create a query builder to batch insert signals.
        let names: Vec<_> = signals.iter().map(|item| item.name.to_string()).collect();
        let quantities: Vec<_> = signals
            .iter()
            .map(|item| item.quantity.to_string())
            .collect();
        let units: Vec<_> = signals.iter().map(|item| item.unit.to_string()).collect();
        let prefixes: Vec<_> = signals.iter().map(|item| item.prefix.clone()).collect();
        let _res = sqlx::query!(
            r#"
                INSERT INTO sensor_signals (sensor_id, alias, quantity, unit, prefix)
                SELECT $1::uuid, alias, quantity::quantity, unit::unit, prefix
                FROM UNNEST($2::text[], $3::text[], $4::text[], $5::decimal[])
                    AS x(alias, quantity, unit, prefix);
            "#,
            sensor_id,
            &names,
            &quantities,
            &units,
            &prefixes
        )
        .execute(&mut **transaction)
        .await?;
        Ok(())
    }

    /// Delete a [`Sensor`] from the database.
    pub async fn delete_sensor(&self, sensor_id: Uuid) -> Result<(), SqlxError> {
        println!("{:?}", sensor_id);
        match sqlx::query!("DELETE FROM sensors WHERE id = $1::uuid", sensor_id)
            .execute(&self.db_pool)
            .await?
            .rows_affected()
        {
            1 => Ok(()),
            _ => Err(SqlxError::RowNotFound),
        }
    }

    /// Get all sensors from the database.
    pub async fn get_all_sensors(
        &self,
    ) -> Result<impl Stream<Item = Result<Sensor, Error>>, Error> {
        use futures::stream::{self, StreamExt};

        let sensors = sqlx::query!(
            "SELECT id, name, description, location[0] as lon, location[1] as lat, twin_id, building_id FROM sensors"
        )
        .fetch_all(&self.db_pool)
        .await?
        .into_iter()
        .map(|s| {
            Sensor::builder(
                s.id,
                s.name,
                s.description,
                (s.lon.unwrap(), s.lat.unwrap()),
                s.twin_id,
                s.building_id
            )
        });

        let sensors = stream::iter(sensors).then(|s| async {
            self.add_signals_to_builder(s)
                .await
                .map(SensorBuilder::build)
        });

        Ok(sensors)
    }

    pub async fn get_all_sensors_for_twin(
        &self,
        twin_id: i32,
    ) -> Result<impl Stream<Item = Result<Sensor, Error>>, Error> {
        use futures::stream::{self, StreamExt};

        let sensors = sqlx::query!(
            "SELECT id, name, description, location[0] as lon, location[1] as lat, twin_id, building_id FROM sensors WHERE twin_id = $1::int",
            twin_id
        )
            .fetch_all(&self.db_pool)
            .await?
            .into_iter()
            .map(|s| {
                Sensor::builder(
                    s.id,
                    s.name,
                    s.description,
                    (s.lon.unwrap(), s.lat.unwrap()),
                    s.twin_id,
                    s.building_id
                )
            });

        let sensors = stream::iter(sensors).then(|s| async {
            self.add_signals_to_builder(s)
                .await
                .map(SensorBuilder::build)
        });

        Ok(sensors)
    }

    /// Retrieves all global sensors. Global sensors have a building_id equal to 'NULL'.
    pub async fn get_all_global_sensors(
        &self,
    ) -> Result<impl Stream<Item = Result<Sensor, Error>>, Error> {
        use futures::stream::{self, StreamExt};

        let sensors = sqlx::query!(
            "SELECT id, name, description, location[0] as lon, location[1] as lat, twin_id, building_id FROM sensors WHERE building_id IS NULL"
        )
            .fetch_all(&self.db_pool)
            .await?
            .into_iter()
            .map(|s| {
                Sensor::builder(
                    s.id,
                    s.name,
                    s.description,
                    (s.lon.unwrap(), s.lat.unwrap()),
                    s.twin_id,
                    s.building_id
                )
            });

        let sensors = stream::iter(sensors).then(|s| async {
            self.add_signals_to_builder(s)
                .await
                .map(SensorBuilder::build)
        });

        Ok(sensors)
    }

    /// Get the total amount of values for a signal.
    pub async fn get_sensor_signal_value_count(&self) -> Result<HashMap<Uuid, u64>, Error> {
        let mut hashmap = HashMap::new();
        let sensor_counts = sqlx::query!("SELECT ss.sensor_id, COUNT(*) FROM sensor_values AS sv NATURAL JOIN sensor_signals AS ss GROUP BY ss.sensor_id")
        .fetch_all(&self.db_pool)
        .await?;
        sensor_counts.into_iter().for_each(|s| {
            hashmap.insert(s.sensor_id, s.count.unwrap_or_default() as u64);
        });
        Ok(hashmap)
    }

    async fn add_signals_to_builder<'a>(
        &self,
        mut sensor_builder: SensorBuilder<'a>,
    ) -> Result<SensorBuilder<'a>, Error> {
        for sensor_signal in sqlx::query!(
            r#"
                SELECT
                    sensor_signal_id AS id,
                    alias,
                    quantity AS "quantity!: Quantity",
                    unit AS "unit!: Unit",
                    prefix
                FROM sensor_signals
                WHERE sensor_id = $1::uuid
            "#,
            sensor_builder.id
        )
        .fetch_all(&self.db_pool)
        .await?
        {
            sensor_builder.add_signal(
                sensor_signal.id,
                sensor_signal.alias,
                sensor_signal.quantity,
                sensor_signal.unit,
                sensor_signal.prefix,
            );
        }

        Ok(sensor_builder)
    }
}

#[cfg(test)]
mod unit_quantity_tests {
    use enumset::EnumSet;

    use super::{Quantity, Unit};

    /// Ensure that every unit belongs to a quantity and vica versa. This is done to find unused
    /// quantities and wrongly associated units.
    #[test]
    fn complete_set() {
        let all_units = EnumSet::<Unit>::all();
        let all_quantities = EnumSet::<Quantity>::all();

        assert_eq!(
            all_quantities
                .iter()
                .flat_map(|q| q.associated_units())
                .collect::<EnumSet<_>>(),
            all_units
        );

        assert_eq!(
            all_units
                .iter()
                .map(|u| u.associated_quantity())
                .collect::<EnumSet<_>>(),
            all_quantities
        );
    }

    /// Ensure that every quantity's base unit is also an associated unit.
    #[test]
    fn base_unit_in_associated_units() {
        let all_quantities = EnumSet::<Quantity>::all();

        assert!(all_quantities
            .into_iter()
            .all(|q| q.associated_units().contains(&q.associated_base_unit())))
    }

    #[test]
    fn associated_unit_sets_are_disjoint_or_equal() {
        let all_quantities = EnumSet::<Quantity>::all();

        let unit_sets = all_quantities
            .iter()
            .map(|q| q.associated_units())
            .collect::<Vec<_>>();

        // Not an optimal implementation as this will do double work.
        // Should be fine as this is a test case.
        for unit_set in unit_sets.iter() {
            assert!(unit_sets
                .iter()
                .all(|s| s == unit_set || s.is_disjoint(unit_set)));
        }
    }

    #[test]
    fn quantities_string_round_trip() {
        use std::str::FromStr;

        let all_quantities = EnumSet::<Quantity>::all();

        let quantities_round_trip = all_quantities
            .iter()
            .map(|q| q.to_string())
            .map(|s| Quantity::from_str(&s));

        let quantities_round_trip = quantities_round_trip
            .collect::<Result<EnumSet<_>, _>>()
            .unwrap();

        assert_eq!(quantities_round_trip, all_quantities);
    }

    #[test]
    fn units_string_round_trip() {
        use std::str::FromStr;

        let all_units = EnumSet::<Unit>::all();

        let units_round_trip = all_units
            .iter()
            .map(|q| q.to_string())
            .map(|s| Unit::from_str(&s));

        let units_round_trip = units_round_trip.collect::<Result<EnumSet<_>, _>>().unwrap();

        assert_eq!(units_round_trip, all_units);
    }
}
