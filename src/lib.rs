pub mod actor;
pub mod service;

use actor::GRPCActor;

type GRPCResult<T> = Result<tonic::Response<T>, tonic::Status>;

pub mod cycling_tracker {
    tonic::include_proto!("cyclingtracker");

    pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("cyclingtracker.bin");
}

pub struct App {
    grpc: GRPCActor,
}

impl App {
    pub fn new(grpc_host_url: String) -> Self {
        let grpc = GRPCActor::new(grpc_host_url);
        Self { grpc: grpc }
    }

    /// Run all actors.
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        tokio::select! {
            e = self.grpc.run() => {
                e
            }
        }
    }
}
