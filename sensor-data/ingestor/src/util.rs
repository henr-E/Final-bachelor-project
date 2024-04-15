use crate::{DEFAULT_UPLOADS_DIR, UPLOADS_DIR_ENV_NAME};
use environment_config::env;
use proto::sensor_data_ingest::sensor_data_file::FileFormat as ProtoSensorDataFileFormat;
use std::{
    io::ErrorKind as IoErrorKind,
    path::{Path, PathBuf},
};
use tokio::sync::OnceCell;
use tracing::debug;
use walk_dir_tree::walk_dir_tree_for_file;

static UPLOADS_DIR: OnceCell<PathBuf> = OnceCell::const_new();

/// Get the uploads directory from the singleton.
///
/// If the $`UPLOADS_DIR_ENV_NAME` variable is set, then that path will be used and a directory
/// will be created, unless it is already present.
///
/// If the environment variable was not set, the [`DEFAULT_UPLOADS_DIR`] constant will be used. It
/// will walk the directory tree upwards until it has found the directory. If not, it will be
/// created relative to your current directory.
///
/// # Panics
///
/// * If the user has incorrect permissions to create the directory.
pub async fn get_uploads_dir() -> &'static Path {
    UPLOADS_DIR
        .get_or_init(|| async {
            let uploads_dir = env(UPLOADS_DIR_ENV_NAME)
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| {
                    // Use the default directory, but walk the tree to find it.
                    match walk_dir_tree_for_file(DEFAULT_UPLOADS_DIR) {
                        Some(u) => u,
                        None => std::env::current_dir()
                            .expect("could not get current directory")
                            .join(DEFAULT_UPLOADS_DIR),
                    }
                });

            std::fs::create_dir(&uploads_dir)
                .or_else(|e| match e.kind() {
                    IoErrorKind::AlreadyExists => Ok(()),
                    e => Err(e),
                })
                .expect("could not create default uploads directory");

            uploads_dir
        })
        .await
}

/// Converts the file format used in the protobuf definition to the file format used by the parser.
///
/// # Errors
///
/// If the fields used for the configuration of the file format are invalid (e.g., multiple bytes
/// as the CSV record delimiter).
pub fn proto_file_format_to_parser_file_format(
    file_format: Option<ProtoSensorDataFileFormat>,
) -> Result<sensor_data_parser::FileFormat, impl Into<String>> {
    Ok(match file_format {
        Some(ProtoSensorDataFileFormat::Csv(csv)) => {
            // Check if the delimiter is a single byte. If not return an appropriate
            // error to the user.
            if csv.delimiter.as_bytes().len() != 1 {
                return Err("csv delimiter must be a single byte");
            }

            sensor_data_parser::FileFormat::Csv {
                delimiter: csv.delimiter.as_bytes()[0],
            }
        }
        Some(ProtoSensorDataFileFormat::Json(_)) => sensor_data_parser::FileFormat::Json,
        None => {
            debug!("File format not supplied! Using CSV with `;` as the delimiter.");
            sensor_data_parser::FileFormat::Csv { delimiter: b';' }
        }
    })
}
