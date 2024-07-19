use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let org = "sahamee";
    let bucket = "bucket";
    let influx_url = "http://localhost:8086";
    let token = std::env::var("INFLUXDB2_TOKEN").unwrap();

    let client = influxdb2::Client::new(influx_url, org, token);

    let measurements = client
        .list_measurements(bucket, Some("-365d"), Some("-1d"))
        .await
        .unwrap();
    println!("measurements: {:?}", measurements);

    for m in measurements.iter() {
        let field_keys = client
            .list_measurement_field_keys(bucket, m, Some("-365d"), Some("now()"))
            .await
            .unwrap();
        println!("field keys: {:?}", field_keys);
    }

    for m in measurements.iter() {
        let tag_values = client
            .list_measurement_tag_values(bucket, m, "host", Some("-365d"), None)
            .await;
        println!(
            "tag values for measurement {} and tag host: {:?}",
            &m, tag_values
        );
    }

    for m in measurements.iter() {
        let tag_values = client
            .list_measurement_tag_keys(bucket, m, Some("-365d"), None)
            .await;
        println!(
            "tag values for measurement {} and tag host: {:?}",
            &m, tag_values
        );
    }

    Ok(())
}
