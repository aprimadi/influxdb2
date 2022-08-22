use std::env;

use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use futures::stream;
use influxdb2::models::DataPoint;
use influxdb2::models::Query;
use influxdb2::Client;
use influxdb2_structmap::FromMap;


#[derive(Debug, influxdb2_structmap_derive::FromMap)]
pub struct StockPrice {
   ticker: String,
   value: f64,
   open: f64,
   time: DateTime<FixedOffset>,
}

impl Default for StockPrice {
    fn default() -> Self {
        let now = Utc::now().naive_utc();
        Self {
            ticker: "".to_owned(),
            value: 0.0,
            open: 0.0,
            time: FixedOffset::east(7 * 3600).from_utc_datetime(&now),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host    = env::var("INFLUXDB_HOST").unwrap();
    let org     = env::var("INFLUXDB_ORG").unwrap();
    let token   = env::var("INFLUXDB_TOKEN").unwrap();
    let bucket  = env::var("INFLUXDB_BUCKET").unwrap();

    let client = Client::new(host, org, token);


    println!("HealthCheck: {:#?}", client.health().await?);
    let points: Vec<DataPoint> = vec![ 
        DataPoint::builder("bar")
            .tag("ticker", "AAPL")
            .field("value", 123.46)
            .field("open", 200.0)
            .build()?,
        DataPoint::builder("bar")
            .tag("ticker", "GOOG")
            .field("value", 321.09)
            .field("open", 309.2)
            .build()?,
    ];
    client.write(&bucket, stream::iter(points)).await?;
    let qs = format!("from(bucket: \"{}\") 
      |> range(start: -1w)
      |> last()
   ", bucket);
    let query = Query::new(qs.to_string());

    println!(
        "Query result was: {:#?}", 
        client.query::<StockPrice>(Some(query)).await?
    );

    Ok(())
}

