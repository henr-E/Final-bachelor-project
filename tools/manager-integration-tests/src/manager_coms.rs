use anyhow::{bail, Context};
use futures::{stream, Future};
use proto::simulation::{
    simulation_manager::{
        ComponentsInfo, PushSimulationRequest, SimulationFrameRequest, SimulationId,
        SimulationManagerClient, SimulationStatus, SimulatorSelection,
    },
    simulator::{
        simulator_server, InitialState, IoConfigRequest, SetupResponse, SimulatorIoConfig,
        SimulatorServer, TimestepResult,
    },
    simulator_connection::{SimulatorConnectionClient, SimulatorInfo},
    State,
};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, SystemTime},
};
use tokio_stream::StreamExt as _;
use tonic::{
    transport::{self, Channel},
    Request, Response, Status,
};
use tracing::{debug, trace};
use uuid::Uuid;

use crate::config::MockSimulatorConfig;

pub(crate) struct Client {
    connection: SimulationManagerClient<Channel>,
}

impl Client {
    /// Tries to connect to a simulator manager exposed on localhost on the given port.
    pub(crate) async fn connect(port: u16) -> anyhow::Result<Self> {
        let connection = SimulationManagerClient::connect(format!("http://127.0.0.1:{port}"))
            .await
            .context("while conecting to manger")?;

        Ok(Self { connection })
    }

    /// Asks the manger for all the components
    pub(crate) async fn get_components(&mut self) -> anyhow::Result<ComponentsInfo> {
        let result = self
            .connection
            .get_components(())
            .await
            .context("manger error while getting componnets")?;
        Ok(result.into_inner())
    }

    /// Pushes a simulation to the manger, waits for the simulation to finish.
    /// And then ask for all the frames and returning them.
    pub(crate) async fn run_simulation(
        &mut self,
        initial_state: State,
        timesteps: u32,
        timestep_delta: Duration,
        simulator_selection: Vec<String>,
    ) -> anyhow::Result<Vec<State>> {
        let id = SimulationId {
            uuid: Uuid::new_v4().to_string(),
        };

        trace!("Pushing simulation with initial state: {:?}", initial_state);
        self.connection
            .push_simulation(PushSimulationRequest {
                id: Some(id.clone()),
                initial_state: Some(initial_state),
                timesteps: timesteps.into(),
                timestep_delta: timestep_delta.as_secs_f64(),
                selection: Some(SimulatorSelection {
                    name: simulator_selection,
                }),
            })
            .await
            .context("manager error while pushing simulation")?;

        let mut simulation_data;

        let start = SystemTime::now();
        loop {
            simulation_data = self
                .connection
                .get_simulation(id.clone())
                .await
                .context("manager error getting simulation details")?
                .into_inner();

            match simulation_data.status() {
                SimulationStatus::Finished => {
                    if simulation_data.timestep_count == simulation_data.max_timestep_count {
                        break;
                    } else {
                        // manager is still saving the frames
                    }
                }
                SimulationStatus::Failed => {
                    if let Some(info) = simulation_data.status_info {
                        bail!("simulation had status \"failed\" with message: {info}");
                    } else {
                        bail!("simulation had status \"failed\" without a message");
                    }
                }
                _ => {}
            }

            tokio::time::sleep(Duration::from_millis(50)).await;

            if start.elapsed()?.as_secs() > 10 {
                if simulation_data.timestep_count == simulation_data.max_timestep_count {
                    bail!("simulation took longer than 10s!\nDit {} of {} frames, but status was: {:?}",
                    simulation_data.timestep_count,
                    simulation_data.max_timestep_count,
                    simulation_data.status(),);
                } else {
                    bail!(
                        "simulation took longer than 10s!\nOnly dit {} of {} frames in 10s",
                        simulation_data.timestep_count,
                        simulation_data.max_timestep_count,
                    );
                }
            }
        }

        if simulation_data.timestep_count != simulation_data.max_timestep_count {
            bail!("simulation `timestep_count` did not equal `max_timestep_count` while manger claimed the simulation was done");
        }
        let steps = simulation_data.timestep_count;
        if steps != timesteps as u64 {
            bail!("manager did {steps} instead of the requested {timesteps}");
        }

        let frames = self
            .connection
            .get_simulation_frames(stream::iter((0..steps).map(move |i| {
                SimulationFrameRequest {
                    simulation_id: Some(id.clone()),
                    frame_nr: i as u32,
                }
            })))
            .await?
            .into_inner()
            .map(|r| {
                r.context("manager returned error")
                    .and_then(|f| f.state.context("getting state from frame"))
            })
            .collect::<Result<Vec<_>, _>>()
            .await
            .context("reading simulaiton frames")?;

        Ok(frames)
    }
}

