use std::net::SocketAddr;

use anyhow::Result;
use thiserror::Error;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{server::Router, Identity, Server, ServerTlsConfig};
use tonic_reflection::server::{ServerReflection, ServerReflectionServer};
use tracing::{info, instrument};

use crate::cycling_tracker::{CyclingTrackerServer, SessionAuthServer};

pub mod auth;
pub mod cycling_tracker;

use auth::SessionAuthService;
use cycling_tracker::CyclingTrackerService;

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

        let cert =
            read_to_string(data_dir.join("example_public.pem")).map_err(|err| {
                BuildError::TLSSetupError(format!("Error reading public key: {}", err))
            })?;

        let key =
            read_to_string(data_dir.join("example_private.key")).map_err(|err| {
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
    ) -> Self {
        match self.router {
            Some(r) => self.router = Some(r.add_service(service)),
            None => self.router = Some(self.server.add_service(service)),
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
