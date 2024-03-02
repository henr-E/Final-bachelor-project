use anyhow::Error as AnyhowError;
use once_cell::sync::OnceCell;
use std::{
    borrow::Cow,
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
};
use thiserror::Error;

const SHOULD_DO_MIGRATIONS_VAR: &str = "COMPILE_TIME_MIGRATIONS";
const POSTGRES_USER_PASSWORD_VAR: &str = "POSTGRES_USER_PASSWORD";
const DEFAULT_HOST: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5432);

static ENV_VARS: OnceCell<HashMap<String, String>> = OnceCell::new();

/// Error type for this crate. Any public function that returns a `Result` returns this error as
/// the error value.
#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("`.env` file could not be found")]
    EnvFileNotFound,
    #[error("{0}")]
    Generic(#[from] anyhow::Error),
    #[error("following variables not found in `.env`: `{0:?}`")]
    VarNotFound(Cow<'a, str>),
    #[error("error parsing `.env`: {0}")]
    Dotenvy(#[from] dotenvy::Error),
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
    let should_do_migrations = match get_env_variable(SHOULD_DO_MIGRATIONS_VAR) {
        Ok(val) => value_is_truthy(val),
        Err(Error::VarNotFound(_)) => false,
        Err(err) => return Err(err),
    };

    if !should_do_migrations {
        return Ok(());
    }

    let database = get_env_variable(format!("{}{}", db_name_env, "_DB_NAME"))?;
    let postgres_user_password = get_env_variable(POSTGRES_USER_PASSWORD_VAR)?;
    let db_url = postgres_database_url("postgres", postgres_user_password, host_and_port, database);

    let migrator = sqlx::migrate::Migrator::new(migrations_path.as_ref()).await?;
    let db_pool = sqlx::PgPool::connect(&db_url).await?;
    migrator.run(&db_pool).await?;

    Ok(())
}

/// Sets the `DATABASE_URL` rustc environment variable. This needs to be set to compile [`sqlx`]
/// queries.
pub fn set_database_url<'a>(
    db_name_env: &'a str,
    db_pass_env: &'a str,
    host_and_port: Option<impl Into<SocketAddr>>,
) -> Result<(), Error<'a>> {
    let database = get_env_variable(format!("{}{}", db_name_env, "_DB_NAME"))?;
    let password = get_env_variable(format!("{}{}", db_pass_env, "_DB_PASSWORD"))?;
    println!(
        "cargo:rustc-env=DATABASE_URL={}",
        postgres_database_url(database, password, host_and_port, database)
    );
    Ok(())
}

/// Configures cargo to recompile the crate when any of the paths contain an updated file.
pub fn configure_recompile(migrations_path: &str, dotenv_path: &str) {
    println!("cargo:rerun-if-changed={}", migrations_path);
    println!("cargo:rerun-if-changed={}", dotenv_path);
}

/// Constructs a postgres database url from the given values.
pub fn postgres_database_url(
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

fn value_is_truthy(val: &str) -> bool {
    // Implementation taken from https://github.com/sagiegurari/envmnt/blob/master/src/util.rs
    let val = val.to_lowercase();
    !val.is_empty() && val != "0" && val != "false" && val != "no"
}

fn get_env_variables<'a>() -> Result<&'static HashMap<String, String>, Error<'a>> {
    ENV_VARS.get_or_try_init(|| {
        {
            // Collect the variables in the file into a `HashMap`. Handle errors that might occur
            // because the file was not found, could not be read, contains a syntax error, etc.
            match dotenvy::dotenv_iter().map(|i| i.collect::<Result<HashMap<_, _>, _>>()) {
                Ok(env_vars) => env_vars,
                Err(err) if err.not_found() => return Err(Error::EnvFileNotFound),
                Err(err) => {
                    return Err(AnyhowError::new(err)
                        .context("error loading `.env` file")
                        .into())
                }
            }
        }
        // `?` not used here to avoid wrapping the block in an `Ok`.
        .map_err(Error::from)
    })
}

/// Get the value of an environment variable from the environment variable hashmap.
fn get_env_variable<'a>(var: impl Into<Cow<'a, str>>) -> Result<&'static str, Error<'a>> {
    let env_vars = get_env_variables()?;
    let var = var.into();

    Ok(env_vars
        .get(var.as_ref())
        .ok_or_else(|| Error::VarNotFound(var))?)
}

#[cfg(test)]
mod tests {
    use super::value_is_truthy;

    #[test]
    fn truthy_values() {
        assert!(["1", "true", "any", "foo", "yes", "CaP", "WEirD"]
            .into_iter()
            .all(value_is_truthy));
    }

    #[test]
    fn falsy_values() {
        assert!(!["", "0", "no", "false"].into_iter().any(value_is_truthy));
    }
}
