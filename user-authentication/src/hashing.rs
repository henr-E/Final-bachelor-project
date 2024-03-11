use argon2::{
    password_hash::{
        rand_core::OsRng, Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};

/// Hashes a password using Argon2id algorithm.
///
/// # Arguments
///
/// * `password` - &str
///
/// # Returns
///
/// Returns a `Result` containing the hashed password as a string if successful, or an `AuthenticationError::HashingError` if an error occurs
///
/// # Errors
///
/// `AuthenticationError::HashingError`
pub fn hash_password(password: &[u8]) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2.hash_password(password, &salt);

    let password_hash_string = password_hash.unwrap().serialize();

    password_hash_string.to_string()
}
/// Verifies a password against its hashed counterpart.
///
/// # Arguments
///
/// * `password` - A string slice containing the password to be verified.
/// * `hash` - A string slice containing the hashed password to be verified against.
///
/// # Returns
///
/// Returns a `Result` containing a boolean value indicating whether the password matches the hashed password or not, or an `AuthenticationError::HashingError` if an error occurs during verification.
///
/// # Errors
///
/// Returns an `AuthenticationError::HashingError` if an error occurs during verification.
///
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<(), Error> {
    let _verifier = Argon2::default();

    let parsed_hash = PasswordHash::new(hash);

    let hashed_result = match parsed_hash {
        Ok(parsed_hash) => parsed_hash,
        Err(_) => return Err(argon2::password_hash::Error::Password),
    };

    let is_valid = Argon2::default().verify_password(password.as_bytes(), &hashed_result);

    is_valid
}

#[cfg(test)]
mod test_hashing {
    use super::*;

    /// test the hashing and verification
    #[test]
    fn test_hashing_and_verification() {
        let password = "secretpassword";

        let hash = hash_password(password.as_bytes());

        let is_match = verify_password(password, &hash);

        assert!(is_match.is_ok());
    }

    /// test the hashing and verification with a wrong password
    #[test]
    fn test_wrong_verification() {
        let password = "secretpassword".as_bytes();

        let hash = hash_password(password);

        let is_not_match = verify_password("wrong password", &hash);

        assert!(is_not_match.is_err());
    }
}
