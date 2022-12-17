use futures::prelude::*;

use influxdb2::Client;
use influxdb2::api::write::TimestampPrecision;
use influxdb2::models::DataPoint;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = std::env::var("INFLUXDB_HOST").unwrap();
    let org = std::env::var("INFLUXDB_ORG").unwrap();
    let token = std::env::var("INFLUXDB_TOKEN").unwrap();
    let bucket = "bucket";
    let client = Client::new(host, org, token);
    
    let points = vec![
        DataPoint::builder("cpu")
            .tag("host", "server01")
            .field("usage", 0.5)
            .timestamp(1671095854)
            .build()?,
    ];
                                                            
    client.write_with_precision(bucket, stream::iter(points), TimestampPrecision::Seconds).await?;
    
    Ok(())
}

