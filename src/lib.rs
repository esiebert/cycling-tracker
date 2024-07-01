pub mod actor;
pub mod service;
pub mod util;

use actor::GRPCActor;

pub mod useless_box {
    tonic::include_proto!("uselesspackage");

    pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("useless.bin");
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
