use std::{collections::BTreeMap, env, path::PathBuf};

use anyhow::Context;
use serde::Deserialize;
use sqlx::{PgPool, Row};
use tokio::fs;
use tracing::info;

/// Contains configuration options for the databases in the system.
#[derive(Deserialize, Clone, Debug)]
struct Config {
    /// A mapping from database name to the options for this database.
    ///
    /// Stored in a [BTreeMap] to guarantee deterministic ordering of databases.
    databases: BTreeMap<String, DatabaseSettings>,
}

/// Configuration options per database.
#[derive(Deserialize, Clone, Debug)]
struct DatabaseSettings {
    /// The folder where the migrations for this particular database can be found.
    migrations: Option<PathBuf>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt().init();

    let db_config_path = env::var("DATABASE_CONFIG")
        .context("expected environment variable `DATABASE_CONFIG` to be present")?;

    let db_config_file = fs::read_to_string(db_config_path)
        .await
        .context("while trying to read databse config file")?;
    let db_config: Config =
        toml::from_str(&db_config_file).context("while trying to decode database config file")?;

    let root_connection = PgPool::connect(&make_connection_string("postgres"))
        .await
        .context("while trying to connect to root database")?;

    // Ensure each database is created and run the migrations for each database.
    for (db_name, db_options) in &db_config.databases {
        // First check if the database exists. Postgres does not have a `create database if not
        // exists` command.
        // Intuitively this would be done with a transaction but you cannot create databases in
        // postgres transactions.
        info!(?db_name, "Check if database exists.");
        let db_exists: bool =
            sqlx::query("SELECT EXISTS(SELECT FROM pg_database WHERE datname = $1) AS exists")
                .bind(db_name)
                .fetch_one(&root_connection)
                .await
                .context("while trying to check if database exists")?
                .get("exists");

        if !db_exists {
            info!(?db_name, "Creating database.");
            // For some reason we need to manually interpolate the string in postgres.
            sqlx::query(&format!("CREATE DATABASE {db_name}"))
                .execute(&root_connection)
                .await
                .context("could not create database")?;
        } else {
            info!(?db_name, "Database exists.");
        }

        info!(?db_name, "Connecting to newly made database.");
        let db_connection = PgPool::connect(&make_connection_string(db_name))
            .await
            .context("while trying to connect to root database")?;

        // Run migrations, if specified.
        if let Some(migrations) = db_options.migrations.as_ref() {
            info!(?db_name, "Running migrations on database.");
            sqlx::migrate::Migrator::new(migrations.as_path())
                .await
                .context("while creating migrator")?
                .run(&db_connection)
                .await
                .context("while running migrations")?;
        } else {
            info!(?db_name, "No migrations folder specified.");
        }
    }
    Ok(())
}

/// Helper function to create database connection strings from the system environment.
fn make_connection_string(db_name: &str) -> String {
    format!(
        "postgres://{}:{}@{}:{}/{db_name}",
        env::var("DATABASE_USER").expect("environment variable `DATABASE_USER` to be present"),
        secrets::secret("DATABASE_PASSWORD").expect("secret `DATABASE_PASSWORD` to be present"),
        env::var("DATABASE_HOST").expect("environment variable `DATABASE_HOST` to be present"),
        env::var("DATABASE_PORT").expect("environment variable `DATABASE_PORT` to be present"),
    )
}
