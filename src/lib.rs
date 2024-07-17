pub mod api;
pub mod service;
use anyhow::Result;
use api::grpc;
use api::{SQLite, GRPC};
use service::{CyclingTrackerService, SessionAuthService};
use thiserror::Error;
use tonic_reflection::server::Builder as ReflectionServerBuilder;

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("fds/cyclingtracker.bin");
pub mod cycling_tracker {
    tonic::include_proto!("cyclingtracker");

    pub use cycling_tracker_server::CyclingTrackerServer;
    pub use session_auth_server::SessionAuthServer;
}

pub struct App {
    grpc: GRPC,
    sqlite: SQLite,
}

impl App {
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Run all actors.
    pub async fn run(mut self) -> Result<()> {
        tokio::select! {
            e = self.grpc.run() => {
                e
            }
            e = self.sqlite.run() => {
                e
            }
        }
    }
}

pub struct Builder {
    grpc: Option<GRPC>,
    sqlite: Option<SQLite>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            grpc: None,
            sqlite: None,
        }
    }

    pub fn setup_grpc(mut self, host_url: String) -> Result<Self, BuildError> {
        let auth = cycling_tracker::SessionAuthServer::new(SessionAuthService {});

        let sqlite = self.sqlite.as_ref().expect("auefiaef");

        let cts = cycling_tracker::CyclingTrackerServer::new(
            CyclingTrackerService::new(sqlite.handler()),
        );

        let refl = ReflectionServerBuilder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build()
            .expect("Failed to setup reflection service");

        let grpc = grpc::Builder::new()
            .with_addr(host_url)
            .expect("eafae")
            .with_tls()
            .expect("eafae")
            .add_auth_service(auth)
            .add_reflection_service(refl)
            .add_ct_service(cts, false)
            .build()
            .expect("eafae");

        self.grpc = Some(grpc);
        Ok(self)
    }

    pub fn setup_sqlite(mut self) -> Self {
        self.sqlite = Some(SQLite::new());
        self
    }

    pub fn build(self) -> App {
        let grpc = self.grpc.expect("Was supposed to be here");
        let sqlite = self.sqlite.expect("Was supposed to be here");

        App {
            grpc: grpc,
            sqlite: sqlite,
        }
    }
}

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Failed to build gRPC: {0}")]
    GRPCBuildFailure(#[from] grpc::BuildError),
}
