mod config;
mod database;
mod manager_coms;
mod tester;

use std::{
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};

use clap::{Parser, Subcommand, ValueEnum};
use tracing::{debug, info, trace};

use crate::config::TestConfigFile;

/// A test runner from simulator to manager.
#[derive(Debug, Parser)]
#[command(name = "manager-integration-tests")]
#[command(about = "A test runner from simulator to manager.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

impl Cli {
    /// Some command need to be allowed to run docker containers
    fn need_root(&self) -> bool {
        match self.command {
            Some(Commands::Generate(Generate {
                target: GeneratorTarget::OutputHeaders,
                ..
            })) => false,
            Some(Commands::Generate(Generate {
                target: GeneratorTarget::Output,
                ..
            })) => true,
            None => true,
        }
    }
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum GeneratorTarget {
    /// Generate an output file with the current output of the simulation.
    /// Only writing to the columns where there already existed headers for.
    Output,
    /// Generate an output file with a header for all the fields in the simulation
    OutputHeaders,
}

#[derive(Debug, Parser)]
struct Generate {
    /// What part of the test to generate.
    #[arg()]
    target: GeneratorTarget,
    /// Path to the test to generate the csv file for
    #[arg(required = true)]
    test_file: PathBuf,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate auxiliary files from partial test file.
    Generate(Generate),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    if cli.need_root() && sudo::with_env(&["RUST_LOG"]).is_err() {
        bail!("Need root to run this command");
    }

    match cli.command {
        None => run_tests().await,
        Some(Commands::Generate(g)) => generate(&g).await,
    }
}

/// Run the file generation functions
async fn generate(generate_config: &Generate) -> anyhow::Result<()> {
    let test_path = &generate_config.test_file;

    let file = std::fs::read_to_string(test_path)
        .with_context(|| format!("Error while reading test {}", test_path.display()))?;

    let config: TestConfigFile = toml::from_str(&file)
        .with_context(|| format!("Error while parsing test {}", test_path.display()))?;

    let base_path = test_path.parent().context("getting base path")?;

    match generate_config.target {
        GeneratorTarget::Output => {
            let test_config = config.clone().into_config(base_path)?;

            let frames = tester::run_test(&test_config, 0).await?;

            info!("Writing output to file");
            config
                .write_frames(base_path, frames)
                .context("writing frames to output file")?;
            println!("Ouput written");
        }
        GeneratorTarget::OutputHeaders => {
            config.generate_output_file(base_path)?;
        }
    }

    Ok(())
}

/// Run the test cases
async fn run_tests() -> anyhow::Result<()> {
    let (total_files, tests) = parse_test_cases()?;
    let total_tests = tests.len();

    let had_error = test_test(tests).await;

    if had_error {
        bail!("Some tests failed");
    } else if total_files != total_tests {
        println!(
            "\nAll tests passed! But {} out of {} tests where ignored due to errors reading the test specification.",
            total_files - total_tests,
            total_files
        );
    } else {
        println!("\nAll tests passed!");
    }
    Ok(())
}

fn parse_test_cases() -> anyhow::Result<(usize, Vec<(String, config::TestConfig)>)> {
    let base_tests_path = Path::new("./integration-tests");
    let tests = std::fs::read_dir(base_tests_path).context(
        "while looking for `integration-tests` dir.\n(HELP: Are you runnig from the root dir?)",
    )?;
    let base_tests_path = base_tests_path.canonicalize()?;
    let mut total_files = 0;
    let tests = tests
        .filter_map(|path| {
            let Ok(path) = path else {
                eprintln!("Error while looking for tests");
                return None;
            };
            match path.metadata() {
                Ok(m) => {
                    if !m.is_file() {
                        return None;
                    }
                }
                Err(e) => {
                    eprintln!("Error while reading test {}: {}", path.path().display(), e);
                    return None;
                }
            };
            let path = path.path();
            if path.extension() != Some(&std::ffi::OsString::from("toml")) {
                return None;
            }
            total_files += 1;

            let file = match std::fs::read_to_string(&path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error while reading test {}: {}", path.display(), e);
                    return None;
                }
            };

            let config: TestConfigFile = match toml::from_str(&file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error while parsing test {}:\n{}", path.display(), e);
                    return None;
                }
            };

            let config = match config.into_config(&base_tests_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error while parsing test {}:\n{:#}", path.display(), e);
                    debug!("{}", e.backtrace());
                    return None;
                }
            };

            trace!("Found test {}:\n{:#?}", path.display(), config);

            Some((path.display().to_string(), config))
        })
        .collect::<Vec<_>>();

    println!("Found {} valid tests.", tests.len());
    Ok((total_files, tests))
}

/// Run all the tests
async fn test_test(tests: Vec<(String, config::TestConfig)>) -> bool {
    let mut had_error = false;
    for (idx, (name, test)) in tests.into_iter().enumerate() {
        print!("Running `{name}`{}", " ".repeat(36 - name.chars().count()));
        std::io::stdout().flush().ok();
        if let Err(e) = tester::run_test_and_cmp(test, idx as u32).await {
            had_error = true;
            println!(": Fail!\n{e:#}\n");
        } else {
            println!(": Pass :)");
        }
    }
    had_error
}
