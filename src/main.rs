use tracing::{info, Level};
use tracing_subscriber;
use cycling_tracker::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting gRPC server");

    let app = App::new(String::from("[::1]:10000"));
    app.run().await?;

    Ok(())
}
