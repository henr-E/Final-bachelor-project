fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile the sensor proto files.
    tonic_build::configure()
        // Compile only the relevant proto files. The second argument is a list of include
        // directories. These are directories that will be searched when resolving import
        // statements.
        .compile(
            &[
                "../../proto/sensor/data-ingest.proto",
                "../../proto/sensor/sensor-crud.proto",
                "../../proto/sensor/data-fetching.proto",
                "../../proto/sensor/bigdecimal.proto",
            ],
            // Only allow imports from the `sensor` subproject.
            &["../../proto/sensor"],
        )?;

    tonic_build::configure().compile(
        &[
            "../../proto/simulation/simulation-manager.proto",
            "../../proto/simulation/simulator.proto",
            "../../proto/simulation/simulator-connection.proto",
        ],
        &["../../proto/simulation"],
    )?;

    tonic_build::configure().compile(&["../../proto/twins/twin.proto"], &["../../proto/twins"])?;

    tonic_build::configure().compile(
        &["../../proto/authentication/auth.proto"],
        &["../../proto/authentication"],
    )?;

    tonic_build::configure().compile(
        &["../../proto/simulation/frontend.proto"],
        &["../../proto/simulation"],
    )?;

    // Compile another subproject proto files.
    // NOTE: This separation is done to prevent cross subproject imports in proto files.
    // INSERT HERE

    Ok(())
}
