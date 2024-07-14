use tonic::service::interceptor::InterceptedService;
use tonic_reflection::server::{ServerReflection, ServerReflectionServer};

use crate::cycling_tracker::cycling_tracker_server::CyclingTrackerServer;
use crate::cycling_tracker::session_auth_server::SessionAuthServer;
use tonic::{
    metadata::MetadataValue,
    transport::{server::Router, Identity, Server, ServerTlsConfig},
    Request, Status,
};

use crate::service::CyclingTrackerService;
use crate::service::SessionAuthService;

use tracing::info;

pub struct GRPC {
    addr: String,
    // Wrap router with Option, because we will have to swap its content
    // with another aux Option<Router>. See run().
    router: Option<Router>,
}

impl GRPC {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = self.addr.parse().unwrap();
        info!("CyclingTracker listening on: {}", addr);

        // Router doesn't implement clone, so we create an auxiliary variable,
        // swap its contents, and use it to create GRPC.
        // Aux is Option<Router> because there's no easy way to instantiate
        // a Router
        let mut router: Option<Router> = None;
        std::mem::swap(&mut self.router, &mut router);

        router.unwrap().serve(addr).await?;

        Ok(())
    }
}

fn config_tls() -> Result<ServerTlsConfig, Box<dyn std::error::Error>> {
    let data_dir =
        std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR"), "data/tls"]);
    let cert = std::fs::read_to_string(data_dir.join("server.pem"))
        .map_err(|err| format!("Error reading public key file: {}", err))?;
    let key = std::fs::read_to_string(data_dir.join("server.key"))
        .map_err(|err| format!("Error reading private key file: {}", err))?;
    Ok(ServerTlsConfig::new().identity(Identity::from_pem(cert, key)))
}

fn check_session_token(req: Request<()>) -> Result<Request<()>, Status> {
    let token: MetadataValue<_> = "Bearer session-token".parse().unwrap();

    match req.metadata().get("Authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("Invalid session token")),
    }
}

pub struct Builder {
    server: Server,
    addr: Option<String>,
    router: Option<Router>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            server: Server::builder(),
            addr: None,
            router: None,
        }
    }

    pub fn with_addr(mut self, addr: String) -> Self {
        self.addr = Some(addr);
        self
    }

    pub fn with_tls(mut self) -> Self {
        self.server = self.server.tls_config(config_tls().unwrap()).unwrap();
        self
    }

    pub fn add_auth_service(
        mut self,
        service: SessionAuthServer<SessionAuthService>,
    ) -> Self {
        match self.router {
            Some(r) => self.router = Some(r.add_service(service)),
            None => self.router = Some(self.server.add_service(service)),
        }
        self
    }

    pub fn add_reflection_service(
        mut self,
        service: ServerReflectionServer<impl ServerReflection>,
    ) -> Self {
        match self.router {
            Some(r) => self.router = Some(r.add_service(service)),
            None => self.router = Some(self.server.add_service(service)),
        }

        self
    }

    pub fn add_ct_service(
        mut self,
        service: CyclingTrackerServer<CyclingTrackerService>,
        intercept_session_token: bool,
    ) -> Self {
        if intercept_session_token {
            let intercepted_service =
                InterceptedService::new(service.clone(), check_session_token);
            match self.router {
                Some(r) => self.router = Some(r.add_service(intercepted_service)),
                None => {
                    self.router = Some(self.server.add_service(intercepted_service))
                }
            }
        } else {
            match self.router {
                Some(r) => self.router = Some(r.add_service(service)),
                None => self.router = Some(self.server.add_service(service)),
            }
        }
        self
    }

    pub fn build(&mut self) -> GRPC {
        let addr = self.addr.as_ref().unwrap();

        // Router doesn't implement clone, so we create an auxiliary variable,
        // swap its contents, and use it to create GRPC
        let mut router: Option<Router> = None;
        std::mem::swap(&mut self.router, &mut router);
        GRPC {
            router: router,
            addr: addr.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_check_session_token_valid() {
        let mut req = Request::new(());
        let token: MetadataValue<_> = "Bearer session-token".parse().unwrap();
        req.metadata_mut().insert("Authorization", token);

        let result = check_session_token(req);

        assert!(result.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn test_check_session_token_invalid() {
        let mut req = Request::new(());
        let token: MetadataValue<_> = "Bearer invalid-token".parse().unwrap();
        req.metadata_mut().insert("Authorization", token);

        let result = check_session_token(req);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn test_check_session_token_no_token() {
        let req = Request::new(());

        let result = check_session_token(req);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }
}
