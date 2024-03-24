//! Library to facilitate reading secrets.
#![deny(missing_docs)]

/// Tries to read the secret with the specified name. Returns `None` if there is no such secret.
///
/// It will look for the secret in the following order:
/// - A variable from `.env` with name `key`.
/// - As a variable from the system environment with name `key`.
/// - The contents of the file with path: `/run/secrets/[key]`. (where docker stores them)
/// - The contents of the file with path: `.secrets/[key]`.
pub fn secret(key: &str) -> Option<String> {
    if let Ok(var) = dotenvy::var(key) {
        return Some(var);
    }
    if let Ok(var) = std::fs::read_to_string(format!("/run/secrets/{key}")) {
        return Some(var.trim_end().to_string());
    }
    walk_dir_tree_for_secrets(key).map(|s| s.trim_end().to_string())
}

/// Walk the directory tree until it finds the `.secrets/{key}` file.
///
/// Stops when it runs into permission errors or it cannot go up an further.
fn walk_dir_tree_for_secrets(key: &str) -> Option<String> {
    let file = std::path::PathBuf::from(format!(".secrets/{key}"));
    let mut current_dir = std::env::current_dir().ok()?;

    while !current_dir.join(&file).exists() {
        // Go up a directory.
        current_dir = current_dir.join("../");

        if !current_dir.exists() {
            // We cannot go higher up the tree.
            return None;
        }
    }

    std::fs::read_to_string(current_dir.join(file)).ok()
}
