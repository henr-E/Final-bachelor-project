use std::path::{Path, PathBuf};

/// Walks the directory tree until it finds `file`.
///
/// Stops when it runs into permission errors or it cannot go up an further.
///
/// NOTE: This function can both be used to find files and directories.
///
/// In debug mode this function will also stop searching when it finds the root of a cargo project.
pub fn walk_dir_tree_for_file(file: impl AsRef<Path>) -> Option<PathBuf> {
    let file = file.as_ref();
    let mut current_dir = std::env::current_dir().ok()?;

    while !current_dir.join(file).exists() {
        // Safe guard for development. Stop searching when the root of a cargo project is reached.
        // Disables this safeguard when compiling in release mode.
        #[cfg(debug_assertions)]
        if current_dir.join("Cargo.lock").exists() {
            return None;
        }

        // Go up a directory.
        current_dir = current_dir.join("../");

        if !current_dir.exists() {
            // We cannot go higher up the tree or a permission error occured.
            return None;
        }
    }

    Some(current_dir.join(file))
}
