use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthenticationError {
    /// AuthenticationError indicating invalid username or password
    #[error("invalid username or password")]
    InvalidUsernameOrPassword,

    /// AuthenticationError when a hashing error occurs
    #[error("hashing error")]
    HashingError,

    #[error("creating jwt error")]
    JwtError,
}
