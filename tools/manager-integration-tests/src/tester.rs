use std::collections::{HashMap, HashSet};

use crate::{
    config::{self, TestConfig},
    database::DataBase,
    manager_coms::{self, run_mock_simulator},
};

use anyhow::{bail, Context};
use proto::simulation::State;
use testcontainers::{clients::Cli, RunnableImage};
use tracing::{trace, warn};

const NETWORK_NAME: &str = "integration-test-network-main";

/// A simulation manger docker container
#[derive(Debug, Clone)]
struct SimulationManager {
    env_vars: Vec<(String, String)>,
}

impl SimulationManager {
    fn new(idx: u32) -> Self {
        SimulationManager {
            env_vars: [
                (
                    "SIMULATOR_MANAGER_ADDR".to_owned(),
                    "0.0.0.0:8100".to_owned(),
                ),
                (
                    "SIMULATOR_CONNECTOR_ADDR".to_owned(),
                    "0.0.0.0:8099".to_owned(),
                ),
                ("ENV_IMPURE".to_owned(), "true".to_owned()),
                ("RUST_LOG".to_owned(), "debug".to_owned()),
                ("RUST_BACKTRACE".to_owned(), "full".to_owned()),
                (
                    "DATABASE_HOST".to_owned(),
                    format!("database-integration-{idx}"),
                ),
                ("DATABASE_PORT".to_owned(), "5432".to_owned()),
                ("DATABASE_USER".to_owned(), "postgres".to_owned()),
                ("DATABASE_PASSWORD".to_owned(), "postgres".to_owned()),
                ("DATABASE_CONFIG".to_owned(), "/databases.toml".to_owned()),
            ]
            .into_iter()
            .collect(),
        }
    }
}

impl testcontainers::Image for SimulationManager {
    type Args = ();

    fn name(&self) -> String {
        "rust-bins".to_owned()
    }

    fn tag(&self) -> String {
        "latest".to_owned()
    }

    fn ready_conditions(&self) -> Vec<testcontainers::core::WaitFor> {
        vec![testcontainers::core::WaitFor::Nothing]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter().map(|s| (&s.0, &s.1)))
    }

    fn entrypoint(&self) -> Option<String> {
        Some("/bin/simulation-manager".to_owned())
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![8099, 8100]
    }
}

// A simulator docker container
#[derive(Debug, Clone)]
struct Simulator {
    executable_name: String,
    env_vars: Vec<(String, String)>,
}

impl Simulator {
    fn new(executable_name: &str, idx: u32) -> Self {
        Self {
            executable_name: executable_name.to_owned(),
            env_vars: [
                (
                    "SIMULATOR_CONNECTOR_ADDR".to_owned(),
                    format!("http://simulation-manager-{idx}:8099"),
                ),
                ("RUST_LOG".to_owned(), "trace".to_owned()),
                ("RUST_BACKTRACE".to_owned(), "full".to_owned()),
            ]
            .into_iter()
            .collect(),
        }
    }
}

impl testcontainers::Image for Simulator {
    type Args = ();

    fn name(&self) -> String {
        "rust-bins".to_owned()
    }

    fn tag(&self) -> String {
        "latest".to_owned()
    }

    fn ready_conditions(&self) -> Vec<testcontainers::core::WaitFor> {
        vec![testcontainers::core::WaitFor::Nothing]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter().map(|s| (&s.0, &s.1)))
    }

    fn entrypoint(&self) -> Option<String> {
        Some(format!("/bin/{}", self.executable_name))
    }
}

/// Helper to send a `()` on a oneshot channel once it is dropped.
struct SendOnDrop {
    tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl SendOnDrop {
    fn new(tx: tokio::sync::oneshot::Sender<()>) -> Self {
        Self { tx: Some(tx) }
    }

    /// Early send signal the signal, if this returns an error the other side of the channel was
    // already dropped.
    fn send(mut self) -> Result<(), ()> {
        self.tx.take().unwrap().send(())
    }
}

impl Drop for SendOnDrop {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            if tx.send(()).is_err() {
                warn!("failed to send stop signal to mock simulator");
            }
        }
    }
}

