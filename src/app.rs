use anyhow::Result;
use thiserror::Error;
use tokio_stream::wrappers::TcpListenerStream;
use tonic_reflection::server::Builder as ReflectionServerBuilder;

use crate::grpc::{BuildError as GRPCBuildError, Builder as GRPCBuilder, GRPC, cycling_tracker::CyclingTrackerService, auth::SessionAuthService};
use crate::cycling_tracker;
use crate::handler::{SQLiteHandler, WorkoutHandler};
use crate::FILE_DESCRIPTOR_SET;

pub struct App {
    grpc: GRPC,
}

impl App {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub async fn run(mut self) -> Result<()> {
        tokio::select! {
            e = self.grpc.run() => {
                e
            }
        }
    }

    pub async fn run_tcp(mut self, tcp_listener: TcpListenerStream) -> Result<()> {
        tokio::select! {
            e = self.grpc.run_tcp(tcp_listener) => {
                e
            }
        }
    }
}

pub struct Builder {
    grpc: Option<GRPC>,
}

impl Builder {
    pub fn new() -> Self {
        Self { grpc: None }
    }

    pub fn setup_grpc(
        mut self,
        host_url: &str,
        with_tls: bool,
        with_session_tokens: bool,
    ) -> Result<Self, BuildError> {
        let auth = cycling_tracker::SessionAuthServer::new(SessionAuthService {});

        let cts = cycling_tracker::CyclingTrackerServer::new(
            CyclingTrackerService::new(WorkoutHandler {
                sqlite_handler: SQLiteHandler {},
            }),
        );

        let refl = ReflectionServerBuilder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build()
            .map_err(|e| BuildError::ReflectionBuildError(e.to_string()))?;

        let mut grpc_builder = GRPCBuilder::new().with_addr(host_url)?;

        if with_tls {
            grpc_builder = grpc_builder.with_tls()?;
        }

        let grpc = grpc_builder
            .add_auth_service(auth)
            .add_reflection_service(refl)
            .add_ct_service(cts, with_session_tokens)
            .build()?;

        self.grpc = Some(grpc);
        Ok(self)
    }

    pub fn build(self) -> Result<App, BuildError> {
        let grpc = self.grpc.ok_or(BuildError::GRPCNotSet)?;

        Ok(App { grpc })
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Failed to build gRPC: {0}")]
    GRPCBuildFailure(#[from] GRPCBuildError),
    #[error("Failed to build reflection server: {0}")]
    ReflectionBuildError(String),
    #[error("gRPC service not set")]
    GRPCNotSet,
}
