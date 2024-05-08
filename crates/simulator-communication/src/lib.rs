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

use std::{error::Error, net::SocketAddr, time::Duration};

use tokio::sync::Mutex;
use tonic::{transport, Request, Response, Status};

use proto::{
    simulator::{
        simulator_server::SimulatorServer, InitialState, IoConfigRequest, SetupResponse,
        SimulatorIoConfig, TimestepResult,
    },
    simulator_connection::{SimulatorConnectionClient, SimulatorInfo},
    State,
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
pub use proto::simulation as proto;
#[doc(hidden)]
pub use proto::{component_structure, ComponentSpecification, ComponentStructure};
use tracing::info;

#[tonic::async_trait]
impl<S: Simulator> proto::simulator::simulator_server::Simulator for Server<S> {
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

        *self.simulator.lock().await = Some(S::new(delta_time, graph).await?);

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
            .do_timestep(graph)
            .await?;

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
/// # use simulator_communication::{Server, Graph, ComponentsInfo, Simulator, simulator::SimulationError};
/// #
/// # struct ExampleSimulator {}
/// # impl Simulator for ExampleSimulator {
/// # fn get_component_info() -> ComponentsInfo { todo!() }
/// # async fn new(delta_time: Duration, graph: Graph) -> Result<Self, SimulationError> { todo!() }
/// # async fn do_timestep(&mut self, graph: Graph) -> Result<Graph, SimulationError> { todo!() }
/// # }
/// # async fn a() -> ExitCode {
/// # let simulator_addr: SocketAddr = todo!();
/// # let connector_addr: String = todo!();
/// // Create a simulator server using a simulator.
/// let server = Server::<ExampleSimulator>::new();
///
/// // Start the server using `listen_on`. This may return an error if
/// // something goes wrong during the execution of the program,
/// // so we need to handle this error appropriately. Here we print the error and exit.
/// if let Err(err) = server.start(simulator_addr, connector_addr, "name").await {
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

/// Possible errors the server could return.
pub enum ServerError {
    /// A transport error from main server.
    Transport(tonic::transport::Error),
    /// Failed while trying to connect to the manger to advertise.
    ConnectionTransport(tonic::transport::Error),
    /// Failed while trying to advertise to the manger.
    ConnectionReturn(String),
    /// ANy other error, like a tokio error.
    Other(Box<dyn Error>),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::Transport(e) => write!(f, "Transport error: {e}"),
            ServerError::ConnectionTransport(e) => write!(f, "Conneciton transport error: {e}"),
            ServerError::ConnectionReturn(e) => write!(f, "Connection returned an error: {e}"),
            ServerError::Other(e) => write!(f, "Other error: {e}"),
        }
    }
}

impl<E: Error + 'static> From<E> for ServerError {
    fn from(value: E) -> Self {
        Self::Other(Box::new(value))
    }
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

    /// Start a new server listening on the given `simulator_addr` and attaches itself to the manager with given `manager_addr`.
    /// `manager_addr` needs to be a valid endpoint, meaning "http://" is included.
    /// This function does not return unless there is some error.
    pub async fn start(
        self,
        simulator_addr: impl Into<SocketAddr>,
        manager_addr: String,
        name: &str,
    ) -> Result<(), ServerError> {
        let addr = simulator_addr.into();
        let port = addr.port() as u32;

        let server = transport::Server::builder()
            .add_service(SimulatorServer::new(self))
            .serve(addr);

        // Keep simulator running in a different thread
        let mut task = tokio::spawn(server);

        let connection = async move {
            info!("Sending connection request to manager");
            SimulatorConnectionClient::connect(manager_addr).await
        };

        // Checks if the simulator returns an early error. If it doesn't, then the connection to the manager will finish first.
        tokio::select! {
            err = &mut task => {
                err?.map_err(ServerError::Transport)?
            },
            connection = connection => {
                connection
                    .map_err(ServerError::ConnectionTransport)?
                    .connect_simulator(SimulatorInfo { port, name: name.to_string()  }).await
                    .map_err(|e| ServerError::ConnectionReturn(e.message().to_owned()))?;

                // Keep simulator running
                task.await?.map_err(ServerError::Transport)?
            },
        }

        Ok(())
    }
}

impl<S: Simulator> Default for Server<S> {
    fn default() -> Self {
        Self::new()
    }
}
