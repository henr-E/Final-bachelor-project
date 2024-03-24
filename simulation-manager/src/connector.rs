use std::{net::SocketAddr, sync::Arc};

use proto::simulation::{
    simulator::SimulatorClient,
    simulator_connection::{SimulatorConnection, SimulatorPort},
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
pub struct SimulatorConnector {
    simulators: Arc<Mutex<Vec<SimulatorClient<Channel>>>>,
}

impl SimulatorConnector {
    pub fn new(simulators: Arc<Mutex<Vec<SimulatorClient<Channel>>>>) -> Self {
        Self { simulators }
    }
}

#[tonic::async_trait]
impl SimulatorConnection for SimulatorConnector {
    async fn connect_simulator(
        &self,
        request: Request<SimulatorPort>,
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
        let port: u16 = request.into_inner().port as u16;
        address.set_port(port);

        // connect to simulator
        let client = SimulatorClient::connect(format!("http://{}", address))
            .await
            .map_err(|err| Status::from_error(Box::new(err)))?;

        self.simulators.lock().await.push(client);
        Ok(Response::new(()))
    }
}
