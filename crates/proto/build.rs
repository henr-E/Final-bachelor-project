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
        )
        // Convert error type into a polomorphic error stored on the heap.
        .map_err(Box::new)?;

    // Compile the simulation proto files.
    tonic_build::compile_protos("../../proto/simulator.proto")
        // Compile only the relevant proto files. The second argument is a list of include
        // directories. These are directories that will be searched when resolving import
        // statements.
        // .compile(&["../../proto/simulator.proto"], &[...])
        // Convert error type into a polomorphic error stored on the heap.
        .map_err(Box::new)?;

    // Compile another subproject proto files.
    // NOTE: This separation is done to prevent cross subproject imports in proto files.
    // INSERT HERE

    Ok(())
}
