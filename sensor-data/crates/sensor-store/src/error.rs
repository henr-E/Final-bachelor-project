use thiserror::Error;

/// Sensor store error type used throughout the crate and returned to the user.
#[derive(Error, Debug)]
pub enum Error {
    #[error("sensor id not found")]
    SensorIdNotFound,
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}