/// Starts a mock simulator returning frames according the given config. It will try to connect
/// to a manger running on the `connection_port`. The stop_signal should be a future that resolves
/// when the mock simulator should be turned off.
pub(crate) async fn run_mock_simulator<S>(
    config: MockSimulatorConfig,
    connection_port: u16,
    stop_signal: S,
) -> anyhow::Result<()>
where
    S: Future<Output = ()> + Send + 'static,
{
    let server = SimulatorServer::new(MockSimulator {
        step: AtomicUsize::new(0),
        config,
    });

    let server = transport::Server::builder()
        .add_service(server)
        .serve_with_shutdown(
            SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 8843),
            stop_signal,
        );
    let mut server = std::pin::pin!(server);

    let connection = async move {
        debug!("sending connection request to manger");
        SimulatorConnectionClient::connect(format!("http://127.0.0.1:{connection_port}")).await
    };

    tokio::select! {
        err = &mut server => {
            err.context("mock server errored")?
        },
        connection = connection => {
            connection
                .context("mockserver could not connect with manger")?
                .connect_simulator(SimulatorInfo { port: 8843, name: "mock simulator".to_owned() }).await
                .context("advertising to manager")?;

            // Keep simulator running
            server.await.context("mock server errored")?
        },
    }

    Ok(())
}

struct MockSimulator {
    step: AtomicUsize,
    config: MockSimulatorConfig,
}

#[tonic::async_trait]
impl simulator_server::Simulator for MockSimulator {
    async fn get_io_config(
        &self,
        _request: Request<IoConfigRequest>,
    ) -> Result<Response<SimulatorIoConfig>, Status> {
        Ok(Response::new(SimulatorIoConfig {
            output_components: self.config.output_components.keys().cloned().collect(),
            required_input_components: Vec::new(),
            optional_input_components: Vec::new(),
            components: self
                .config
                .output_components
                .iter()
                .map(|(name, comp)| (name.clone(), comp.to_proto()))
                .collect(),
        }))
    }

    /// Accepts an initial simulation state and initializes the simulation with it for a given delta_time.
    async fn setup(
        &self,
        _request: Request<InitialState>,
    ) -> Result<Response<SetupResponse>, Status> {
        Ok(Response::new(SetupResponse {}))
    }

    /// Takes a simulation state as input, executes a simulation step, and returns the resulting state.
    async fn do_timestep(
        &self,
        _request: Request<State>,
    ) -> Result<Response<TimestepResult>, Status> {
        let frame = self
            .config
            .data
            .get(self.step.load(Ordering::SeqCst))
            .ok_or(Status::internal(format!(
                "Mock simulator got asked for more timeframes than there was date for. Asked for frame {} but got {} frames",
                self.step.load(Ordering::SeqCst),
                self.config.data.len()
            )))?;

        // This simulator should only be used for single simulation, so it should be fine to just
        // use a "global" value.
        self.step.fetch_add(1, Ordering::SeqCst);

        debug!("Sending frame {:?} to manager", self.step);
        trace!("Frame {:?} data {:?}", self.step, &frame);

        let state = frame.clone().into_proto_state();

        Ok(Response::new(TimestepResult {
            output_state: Some(state),
        }))
    }
}
