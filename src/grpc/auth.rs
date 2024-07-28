use tonic::{Request, Response, Status};

use crate::cycling_tracker::{
    session_auth_server::SessionAuth, Credentials, SessionToken,
};

#[derive(Debug)]
pub struct SessionAuthService {}

#[tonic::async_trait]
impl SessionAuth for SessionAuthService {
    async fn login(
        &self,
        request: Request<Credentials>,
    ) -> Result<Response<SessionToken>, Status> {
        println!("Login = {:?}", request);
        let credentials = request.get_ref();
        if credentials.username == "root" && credentials.password == "admin" {
            return Ok(Response::new(SessionToken {
                token: "session-token".to_string(),
            }));
        }

        Err(Status::unauthenticated("Invalid credentials"))
    }
}
