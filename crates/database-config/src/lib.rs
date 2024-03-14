use environment_config::{env, is_truthy, Error as EnvError};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
};

use thiserror::Error;

const SHOULD_DO_MIGRATIONS_VAR: &str = "DO_MIGRATIONS";
const POSTGRES_USER_PASSWORD_VAR: &str = "POSTGRES_USER_PASSWORD";
const DEFAULT_HOST: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5432);

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
/// * password: $POSTGRES_USER_PASSWORD
pub async fn do_migrations_if_enabled(
    migrations_path: impl AsRef<Path>,
    db_name_env: &str,
    host_and_port: Option<impl Into<SocketAddr>>,
) -> Result<(), Error> {
    let should_do_migrations = match env(SHOULD_DO_MIGRATIONS_VAR) {
        Ok(val) => is_truthy(val),
        Err(EnvError::VarNotFound(_)) => false,
        Err(err) => return Err(err.into()),
    };

    if !should_do_migrations {
        return Ok(());
    }

    let database = env(format!("{}{}", db_name_env, "_DB_NAME"))?;
    let postgres_user_password = env(POSTGRES_USER_PASSWORD_VAR)?;
    let db_url = postgres_database_url("postgres", postgres_user_password, host_and_port, database);

    let migrator = sqlx::migrate::Migrator::new(migrations_path.as_ref()).await?;
    let db_pool = sqlx::PgPool::connect(&db_url).await?;
    migrator.run(&db_pool).await?;
    // Set table permissions.
    sqlx::query(&format!(
        "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public to {}",
        database
    ))
    .execute(&db_pool)
    .await?;
    sqlx::query(&format!(
        "GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public to {}",
        database
    ))
    .execute(&db_pool)
    .await?;

    Ok(())
}

/// Sets the `DATABASE_URL` rustc environment variable. This needs to be set to compile [`sqlx`]
/// queries. Will do nothing if `SQLX_OFFLINE` is enabled.
pub fn set_database_url(
    db_name_env: &str,
    db_pass_env: &str,
    host_and_port: Option<impl Into<SocketAddr>>,
) -> Result<(), Error> {
    let sqlx_offline = env("SQLX_OFFLINE").map(is_truthy).unwrap_or(false);
    if sqlx_offline {
        return Ok(());
    }
    let database = env(format!("{}{}", db_name_env, "_DB_NAME"))?;
    let password = env(format!("{}{}", db_pass_env, "_DB_PASSWORD"))?;
    println!(
        "cargo:rustc-env=DATABASE_URL={}",
        postgres_database_url(database, password, host_and_port, database)
    );
    Ok(())
}

/// Sets the `DATABASE_URL` rustc environment variable just like set_database_url but for the admin user
pub fn set_database_url_admin(
    db_name_env: &str,
    host_and_port: Option<impl Into<SocketAddr>>,
) -> Result<(), Error> {
    let sqlx_offline = env("SQLX_OFFLINE").map(is_truthy).unwrap_or(false);
    if sqlx_offline {
        return Ok(());
    }
    let database = env(format!("{}{}", db_name_env, "_DB_NAME"))?;
    println!(
        "cargo:rustc-env=DATABASE_URL={}",
        postgres_database_url(
            "postgres",
            env("POSTGRES_USER_PASSWORD")?,
            host_and_port,
            database
        )
    );
    Ok(())
}

/// Configures cargo to recompile the crate when any of the paths contain an updated file.
pub fn configure_recompile(migrations_path: &str, dotenv_path: &str) {
    println!("cargo:rerun-if-changed={}", migrations_path);
    println!("cargo:rerun-if-changed={}", dotenv_path);
}

/// Returns a postgres database url for use at runtime.
/// NOTE: This will not use the callers environment, but will look for the closest `.env` file to
/// take variables from. This is done to maintain a single source of truth.
///
/// * database: $`db_name_env`_DB_NAME
/// * password: $`db_pass_env`_DB_PASSWORD
/// * host: $`db_host_env`_DB_HOST
/// * port: $`db_port_env`_DB_PORT
///
/// If a variable is `None` or does not exist, the postgres default will be assumed.
pub fn database_url(
    db_name_env: &str,
    db_pass_env: &str,
    db_host_env: Option<&str>,
    db_port_env: Option<&str>,
) -> String {
    let database = env(format!("{}{}", db_name_env, "_DB_NAME")).unwrap_or("postgres");
    let password = env(format!("{}{}", db_pass_env, "_DB_PASSWORD")).unwrap_or("postgres");
    let host = db_host_env
        .and_then(|h| env(format!("{}{}", h, "_DB_HOST")).ok())
        .unwrap_or("localhost");
    let port = db_port_env
        .and_then(|h| env(format!("{}{}", h, "_DB_PORT")).ok())
        .unwrap_or("5432");

    format!(
        "postgres://{}:{}@{}:{}/{}",
        database, password, host, port, database
    )
}

/// Constructs a postgres database url from the given values.
fn postgres_database_url(
    user: &str,
    password: &str,
    host_and_port: Option<impl Into<SocketAddr>>,
    database: &str,
) -> String {
    format!(
        "postgres://{}:{}@{}/{}",
        user,
        password,
        host_and_port.map(|h| h.into()).unwrap_or(DEFAULT_HOST),
        database
    )
}
