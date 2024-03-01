fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile the sensor proto files.
    tonic_build::configure()
        // Compile only the relevant proto files. The second argument is a list of include
        // directories. These are directories that will be searched when resolving import
        // statements.
        .compile(
            &["../../proto/sensor/data-ingest.proto"],
            // Only allow imports from the `senor` subproject.
            &["../../proto/sensor"],
        )?;

    tonic_build::configure().compile(
        &[
            "../../proto/simulation/simulation-manager.proto",
            "../../proto/simulation/simulator.proto",
        ],
        &["../../proto/simulation"],
    )?;

    // Compile another subproject proto files.
    // NOTE: This separation is done to prevent cross subproject imports in proto files.
    // INSERT HERE

    Ok(())
}
