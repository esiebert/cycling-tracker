use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::task;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::channel::Channel;

use cycling_tracker::cycling_tracker::cycling_tracker_client::CyclingTrackerClient;
use cycling_tracker::App;

pub async fn run_service_in_background() -> SocketAddr {
    let addr = "127.0.0.1:10000";

    let app = App::builder()
        .setup_sqlite()
        // Disable TLS and session tokens for test purposes
        .setup_grpc(&addr, false, false)
        .expect("Failed to setup gRPC")
        .build()
        .expect("Failed to build App");

    let listener = TcpListener::bind(&addr).await.unwrap();

    task::spawn(app.run_tcp(TcpListenerStream::new(listener)));

    addr.parse().unwrap()
}

pub async fn get_grpc_client(addr: SocketAddr) -> CyclingTrackerClient<Channel> {
    CyclingTrackerClient::connect(format!("http://{}", addr))
        .await
        .expect("Failed to connect to gRPC CT Server")
}

pub async fn run_service_and_get_client() -> CyclingTrackerClient<Channel> {
    let addr = run_service_in_background().await;
    get_grpc_client(addr).await
}
