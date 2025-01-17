//! Crate for retrieving environment variable from dotenv file.

pub use crate::{get_env_variable as env, value_is_truthy as is_truthy};

use anyhow::Error as AnyhowError;
use once_cell::sync::OnceCell;
use std::{borrow::Cow, collections::HashMap};
use thiserror::Error;

static ENV_VARS: OnceCell<HashMap<String, String>> = OnceCell::new();

#[derive(Error, Debug)]
pub enum Error {
    #[error("`.env` file could not be found")]
    EnvFileNotFound,
    #[error("{0}")]
    Generic(#[from] anyhow::Error),
    #[error("following variables not found in `.env`: `{0:?}`")]
    VarNotFound(Cow<'static, str>),
    #[error("error parsing `.env`: {0}")]
    Dotenvy(#[from] dotenvy::Error),
}

/// Check if an environment variable has a truthy value.
pub fn value_is_truthy(val: impl AsRef<str>) -> bool {
    // Implementation taken from https://github.com/sagiegurari/envmnt/blob/master/src/util.rs
    let val = val.as_ref().to_lowercase();
    !val.is_empty() && val != "0" && val != "false" && val != "no"
}

fn get_env_variables() -> Result<&'static HashMap<String, String>, Error> {
    ENV_VARS.get_or_try_init(|| {
        {
            // Collect the variables in the file into a `HashMap`. Handle errors that might occur
            // because the file was not found, could not be read, contains a syntax error, etc.
            match dotenvy::dotenv_iter().map(|i| i.collect::<Result<HashMap<_, _>, _>>()) {
                Ok(env_vars) => Ok(env_vars?),
                Err(err) if err.not_found() => Err(Error::EnvFileNotFound),
                Err(err) => Err(AnyhowError::new(err)
                    .context("error loading `.env` file")
                    .into()),
            }
        }
        // Check if the user has enabled the use of environment variables.
        .or_else(|err| match err {
            Error::EnvFileNotFound => {
                let env_impure = std::env::var("ENV_IMPURE")
                    .map(value_is_truthy)
                    .unwrap_or(false);

                if !env_impure {
                    return Err(err);
                }

                // Insert current environment variables. (not from `.env`)
                let vals = std::env::vars().collect::<HashMap<_, _>>();
                Ok(vals)
            }
            e => Err(e),
        })
    })
}

/// Get the value of an environment variable from the environment variable hashmap.
pub fn get_env_variable(var: impl Into<Cow<'static, str>>) -> Result<&'static str, Error> {
    let env_vars = get_env_variables()?;
    let var = var.into();

    Ok(env_vars
        .get(var.as_ref())
        .ok_or_else(|| Error::VarNotFound(var))?)
}

#[cfg(test)]
mod tests {
    use super::value_is_truthy;

    #[test]
    fn truthy_values() {
        assert!(["1", "true", "any", "foo", "yes", "CaP", "WEirD"]
            .into_iter()
            .all(value_is_truthy));
    }

    #[test]
    fn falsy_values() {
        assert!(!["", "0", "no", "false"].into_iter().any(value_is_truthy));
    }
}
