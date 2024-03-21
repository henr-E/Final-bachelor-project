use environment_config::{env, is_truthy, Error as EnvError};
use secrets::secret;
use std::path::Path;
use thiserror::Error;
use tracing::warn;

const SHOULD_DO_MIGRATIONS_VAR: &str = "DO_MIGRATIONS";
const POSTGRES_USER_PASSWORD_VAR: &str = "DATABASE_PASSWORD";

/// Error type for this crate. Any public function that returns a `Result` returns this error as
/// the error value.
#[derive(Error, Debug)]
pub enum Error {
    #[error("environment config error: {0}")]
    EnvError(#[from] environment_config::Error),
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("sqlx migration error: {0}")]
    SqlxMigration(#[from] sqlx::migrate::MigrateError),
}

/// Runs migrations in `migrations_path` on the database connected to with the specified
/// environment variables. User "postgres" will be used for authentication.
///
/// * database: $`db_name_env`_DB_NAME
/// * password: $DATABASE_PASSWORD
pub async fn do_migrations_if_enabled(
    migrations_path: impl AsRef<Path>,
    db_name: &str,
) -> Result<(), Error> {
    let should_do_migrations = match env(SHOULD_DO_MIGRATIONS_VAR) {
        Ok(val) => is_truthy(val),
        Err(EnvError::VarNotFound(_)) => false,
        Err(err) => return Err(err.into()),
    };

    if !should_do_migrations {
        return Ok(());
    }

    let db_url = database_url(db_name);

    let migrator = sqlx::migrate::Migrator::new(migrations_path.as_ref()).await?;
    let db_pool = sqlx::PgPool::connect(&db_url).await?;
    migrator.run(&db_pool).await?;
    // Set table permissions.
    let user = database_user();
    sqlx::query(&format!(
        "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public to {}",
        user
    ))
    .execute(&db_pool)
    .await?;
    sqlx::query(&format!(
        "GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public to {}",
        user,
    ))
    .execute(&db_pool)
    .await?;

    Ok(())
}

/// Sets the `DATABASE_URL` rustc environment variable. This needs to be set to compile [`sqlx`]
/// queries. Will do nothing if `SQLX_OFFLINE` is enabled.
pub fn set_database_url(db_name: &str) -> Result<(), Error> {
    let sqlx_offline = env("SQLX_OFFLINE").map(is_truthy).unwrap_or(false);
    if sqlx_offline {
        return Ok(());
    }
    println!("cargo:rustc-env=DATABASE_URL={}", database_url(db_name));
    Ok(())
}

/// Configures cargo to recompile the crate when any of the paths contain an updated file.
pub fn configure_recompile(migrations_path: &str, dotenv_path: &str, databases_path: &str) {
    println!("cargo:rerun-if-changed={}", migrations_path);
    println!("cargo:rerun-if-changed={}", dotenv_path);
    println!("cargo:rerun-if-changed={}", databases_path);
}

/// Returns a postgres database url for use at runtime.
/// NOTE: This will not use the callers environment, but will look for the closest `.env` file to
/// take variables from. This is done to maintain a single source of truth.
///
/// * database: $`db_name_env`_DB_NAME
/// * host: $`db_host_env`_DB_HOST
/// * port: $`db_port_env`_DB_PORT
///
/// If a variable is `None` or does not exist, the postgres default will be assumed.
pub fn database_url(db_name: &str) -> String {
    let password = secret(POSTGRES_USER_PASSWORD_VAR)
        .unwrap_or_else(|| {
            let default = "postgres".to_string();
            warn!("Secret variable {POSTGRES_USER_PASSWORD_VAR} not found, using default value `{default}`");
            default
        });
    let user = database_user();
    let host = env("DATABASE_HOST").unwrap_or_else(|_| {
        let default = "localhost";
        warn!("Environment variable \"DATABASE_HOST\" not found, using default value `{default}`");
        default
    });
    let port = env("DATABASE_PORT").unwrap_or_else(|_| {
        let default = "5432";
        warn!("Environment variable \"DATABASE_PORT\" not found, using default value `{default}`");
        default
    });

    format!(
        "postgres://{}:{}@{}:{}/{}",
        user, password, host, port, db_name
    )
}

fn database_user() -> String {
    env("DATABASE_USER")
        .unwrap_or_else(|_| {
            let default = "postgres";
            warn!(
                "Environment variable \"DATABASE_USER\" not found, using default value `{default}`"
            );
            default
        })
        .to_string()
}
