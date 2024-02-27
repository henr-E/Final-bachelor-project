use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only set the environment variable if an dotenv file was found
    match dotenvy::dotenv_iter().map(|i| i.collect::<Result<HashMap<_, _>, _>>()) {
        Ok(env_variables) => {
            // Return the inner error.
            let env_variables = env_variables?;
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
        }
        Err(err) if err.not_found() => {
            eprintln!("[sensor-data-ingestor] no .env file found!");
            return Err(err.into());
        }
        Err(err) => return Err(err.into()),
    }

    // Rebuild (partial) the project when the migrations directory is updated.
    println!("cargo:rerun-if-changed=./migrations");

    Ok(())
}