/// Helper to compare two hash maps
fn cmp_components_hash_maps(
    expected_components: &HashMap<String, config::ComponentInfo>,
    ignored_components: &HashSet<String>,
    components: &HashMap<String, proto::simulation::ComponentSpecification>,
) -> anyhow::Result<()> {
    if expected_components.len() + ignored_components.len() != components.len() {
        let mut expected = expected_components
            .keys()
            .chain(ignored_components.iter())
            .cloned()
            .collect::<Vec<_>>();
        expected.sort_unstable();
        let mut got = components.keys().cloned().collect::<Vec<_>>();
        got.sort_unstable();

        bail!(
            "Not the same amount of components {} vs {}\nExpected: {}\n     Got: {}",
            expected_components.len() + ignored_components.len(),
            components.len(),
            expected.join(", "),
            got.join(", "),
        );
    }
    for name in ignored_components {
        if !components.contains_key(name) {
            bail!("Expected component: {name}");
        }
    }

    for (name, comp) in expected_components {
        let Some(other) = components.get(name) else {
            bail!("Expected component: {name}")
        };
        if !comp.eq_proto(other) {
            bail!("Component {name} does not have to expected structure.\nExpected:\n{:?}\nGot:\n{:?}",
            comp,
            other);
        }
    }
    Ok(())
}

/// Run a single test, returning the frames gotten from the manager.
pub(crate) async fn run_test(config: &TestConfig, idx: u32) -> anyhow::Result<Vec<State>> {
    let network_name = format!("{NETWORK_NAME}-{idx}");

    let docker_runner = Cli::new::<testcontainers::core::env::Os>();

    let data_base = DataBase::run(&docker_runner, &network_name, idx);

    let simulator_manager: RunnableImage<_> = SimulationManager::new(idx).into();
    let simulator_manager = simulator_manager
        .with_network(&network_name)
        .with_container_name(format!("simulation-manager-{idx}"));
    let simulator_manager = docker_runner.run(simulator_manager);

    let simulators = config
        .used_simulators
        .iter()
        .map(|name| {
            let simulator: RunnableImage<_> = Simulator::new(name, idx).into();
            let simulator = simulator.with_network(&network_name);
            docker_runner.run(simulator)
        })
        .collect::<Vec<_>>();

    // HACK: to get simulator name from bin name
    let mut simulator_selection: Vec<_> = config
        .used_simulators
        .iter()
        .map(|n| n.replace('-', " "))
        .collect();

    let port = simulator_manager.get_host_port_ipv4(8100);
    let connection_port = simulator_manager.get_host_port_ipv4(8099);

    let mock = match &config.mock_simulator {
        Some(mock_simulator) => {
            let (tx, rx) = tokio::sync::oneshot::channel();

            let simulator = tokio::spawn(run_mock_simulator(
                mock_simulator.clone(),
                connection_port,
                async move { rx.await.unwrap_or_default() },
            ));

            simulator_selection.push("mock simulator".to_string());

            Some((SendOnDrop::new(tx), simulator))
        }
        None => None,
    };

    let mut client = manager_coms::Client::connect(port).await?;

    let components = &client.get_components().await?.components;
    trace!("Got components from manger: {components:?}");
    cmp_components_hash_maps(
        &config.expected_components,
        &config.ignored_components,
        components,
    )
    .context("testing if expected components match")?;

    let frames = client
        .run_simulation(
            config.initial_state.clone().into_proto_state(),
            config.amount_of_timesteps,
            config.timestep_delta,
            simulator_selection,
        )
        .await?;
    trace!("Got frames from manger: {frames:?}");

    drop((data_base, simulator_manager, simulators));
    if let Some((mock_stop_channel, simulator)) = mock {
        if mock_stop_channel.send().is_err() {
            warn!("failed to send stop signal to mock simulator");
        }
        simulator.await?.context("mock simulator")?;
    }

    Ok(frames)
}

/// Run a single test, comparing the gotten frames to the expected output from the `config`.
pub(crate) async fn run_test_and_cmp(config: TestConfig, idx: u32) -> anyhow::Result<()> {
    let frames = run_test(&config, idx).await?;

    for (i, (expected, got)) in config
        .expected_output_frames
        .into_iter()
        .zip(frames)
        .enumerate()
    {
        expected
            .eq_proto_state(got)
            .with_context(|| format!("in frame {i}"))?;
    }
    Ok(())
}
