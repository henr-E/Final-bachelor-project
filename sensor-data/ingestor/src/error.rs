use thiserror::Error;

/// Custom error type to use for this crate. Prevents having to return a boxed error and losing all
/// type information.
#[derive(Error, Debug)]
pub(crate) enum DataIngestError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("sqlx database error: `{0}`")]
    Sqlx(#[from] sqlx::Error),
    #[error("sqlx migration error: `{0}`")]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),
    #[error("tonic transport error: `{0}`")]
    TonicServe(#[from] tonic::transport::Error),
}

/// Custom error for the `register_to_database` function.
#[derive(Error, Debug)]
pub(crate) enum DatabaseRegisterError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
