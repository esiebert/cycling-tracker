use std::{pin::Pin, vec::IntoIter};

use sqlx::SqlitePool;
use tokio::{net::TcpListener, task::spawn};
use tokio_stream::{wrappers::TcpListenerStream, Iter, StreamExt};
use tonic::{transport::channel::Channel, Request};

use cycling_tracker::cycling_tracker::cycling_tracker_client::CyclingTrackerClient;
use cycling_tracker::App;

pub struct TestEnvironment {
    pub grpc_client: CyclingTrackerClient<Channel>,
}

pub async fn run_test_env(db: SqlitePool) -> TestEnvironment {
    let addr = "127.0.0.1:0";

    // Build app
    let app = App::builder()
        // Disable TLS and session tokens for test purposes
        .with_db(db)
        .setup_grpc(&addr, false, false)
        .await
        .expect("Failed to setup gRPC")
        .build()
        .expect("Failed to build App");

    // Setup TCP listener and get address with assigned port
    let listener = TcpListener::bind(&addr).await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Run app
    spawn(app.run_tcp(TcpListenerStream::new(listener)));

    // Get gRPC client
    let grpc_client = CyclingTrackerClient::connect(format!("http://{}", addr))
        .await
        .expect("Failed to connect to gRPC CT Server");

    TestEnvironment { grpc_client }
}

pub async fn stream_to_vec<T>(mut stream: tonic::Streaming<T>) -> Vec<T> {
    let mut vec = vec![];

    while let Some(res) = stream.next().await {
        vec.push(res.unwrap());
    }

    vec
}

pub fn vec_to_stream<T>(vec: Vec<T>) -> Request<Pin<Box<Iter<IntoIter<T>>>>> {
    let stream = tokio_stream::iter(vec);
    Request::new(Box::pin(stream))
}
