pub mod api;
pub mod app;
pub mod handler;
pub mod service;

pub mod cycling_tracker {
    tonic::include_proto!("cyclingtracker");

    pub use cycling_tracker_server::CyclingTrackerServer;
    pub use session_auth_server::SessionAuthServer;
}

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("fds/cyclingtracker.bin");

pub use app::App;
