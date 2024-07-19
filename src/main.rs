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
        .setup_sqlite()
        .setup_grpc(String::from("[::1]:10000"))?
        .build()?;

    app.run().await?;

    Ok(())
}
