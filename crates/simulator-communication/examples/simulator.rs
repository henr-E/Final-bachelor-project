use std::{env, net::SocketAddr, process::ExitCode};

use simulator_communication::{
    Component, ComponentPiece, ComponentsInfo, Graph, Server, Simulator,
};

/// Every simulator will have their own type that implements the [`Simulator`] trait.
///
/// All the actual logic of the simulator is driven by this type.
pub struct ExampleSimulator {}

impl Simulator for ExampleSimulator {
    fn get_component_info() -> ComponentsInfo {
        // In this function we should indicate which components that we want to receive and send back.
        // This should always return the same value.
        ComponentsInfo::new()
            // We want to both use `ExampleNodeComponent` as input and output.
            .add_required_component::<ExampleNodeComponent>()
            .add_output_component::<ExampleNodeComponent>()
            // Only ask for `ExampleEdgeComponent` as input.
            .add_required_component::<ExampleEdgeComponent>()
            // Ask for `ExampleGlobalComponent` as input only if it exists.
            .add_optional_component::<ExampleGlobalComponent>()
    }

    fn new(_delta_time: std::time::Duration, _graph: Graph) -> Self {
        Self {}
    }

    fn do_timestep(&mut self, graph: Graph) -> Graph {
        // A real simulator would apply some operations on the graph to modify it before sending
        // back the result.
        graph
    }
}

/// An example component that will be present in nodes.
#[derive(Debug, ComponentPiece, Component)]
#[component(name = "node_example", ty = "node")]
pub struct ExampleNodeComponent {
    pub some_int: i32,
    pub some_string: String,
    pub some_list: Vec<bool>,
}

/// This component is found in edges.
///
/// It is not represented as a struct but rather just as an [f64].
#[derive(Debug, ComponentPiece, Component)]
#[component(name = "edge_example", ty = "edge")]
pub struct ExampleEdgeComponent(pub f64);

/// Example of a global component.
#[derive(Debug, ComponentPiece, Component)]
#[component(name = "global_example", ty = "global")]
pub struct ExampleGlobalComponent {
    pub interesting_value: u32,
}

#[tokio::main]
async fn main() -> ExitCode {
    let listen_addr = env::var("SIMULATOR_EXAMPLE_ADDR")
        .unwrap_or("127.0.0.1:8101".to_string())
        .parse::<SocketAddr>()
        .expect("a valid listen address");

    // Manager address
    let connector_addr =
        env::var("SIMULATOR_CONNECTOR_ADDR").unwrap_or("http://127.0.0.1:8099".to_string());

    // Create a simulator server using our simulator defined above. Will use the `new` function in the simulator.
    let server = Server::<ExampleSimulator>::new();
    // Start the server using `start`. This may return an error if something goes wrong during the execution of the program,
    // so we need to handle this error appropriately. Here we print the error and exit.
    println!("Starting example simulator server on {listen_addr}");
    if let Err(err) = server.start(listen_addr, connector_addr).await {
        eprintln!("Server return an error: {err}");
        return ExitCode::FAILURE;
    }
    println!("Server exited successfully");
    ExitCode::SUCCESS
}
