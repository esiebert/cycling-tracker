pub mod api;
pub mod service;
use anyhow::Result;
use api::{grpc::Builder, SQLite, GRPC};
use service::{CyclingTrackerService, SessionAuthService};
use tonic_reflection::server::Builder as ReflectionServerBuilder;

type GRPCResult<T> = Result<tonic::Response<T>, tonic::Status>;

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
    pub fn new(grpc_host_url: String) -> Self {
        let sqlite = SQLite::new();

        let auth = cycling_tracker::SessionAuthServer::new(SessionAuthService {});

        let cts = cycling_tracker::CyclingTrackerServer::new(
            CyclingTrackerService::new(sqlite.handler()),
        );

        let refl = ReflectionServerBuilder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build()
            .expect("Failed to setup reflection service");

        let grpc = Builder::new()
            .with_addr(grpc_host_url)
            .expect("Failed to set address")
            .with_tls()
            .expect("Failed when setting up TLS")
            .add_auth_service(auth)
            .add_reflection_service(refl)
            .add_ct_service(cts, false)
            .build()
            .expect("Failed to build gRPC");

        Self {
            grpc: grpc,
            sqlite: sqlite,
        }
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
