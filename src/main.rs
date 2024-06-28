use useless_box::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new(String::from("[::1]:10000"));
    app.run().await?;

    Ok(())
}
