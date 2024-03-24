use database_config::{configure_recompile, do_migrations_if_enabled, set_database_url};

const MIGRTIONS_DIR_PATH: &str = "../../migrations/sensor";
const ENV_PATH: &str = "../../.env";
const MIGRATIONS_CONFIG_PATH: &str = "../../docker/databases.toml";

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result {
    async fn config() -> Result {
        do_migrations_if_enabled(MIGRTIONS_DIR_PATH, "sensor_archive").await?;

        // Set the sqlx database url env variable.
        // NOTE: This will also be set when compiling for production. Thus, when compiling for
        // production we should use the `SQLX_OFFLINE` flag to force the use of generated database
        // definitions instead of a live one.
        // NOTE: This is only set when building. When running the application from the binary in
        // production, this variable will not be set.
        set_database_url("sensor_archive")?;

        // Configure cargo to recompile the crate when the following directories/files contain changes.
        configure_recompile(MIGRTIONS_DIR_PATH, ENV_PATH, MIGRATIONS_CONFIG_PATH);

        Ok(())
    }

    let result = config().await;
    if let Err(err) = &result {
        // Display print errors for better error messages.
        eprintln!("{}", err);
    }
    result
}
