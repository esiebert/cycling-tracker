use tonic::{Request, Response, Status};

use crate::cycling_tracker::{
    session_auth_server::SessionAuth, Credentials, SessionToken, SignUpResult,
};
use crate::handler::UserHandler;

pub struct SessionAuthService {
    pub user_handler: UserHandler,
}

impl SessionAuthService {
    pub fn new(user_handler: UserHandler) -> Self {
        Self { user_handler }
    }
}

#[tonic::async_trait]
impl SessionAuth for SessionAuthService {
    async fn sign_up(
        &self,
        request: Request<Credentials>,
    ) -> Result<Response<SignUpResult>, Status> {
        let credentials = request.into_inner();
        println!("Sign up for user = {:?}", credentials.username);

        if self.user_handler.create(credentials).await {
            return Ok(Response::new(SignUpResult { result: true }));
        }

        Err(Status::already_exists("Username already taken"))
    }

    async fn login(
        &self,
        request: Request<Credentials>,
    ) -> Result<Response<SessionToken>, Status> {
        let credentials = request.into_inner();
        println!("Login request from user = {:?}", credentials.username);

        if self.user_handler.login(credentials).await {
            return Ok(Response::new(SessionToken {
                token: "session-token".to_string(),
            }));
        }

        Err(Status::unauthenticated("Invalid credentials"))
    }
}
