use std::{pin::Pin, vec::IntoIter};

use redis::Commands;
use sqlx::SqlitePool;
use tokio::{net::TcpListener, task::spawn};
use tokio_stream::{wrappers::TcpListenerStream, Iter, StreamExt};
use tonic::{metadata::MetadataValue, transport::channel::Channel, Request};

use cycling_tracker::cycling_tracker::cycling_tracker_client::CyclingTrackerClient;
use cycling_tracker::cycling_tracker::session_auth_client::SessionAuthClient;
use cycling_tracker::App;

pub struct TestEnvironment {
    pub ct_service: CyclingTrackerClient<Channel>,
    pub auth_service: SessionAuthClient<Channel>,
}

pub async fn run_test_env(db: SqlitePool) -> TestEnvironment {
    let redis_client = redis::Client::open("redis://127.0.0.1/")
        .expect("Failed to start redis client");
    let mut conn = redis_client
        .clone()
        .get_connection()
        .expect("Failed to connect to redis");

    // Add always-valid session-token
    conn.set::<_, _, ()>("session-token", "user1").unwrap();

    let grpc_addr = "127.0.0.1:0";

    // Build app
    let app = App::builder()
        // Disable TLS and session tokens for test purposes
        .with_db(db)
        .with_redis(redis_client)
        .setup_grpc(&grpc_addr, false)
        .await
        .expect("Failed to setup gRPC")
        .build()
        .expect("Failed to build App");

    // Setup TCP listener and get address with assigned port
    let listener = TcpListener::bind(&grpc_addr).await.unwrap();
    let grpc_addr = listener.local_addr().unwrap();

    // Run app
    spawn(app.run_tcp(TcpListenerStream::new(listener)));

    // Get CT service client
    let ct_service = CyclingTrackerClient::connect(format!("http://{}", grpc_addr))
        .await
        .expect("Failed to connect to gRPC CT Server");

    // Get auth service client
    let auth_service = SessionAuthClient::connect(format!("http://{}", grpc_addr))
        .await
        .expect("Failed to connect to gRPC CT Server");

    TestEnvironment {
        ct_service,
        auth_service,
    }
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
    with_metadata(Request::new(Box::pin(stream)))
}

pub fn with_metadata<T>(mut req: Request<T>) -> Request<T> {
    let token: MetadataValue<_> = "session-token".parse().unwrap();
    req.metadata_mut().insert("authorization", token);
    req
}
