static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let influx_url = "http://localhost:8086";
    let token = "some-token";

    let builder = influxdb2::ClientBuilder::with_builder(
        reqwest::ClientBuilder::new().user_agent(APP_USER_AGENT),
        influx_url,
        "org",
        token,
    );
    let client = builder.build()?;

    println!("{:?}", client.health().await?);

    Ok(())
}
