use crate::filehandling::{create_file, find_password, find_username, write_to_file};
use crate::hashing::{hash_password, verify_password};
use crate::jwt::create_jwt;

use proto::auth::{
    login_response, AuthenticationService, LoginError, LoginRequest, LoginResponse,
    RegisterRequest, RegisterResponse, User,
};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct MyAuthenticationService {}

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
        let RegisterRequest { user } = request.into_inner();
        let Some(User { username, password }) = user else {
            return Err(Status::new(
                tonic::Code::InvalidArgument,
                "user field is required",
            ));
        };

        let _ = create_file();

        let _valid = match find_username(&username) {
            Ok(Some(false)) => &username,
            Ok(Some(true)) => {
                return Err(Status::new(
                    tonic::Code::AlreadyExists,
                    "username already exist",
                ));
            }
            Ok(None) => return Err(Status::new(tonic::Code::NotFound, "None")),
            Err(_) => return Err(Status::new(tonic::Code::Unknown, "some other error")),
        };

        let hash_password = hash_password(password.as_bytes());

        let write_result = write_to_file(username.as_str(), hash_password.as_str());

        if let Ok(()) = write_result {}
        Ok(Response::new(RegisterResponse {
            result: Some(proto::auth::register_response::Result::Succes(
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

        let failure = Some(String::from("failure"));

        let hash_password = match find_password(&username) {
            Ok(password) => password,
            Err(_error) => failure.clone(),
        };

        match find_username(&username) {
            Ok(username) => username,
            Err(_) => {
                return Ok(Response::new(LoginResponse {
                    result: Some(login_response::Result::Error(
                        LoginError::InvalidCredentials.into(),
                    )),
                }));
            }
        };

        if hash_password == failure {
            return Ok(Response::new(LoginResponse {
                result: Some(login_response::Result::Error(
                    LoginError::InvalidCredentials.into(),
                )),
            }));
        }

        let hashedpassword = hash_password.unwrap_or(String::new());

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
