use cycling_tracker::App;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let format = tracing_subscriber::fmt::format().with_target(false);

    tracing_subscriber::fmt()
        .event_format(format)
        .with_max_level(Level::INFO)
        .init();

    info!("Starting gRPC server");

    let app = App::new(String::from("[::1]:10000"));
    app.run().await?;

    Ok(())
}
