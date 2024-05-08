use std::time::Duration;

use testcontainers::{clients::Cli, core::WaitFor, Container, RunnableImage};
use tracing::info;

/// The database docker container.
#[derive(Debug, Clone)]
struct TimescaleDataBase {
    env_vars: Vec<(String, String)>,
}

impl TimescaleDataBase {
    fn new() -> Self {
        TimescaleDataBase {
            env_vars: vec![
                ("POSTGRES_DB".to_owned(), "postgres".to_owned()),
                ("POSTGRES_USER".to_owned(), "postgres".to_owned()),
                ("POSTGRES_PASSWORD".to_owned(), "postgres".to_owned()),
            ],
        }
    }
}

impl testcontainers::Image for TimescaleDataBase {
    type Args = ();

    fn name(&self) -> String {
        "timescale/timescaledb".to_owned()
    }

    fn tag(&self) -> String {
        "2.14.1-pg16".to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        )]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter().map(|s| (&s.0, &s.1)))
    }
}

/// The database docker migrator container.
#[derive(Debug, Clone)]
struct DataBaseMigrator {
    env_vars: Vec<(String, String)>,
    volumes: Vec<(String, String)>,
}

impl DataBaseMigrator {
    fn new(idx: u32) -> Self {
        DataBaseMigrator {
            env_vars: [
                (
                    "DATABASE_HOST".to_owned(),
                    format!("database-integration-{idx}"),
                ),
                ("DATABASE_PORT".to_owned(), "5432".to_owned()),
                ("DATABASE_USER".to_owned(), "postgres".to_owned()),
                ("DATABASE_PASSWORD".to_owned(), "postgres".to_owned()),
                (
                    "DATABASE_CONFIG".to_owned(),
                    "/docker/databases.toml".to_owned(),
                ),
            ]
            .into_iter()
            .collect(),
            volumes: [
                ("./docker/".to_owned(), "/docker".to_owned()),
                ("./migrations".to_owned(), "/migrations".to_owned()),
            ]
            .into_iter()
            .collect(),
        }
    }
}

impl testcontainers::Image for DataBaseMigrator {
    type Args = ();

    fn name(&self) -> String {
        "rust-bins".to_owned()
    }

    fn tag(&self) -> String {
        "latest".to_owned()
    }

    fn ready_conditions(&self) -> Vec<testcontainers::core::WaitFor> {
        vec![testcontainers::core::WaitFor::Duration {
            length: Duration::from_secs(3),
        }]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter().map(|s| (&s.0, &s.1)))
    }

    fn volumes(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.volumes.iter().map(|s| (&s.0, &s.1)))
    }

    fn entrypoint(&self) -> Option<String> {
        Some("/bin/database-migrator".to_owned())
    }
}

// Storing these to delay dropping as this would delete the containers
#[allow(dead_code)]
pub struct DataBase<'c> {
    data_base: Container<'c, TimescaleDataBase>,
    migrator: Container<'c, DataBaseMigrator>,
}

impl<'c> DataBase<'c> {
    /// Start the database and the migrator docker container into the cli. Returning a handle that
    /// will delete the containers when it is dropped.
    pub fn run(cli: &'c Cli, network: &str, idx: u32) -> DataBase<'c> {
        info!("Starting database");
        let data_base: RunnableImage<_> = TimescaleDataBase::new().into();
        let data_base = data_base
            .with_network(network)
            .with_container_name(format!("database-integration-{idx}"));
        let data_base = cli.run(data_base);

        info!("Runnig database migrations");
        let migrator: RunnableImage<_> = DataBaseMigrator::new(idx).into();
        let migrator = migrator.with_network(network);
        let migrator = cli.run(migrator);

        DataBase {
            data_base,
            migrator,
        }
    }
}
