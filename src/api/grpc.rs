use std::net::SocketAddr;

use anyhow::Result;
use thiserror::Error;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    metadata::MetadataValue,
    service::interceptor::InterceptedService,
    transport::{server::Router, Identity, Server, ServerTlsConfig},
    Request, Status,
};
use tonic_reflection::server::{ServerReflection, ServerReflectionServer};
use tracing::{info, instrument};

use crate::cycling_tracker::{CyclingTrackerServer, SessionAuthServer};
use crate::service::{CyclingTrackerService, SessionAuthService};

#[derive(Debug)]
pub struct GRPC {
    addr: SocketAddr,
    // Wrap router with Option, because we will have to swap its content
    // with another aux Option<Router>. See run().
    router: Option<Router>,
}

impl GRPC {
    pub fn builder() -> Builder {
        Builder::new()
    }

    #[instrument(name = "gRPC::run", skip(self), err)]
    pub async fn run(&mut self) -> Result<()> {
        info!("CyclingTracker listening on: {}", self.addr);

        // Router doesn't implement clone, so we create an auxiliary variable,
        // swap its contents, and use it to create GRPC.
        // Aux is Option<Router> because there's no easy way to instantiate
        // a Router
        let mut router: Option<Router> = None;
        std::mem::swap(&mut self.router, &mut router);

        router.unwrap().serve(self.addr).await?;

        Ok(())
    }

    #[instrument(name = "gRPC::run", skip(self), err)]
    pub async fn run_tcp(&mut self, tcp_listener: TcpListenerStream) -> Result<()> {
        info!("CyclingTracker listening on TCP: {}", self.addr);

        // Router doesn't implement clone, so we create an auxiliary variable,
        // swap its contents, and use it to create GRPC.
        // Aux is Option<Router> because there's no easy way to instantiate
        // a Router
        let mut router: Option<Router> = None;
        std::mem::swap(&mut self.router, &mut router);

        router.unwrap().serve_with_incoming(tcp_listener).await?;

        Ok(())
    }
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
    addr: Option<SocketAddr>,
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

    pub fn with_addr(mut self, addr: &str) -> Result<Self, BuildError> {
        let socket_addr = addr.parse().map_err(|err| {
            BuildError::InvalidAddr(format!("Can't parse address: {}", err))
        })?;
        self.addr = Some(socket_addr);

        Ok(self)
    }

    pub fn with_tls(mut self) -> Result<Self, BuildError> {
        use std::fs::read_to_string;
        use std::path::PathBuf;

        let data_dir =
            PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR"), "data/tls"]);

        let cert = read_to_string(data_dir.join("example_public.pem")).map_err(|err| {
            BuildError::TLSSetupError(format!("Error reading public key: {}", err))
        })?;

        let key = read_to_string(data_dir.join("example_private.key")).map_err(|err| {
            BuildError::TLSSetupError(format!("Error reading private key: {}", err))
        })?;

        let config_tls = ServerTlsConfig::new().identity(Identity::from_pem(cert, key));

        self.server = self.server.tls_config(config_tls).map_err(|err| {
            BuildError::TLSSetupError(format!("Error configuring TLS: {}", err))
        })?;

        Ok(self)
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

    pub fn build(&mut self) -> Result<GRPC, BuildError> {
        let addr = self.addr.ok_or(BuildError::AddrNotSet)?;

        // Router doesn't implement clone, so we create an auxiliary variable,
        // swap its contents, and use it to create GRPC
        let mut router: Option<Router> = None;
        std::mem::swap(&mut self.router, &mut router);

        router.as_ref().ok_or(BuildError::RouterNotConfigured)?;

        Ok(GRPC { router, addr })
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Unable to setup TLS: {0}")]
    TLSSetupError(String),
    #[error("Invalid socket address: {0}")]
    InvalidAddr(String),
    #[error("Socket address not set")]
    AddrNotSet,
    #[error("Router was not configured. Please add at least one service")]
    RouterNotConfigured,
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
