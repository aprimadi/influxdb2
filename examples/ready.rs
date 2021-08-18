#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let influx_url = "some-url";
    let token = "some-token";

    let client = influxdb2::Client::new(influx_url, "org", token);

    println!("{:?}", client.ready().await?);

    Ok(())
}
