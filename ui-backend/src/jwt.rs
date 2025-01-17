use chrono::{Days, Utc};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{
    decode, encode, errors::Error as JwtError, Algorithm, DecodingKey, EncodingKey, Header,
    Validation,
};
use serde::{Deserialize, Serialize};

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
/// Returns a 'Result' containing the token if succes, or an 'ErrorKind::InvalidToken if an error occurs
///
/// # Errors
///
/// 'ErrorKind::InvalidToken'

pub fn create_jwt(username: &str) -> Result<Jwt, ErrorKind> {
    let secret = secrets::secret("JWT_SECRET").expect("failed to read secret");

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

pub fn verify_jwt(token: &str) -> Result<String, JwtError> {
    // Decode token
    let decoding_key = DecodingKey::from_secret(
        secrets::secret("JWT_SECRET")
            .expect("failed to read secret")
            .as_ref(),
    );
    let validation = Validation::new(Algorithm::HS256);
    let decoded_token = decode::<Claims>(token, &decoding_key, &validation)?;

    // Extract username from claims
    let username = decoded_token.claims.username;

    Ok(username)
}
