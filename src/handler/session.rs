use tonic::{Request, Status};

use crate::handler::RedisHandler;

#[derive(Clone)]
pub struct SessionHandler {
    pub redis_handler: RedisHandler,
}

impl SessionHandler {
    pub fn start(&self, user_name: String) -> String {
        let session_token = uuid7::uuid7().to_string();

        // Session token expires in 5 minutes
        self.redis_handler
            .set_key(&session_token, &user_name, Some(300));

        println!(
            "Created session token {:?} for user {:?}",
            &session_token, &user_name
        );

        session_token
    }

    pub fn verify_session_token<RT>(
        &self,
        request: &Request<RT>,
    ) -> Result<String, Status> {
        let session_token = request
            .metadata()
            .get("Authorization")
            .ok_or(Status::unauthenticated("Session token not provided"))?
            .to_str()
            .unwrap();

        let user_name = self
            .redis_handler
            .get_key(session_token)
            .ok_or(Status::unauthenticated("Invalid session token"))?;

        println!(
            "Session token {:?} correlates to user {:?}",
            &session_token, &user_name
        );

        Ok(user_name)
    }
}
