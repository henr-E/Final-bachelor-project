use std::{net::SocketAddr, sync::Arc};

use proto::simulation::{
    simulator::SimulatorClient,
    simulator_connection::{SimulatorConnection, SimulatorInfo},
};
use tokio::sync::Mutex;
use tonic::{transport::Channel, Request, Response, Status};

/// The connector allows the Simulation Manager to keep track of available simulators.
/// This is done by modifying the Vec which is shared by the manager and the runner.
///
/// Right now, simulators can only be added, but they can not be removed.
///
/// To add a simulator:
/// ```
/// use proto::simulation::simulator_connection::{
///     SimulatorConnectionClient,
///     SimulatorPort
/// };
///
/// let connector = SimulatorConnectionClient::connect("127.0.0.1:8099").await.unwrap();
/// connector.connect_simulator(SimulatorPort { port: 8101 }).await;
/// ```
pub struct SimulatorsInfo {
    pub(crate) simulator: SimulatorClient<Channel>,
    pub(crate) name: String,
}

pub struct SimulatorConnector {
    simulators: Arc<Mutex<Vec<SimulatorsInfo>>>,
}

impl SimulatorConnector {
    pub fn new(simulators_info: Arc<Mutex<Vec<SimulatorsInfo>>>) -> Self {
        Self {
            simulators: simulators_info,
        }
    }
}

#[tonic::async_trait]
impl SimulatorConnection for SimulatorConnector {
    async fn connect_simulator(
        &self,
        request: Request<SimulatorInfo>,
    ) -> Result<Response<()>, Status> {
        // get address from request
        let mut address: SocketAddr = match request.remote_addr() {
            Some(addr) => addr,
            None => {
                return Err(Status::internal(
                    "Could not get socket address from request.",
                ))
            }
        };
        let simulator: SimulatorInfo = request.into_inner();
        let port: u16 = simulator.port as u16;
        address.set_port(port);
        // connect to simulator
        let name = simulator.name;
        let client = SimulatorClient::connect(format!("http://{}", address))
            .await
            .map_err(|err| Status::from_error(Box::new(err)))?;
        let mut simulators = self.simulators.lock().await;
        simulators.push(SimulatorsInfo {
            simulator: client,
            name,
        });

        drop(simulators);
        Ok(Response::new(()))
    }
}
