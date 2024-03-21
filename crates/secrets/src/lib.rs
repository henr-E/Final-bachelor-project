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
        return Some(var);
    }
    std::fs::read_to_string(format!(".secrets/{key}")).ok()
}
