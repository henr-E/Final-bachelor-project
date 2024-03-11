use chrono::{Days, Utc};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::myerror::AuthenticationError;

const JWT_EXPIRATION_TIME: Days = Days::new(30);

/// Claims has an expiration time set at 1 month
/// and has an username associated with

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: i64,
    pub iat: i64,
    pub username: String,
}

impl Claims {
    pub fn new(username: &str) -> Self {
        let current_time = Utc::now();
        let expire = current_time + JWT_EXPIRATION_TIME;

        Claims {
            exp: expire.timestamp(),
            username: username.to_string(),
            iat: current_time.timestamp(),
        }
    }
}
pub type Jwt = String;

/// create a jwt with an associated claim
/// jwt is set to expire a month from creation
///
/// # Arguments
///
/// * 'username' - &str
///
/// # Returns
///
/// Returns a 'Result' containing the token if succes, or an 'AuthenticationError::InvalidToken if an error occurs
///
/// # Errors
///
/// 'AuthenticationError::InvalidToken'

pub fn create_jwt(username: &str) -> Result<Jwt, ErrorKind> {
    let secret = dotenvy::var("JWT_SECRET").expect("failed to create secret");

    let claims = Claims::new(username);

    let token = match encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(t) => Ok(t),
        Err(_err) => Err(ErrorKind::InvalidToken),
    };
    token
}
/// validate a json web token
/// This function was written for testing purpose
///
/// # Arguments
///
/// * 'token' - &str
///
/// # Returns
///
/// Returns a 'Result' containing the claim if succes, or an 'AuthenticationError::InvalidToken if an error occurs
///
/// # Errors
///
/// 'AuthenticationError::InvalidToken'

pub fn validate_jwt(token: &str) -> Result<Claims, AuthenticationError> {
    let secret = dotenvy::var("JWT_SECRET").expect("failed to create secret");

    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
    {
        Ok(c) => Ok(c),
        Err(_err) => Err(AuthenticationError::JwtError),
    };
    token_data
}
