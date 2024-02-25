//! Library to easily crate new simulators in rust.
//!
//! To use this library you will have to implement the [`Simulator`] trait and use it to create a [`Server`].
//!
//! See the examples directory for an example on how to use this library.
#![warn(missing_docs)]
#![deny(clippy::unwrap_used)]

pub mod component;
pub mod graph;
pub mod simulator;

use std::{net::SocketAddr, time::Duration};

use tokio::sync::Mutex;
use tonic::{transport, Request, Response, Status};

use proto::{
    simulator_server::SimulatorServer, InitialState, IoConfigRequest, SetupResponse,
    SimulatorIoConfig, State, TimestepResult,
};

pub use graph::Graph;
pub use simulator::{ComponentsInfo, Simulator};

/// Derive [`Component`] on a struct
///
/// This macro can only be used on structs.
///
/// Use the `component` attribute to specify the component name and type.
/// The posible types are: `"node"`, `"edge"` and `"global"`.
/// # Example:
/// ```
/// # use simulator_communication::{Component, ComponentPiece};
///
/// #[derive(ComponentPiece, Component)]
/// #[component(name = "transmision-line", ty = "edge")]
/// struct TransmisionLine {
///     above_ground: bool,
///     length: f64,
/// }
/// ```
pub use simulator_communication_macros::Component;
/// Derive [`ComponentPiece`] on a struct
///
/// This macro can only be used on structs.
///
/// # Example:
/// ```
/// # use simulator_communication::{Component, ComponentPiece};
///
/// #[derive(ComponentPiece)]
/// struct ExampleSingleValueComp(f64);
///
/// #[derive(ComponentPiece)]
/// struct SharedElecData {
///     max_volt: f64,
///     current_volt: f64,
/// }
///
/// #[derive(ComponentPiece, Component)]
/// #[component(name = "transmision-line", ty = "edge")]
/// struct TransmisionLine {
///     shared_elec_data: SharedElecData,
///     length: f64,
/// }
/// ```
pub use simulator_communication_macros::ComponentPiece;

// Needed to make macro derives work in tests
extern crate self as simulator_communication;
#[doc(hidden)]
pub use prost_types::{self, Value};
#[doc(hidden)]
pub use proto::simulator as proto;
#[doc(hidden)]
pub use proto::{component_structure, ComponentSpecification, ComponentStructure};

#[tonic::async_trait]
impl<S: Simulator> proto::simulator_server::Simulator for Server<S> {
    /// Returns the input/output configuration of the simulation, including required and optional input kinds and the kind of output produced.
    async fn get_io_config(
        &self,
        _request: Request<IoConfigRequest>,
    ) -> Result<Response<SimulatorIoConfig>, Status> {
        Ok(Response::new(self.io_config.clone()))
    }

    /// Accepts an initial simulation state and initializes the simulation with it for a given delta_time.
    async fn setup(
        &self,
        request: Request<InitialState>,
    ) -> Result<Response<SetupResponse>, Status> {
        let initial_state = request.into_inner();

        let delta_time = Duration::from_millis(initial_state.timestep_delta);
        let initial_state = initial_state
            .initial_state
            .ok_or_else(|| Status::invalid_argument("sould provide initial state"))?;

        let graph = Graph::from_state(initial_state, &self.components_info)
            .ok_or_else(|| Status::invalid_argument("Could not create graph"))?;

        *self.simulator.lock().await = Some(S::new(delta_time, graph));

        Ok(Response::new(SetupResponse {}))
    }

    /// Takes a simulation state as input, executes a simulation step, and returns the resulting state.
    async fn do_timestep(
        &self,
        request: Request<State>,
    ) -> Result<Response<TimestepResult>, Status> {
        let state = request.into_inner();

        let graph = Graph::from_state(state, &self.components_info)
            .ok_or_else(|| Status::invalid_argument("Could not create graph"))?;

        let result_graph = self
            .simulator
            .lock()
            .await
            .as_mut()
            .ok_or_else(|| {
                Status::failed_precondition("should `setup` before calling `do_timestep`")
            })?
            .do_timestep(graph);

        Ok(Response::new(TimestepResult {
            output_state: Some(
                result_graph
                    .into_state(&self.components_info)
                    .ok_or_else(|| Status::internal("could not create state from graph"))?,
            ),
        }))
    }
}

/// Server struct that holds the state and configuration of the simulation.
///
/// This is the main struct you will want to use the start a new simulation server.
///
/// # Example:

/// ```
/// # use std::{net::SocketAddr, process::ExitCode, time::Duration};
/// # use simulator_communication::{Server, Graph, ComponentsInfo, Simulator};
/// #
/// # struct ExampleSimulator {}
/// # impl Simulator for ExampleSimulator {
/// # fn get_component_info() -> ComponentsInfo { todo!() }
/// # fn new(delta_time: Duration, graph: Graph) -> Self { todo!() }
/// # fn do_timestep(&mut self, graph: Graph) -> Graph { todo!() }
/// # }
/// # async fn a() -> ExitCode {
/// # let listen_addr: SocketAddr = todo!();
/// // Create a simulator server using a simulator.
/// let server = Server::<ExampleSimulator>::new();
///
/// // Start the server using `listen_on`. This may return an error if
/// // something goes wrong during the execution of the program,
/// // so we need to handle this error appropriately. Here we print the error and exit.
/// if let Err(err) = server.listen_on(listen_addr).await {
///     eprintln!("Server return an error: {err}");
///     return ExitCode::FAILURE;
/// }
/// println!("Server exited successfully");
/// ExitCode::SUCCESS
/// # }
pub struct Server<S: Simulator> {
    io_config: SimulatorIoConfig,
    components_info: ComponentsInfo,
    simulator: Mutex<Option<S>>,
}

impl<S: Simulator> Server<S> {
    /// Creates a new server instance.
    pub fn new() -> Self {
        let components_info = S::get_component_info();

        let output_components = components_info
            .output_components
            .values()
            .map(|i| i.name.clone())
            .collect();

        let required_input_components = components_info
            .components
            .values()
            .filter_map(|i| {
                if i.required {
                    Some(i.name.clone())
                } else {
                    None
                }
            })
            .collect();

        let optional_input_components = components_info
            .components
            .values()
            .filter_map(|i| {
                if !i.required {
                    Some(i.name.clone())
                } else {
                    None
                }
            })
            .collect();

        let components = components_info
            .components
            .values()
            .chain(components_info.output_components.values())
            .map(|i| (i.name.clone(), i.proto_spec.clone()))
            .collect();

        Self {
            io_config: SimulatorIoConfig {
                output_components,
                required_input_components,
                optional_input_components,
                components,
            },
            components_info,
            simulator: Mutex::new(None),
        }
    }

    /// Start a new server listening on the given `addr`. Does not return unless there is some error.
    pub async fn listen_on(
        self,
        addr: impl Into<SocketAddr>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let addr = addr.into();

        transport::Server::builder()
            .add_service(SimulatorServer::new(self))
            .serve(addr)
            .await?;

        Ok(())
    }
}

impl<S: Simulator> Default for Server<S> {
    fn default() -> Self {
        Self::new()
    }
}
