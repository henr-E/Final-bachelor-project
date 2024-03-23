use crate::{error::Error, quantity::Quantity, unit::Unit, SensorStore};
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgInterval, types::BigDecimal};
use std::{borrow::Cow, collections::HashSet};

pub type Signals<'a> = HashSet<Signal<'a>>;

/// Represents a signal field when ingesting sensor data.
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Signal<'a> {
    /// Id of the signal as present in the database.
    pub id: i32,
    /// Name of the field in the data.
    pub name: Cow<'a, str>,
    /// quantity of the signal.
    pub quantity: Quantity,
    /// Unit of the field in the data.
    pub unit: Unit,
    /// Prefix of the value compared to the unit.
    pub prefix: BigDecimal,
}

/// List of values belonging to a [`Signal`].
pub struct SignalValues<'a, 's> {
    pub signal: &'s Signal<'a>,
    pub values: Vec<SignalValue>,
}

/// Value belonging to a [`Signal`] long with a timestamp for the value.
pub struct SignalValue {
    /// Value of the signal.
    pub value: BigDecimal,
    /// Timestamp the value is measured at as ingested into the database.
    pub timestamp: DateTime<Utc>,
}

impl Signal<'_> {
    /// Returns a [`SignalValues`] instance with values whose timestamp is within the provided
    /// `interval` into the past starting from `now`.
    ///
    /// NOTE: This function assumes that the `sensor_signal_id` is unique over all sensors.
    pub async fn values_for_interval_since_now<'s, I>(
        &'s self,
        sensor_store: &SensorStore,
        interval: I,
    ) -> Result<SignalValues<'_, 's>, Error>
    where
        I: TryInto<PgInterval>,
        I::Error: std::error::Error + Send + Sync + 'static,
    {
        let interval = interval
            .try_into()
            .map_err(|e| sqlx::Error::AnyDriverError(e.into()))?;

        let signal_values = sqlx::query!(
            r#"
                SELECT
                    value,
                    timestamp
                FROM sensor_values
                WHERE
                    sensor_signal_id = $1::int
                    AND timestamp > now() - $2::interval
                    AND timestamp < now()
            "#,
            self.id,
            interval,
        )
        .fetch_all(&sensor_store.db_pool)
        .await?;

        let signal_values = signal_values
            .into_iter()
            .map(|sv| SignalValue {
                value: sv.value,
                timestamp: sv.timestamp,
            })
            .collect::<Vec<_>>();

        Ok(SignalValues {
            signal: self,
            values: signal_values,
        })
    }

    /// Returns a [`SignalValues`] instance with values whose timestamp is between the provided
    /// timestamps.
    ///
    /// What timestamp is earlier than the other does not matter as this is flipped automatically.
    ///
    /// NOTE: This function assumes that the `sensor_signal_id` is unique over all sensors.
    pub async fn values_between_timestamps<'s>(
        &'s self,
        sensor_store: &SensorStore,
        start: impl Into<DateTime<Utc>>,
        end: impl Into<DateTime<Utc>>,
    ) -> Result<SignalValues<'_, 's>, Error> {
        let (start, end) = (start.into(), end.into());

        let signal_values = sqlx::query!(
            r#"
                SELECT
                    value,
                    timestamp
                FROM sensor_values
                WHERE
                    sensor_signal_id = $1::int
                    AND timestamp BETWEEN SYMMETRIC
                        $2::TIMESTAMPTZ AND $3::TIMESTAMPTZ
            "#,
            self.id,
            start,
            end,
        )
        .fetch_all(&sensor_store.db_pool)
        .await?;

        let signal_values = signal_values
            .into_iter()
            .map(|sv| SignalValue {
                value: sv.value,
                timestamp: sv.timestamp,
            })
            .collect::<Vec<_>>();

        Ok(SignalValues {
            signal: self,
            values: signal_values,
        })
    }
}
