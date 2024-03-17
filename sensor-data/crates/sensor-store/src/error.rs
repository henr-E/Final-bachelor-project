use thiserror::Error;

/// Sensor store error type used throughout the crate and returned to the user.
#[derive(Error, Debug)]
pub enum Error {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}
