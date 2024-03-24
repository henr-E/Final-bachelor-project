use crate::hashing::{hash_password, verify_password};
use crate::jwt::create_jwt;
use sqlx::PgPool;
use uuid::{NoContext, Timestamp, Uuid};

use proto::frontend::{
    login_response, register_response, AuthenticationService, LoginError, LoginRequest,
    LoginResponse, RegisterError, RegisterRequest, RegisterResponse, User,
};
use tonic::{Request, Response, Status};

pub struct MyAuthenticationService {
    pool: PgPool,
}

impl MyAuthenticationService {
    pub fn new(pool: PgPool) -> MyAuthenticationService {
        Self { pool }
    }
}

#[tonic::async_trait]
impl AuthenticationService for MyAuthenticationService {
    /// register a user by putting it at the moment in a global variable
    /// It will also hash the password
    ///
    /// # Arguments
    ///
    /// self, RegisterRequest which contains a User {username, password} who wants to register
    ///
    /// # Returns
    ///
    /// a register response indicating failure or succes
    ///
    /// # Errors
    ///
    /// tonic::Code::InvalidArgument (user field is required)
    /// username already taken
    async fn register_user(
        &self,
        request: Request<RegisterRequest>,
    ) -> std::result::Result<Response<RegisterResponse>, Status> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|err| Status::from_error(Box::new(err)))?;

        let RegisterRequest { user } = request.into_inner();
        let Some(User { username, password }) = user else {
            return Err(Status::new(
                tonic::Code::InvalidArgument,
                "user field is required",
            ));
        };

        // check if username is already in database
        let username_available = sqlx::query!(
            "SELECT exists(SELECT 1 FROM users WHERE username = $1)",
            username
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Status::internal(format!("Failed to fetch username: {}", e)))?
        .exists
        .unwrap();

        if username_available {
            return Ok(Response::new(RegisterResponse {
                result: Some(register_response::Result::Error(
                    RegisterError::UsernameTaken.into(),
                )),
            }));
        };

        let hash_password = hash_password(password.as_bytes());

        // creating an uuid based on unix timestamp
        let ts = Timestamp::from_unix(NoContext, 1497624119, 1234);

        let valid_id = Uuid::new_v7(ts);

        sqlx::query!(
            "INSERT INTO users (id, username, password_hash) VALUES ($1,$2,$3)",
            valid_id,
            username,
            hash_password,
        )
        .execute(&mut *transaction)
        .await
        .map_err(|err| Status::from_error(Box::new(err)))?;

        transaction
            .commit()
            .await
            .map_err(|err| Status::from_error(Box::new(err)))?;

        Ok(Response::new(RegisterResponse {
            result: Some(register_response::Result::Succes(
                "Success message".to_string(),
            )),
        }))
    }
    /// login a user by checking if the hash matches the one store in the global variable
    ///
    /// # Arguments
    ///
    /// self, LoginRequest which contains a User {username, password} who wants to login
    ///
    /// # Returns
    ///
    /// a register response indicating failure/succes/json web token
    ///
    /// # Errors
    ///
    /// tonic::Code::InvalidArgument (user field is required)
    /// InvalidCredentials
    async fn login_user(
        &self,
        request: Request<LoginRequest>,
    ) -> std::result::Result<Response<LoginResponse>, Status> {
        let LoginRequest { user } = request.into_inner();

        let Some(User { username, password }) = user else {
            return Err(Status::new(
                tonic::Code::InvalidArgument,
                "user field is required",
            ));
        };

        // get the password hash stored in the database
        let password_hash_database = sqlx::query!(
            "SELECT password_hash FROM users WHERE username = $1",
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Status::internal(format!("Failed to fetch user: {}", e)))?;

        // if username is not present in database it should return invalid credentials
        let hashedpassword = match password_hash_database {
            Some(e) => e.password_hash.clone(),
            None => {
                return Ok(Response::new(LoginResponse {
                    result: Some(login_response::Result::Error(
                        LoginError::InvalidCredentials.into(),
                    )),
                }))
            }
        };

        let validation = verify_password(&password, &hashedpassword);

        let jsonwebtoken = create_jwt(&username).expect("Couldn't create json web token");

        match validation {
            Ok(()) => Ok(Response::new(LoginResponse {
                result: Some(login_response::Result::Token(jsonwebtoken)),
            })),
            Err(_) => Ok(Response::new(LoginResponse {
                result: Some(login_response::Result::Error(
                    LoginError::InvalidCredentials.into(),
                )),
            })),
        }
    }
}
