pub mod actor;
pub mod service;
use service::CyclingTrackerService;

use actor::{GRPCActor, SQLiteActor};

type GRPCResult<T> = Result<tonic::Response<T>, tonic::Status>;

pub mod cycling_tracker {
    tonic::include_proto!("cyclingtracker");

    pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("fds/cyclingtracker.bin");
}

pub struct App {
    grpc: GRPCActor,
    sqlite: SQLiteActor,
}

impl App {
    pub fn new(grpc_host_url: String) -> Self {
        let sqlite = SQLiteActor::new();
        let cts = CyclingTrackerService::new(sqlite.handler());
        let grpc = GRPCActor::new(grpc_host_url, cts);
        Self {
            grpc: grpc,
            sqlite: sqlite,
        }
    }

    /// Run all actors.
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
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
