use cycling_tracker::App;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let format = tracing_subscriber::fmt::format().with_target(false);

    tracing_subscriber::fmt()
        .event_format(format)
        .with_max_level(Level::INFO)
        .init();

    info!("Starting gRPC server");

    let app = App::builder()
        .setup_redis("redis://127.0.0.1/")?
        .setup_database("sqlite:ct.db")
        .await?
        .setup_grpc("[::0]:10000", true)
        .await?
        .build()?;

    app.run().await?;

    Ok(())
}
