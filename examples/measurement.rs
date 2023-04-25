use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let org = "sahamee";
    let bucket = "bucket";
    let influx_url = "http://localhost:8086";
    let token = std::env::var("INFLUXDB2_TOKEN").unwrap();

    let client = influxdb2::Client::new(influx_url, org, token);

    let measurements = client.list_measurements(bucket).await.unwrap();
    println!("measurements: {:?}", measurements);

    for m in measurements.iter() {
        let field_keys = client
            .list_measurement_field_keys(bucket, &m)
            .await
            .unwrap();
        println!("field keys: {:?}", field_keys);
    }

    for m in measurements.iter() {
        let tag_values = client.list_measurement_tag_values(bucket, &m, "host").await;
        println!(
            "tag values for measurement {} and tag {}: {:?}",
            &m, "host", tag_values
        );
    }

    Ok(())
}
