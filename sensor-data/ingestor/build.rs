use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_variables =
        dotenvy::dotenv_iter().map(|i| i.collect::<Result<HashMap<_, _>, _>>())??;
    let database = env_variables
        .get("SENSOR_ARCHIVE_DB_NAME")
        .expect("SENSOR_ARCHIVE_DB_NAME environment variable not set");
    let password = env_variables
        .get("SENSOR_DB_PASSWORD")
        .expect("SENSOR_DB_PASSWORD environment variable not set");
    let database_url = format!(
        "postgres://{}:{}@localhost:5432/{}",
        database, password, database
    );

    // Set the sqlx database url env variable.
    // NOTE: This will also be set when compiling for production. Thus, when compiling for
    // production we should use the `SQLX_OFFLINE` flag to force the use of generated database
    // definitions instead of a live one.
    // NOTE: This is only set when building. When running the application from the binary in
    // production, this variable will not be set.

    println!("cargo:rustc-env=DATABASE_URL={}", database_url);

    // Rebuild (partial) the project when the migrations directory is updated.
    println!("cargo:rerun-if-changed=./migrations");

    Ok(())
}
