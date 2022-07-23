# influxdb2

This is a Rust client to InfluxDB using the [2.0 API][2api].

[2api]: https://v2.docs.influxdata.com/v2.0/reference/api/

This project is a fork from the 
https://github.com/influxdata/influxdb_iox/tree/main/influxdb2_client project.
At the time of this writing, the query functionality of the influxdb2 client 
from the official repository isn't working. So, I created this client to use 
it in my project.

## Usage

### Querying

```rust
use chrono::{DateTime, FixedOffset};
use influxdb2::Client;
use influxdb2::models::Query;
use influxdb2_structmap::FromMap;

#[derive(Debug, influxdb2_structmap_derive::FromMap)]
pub struct StockPrice {
    ticker: String,
    value: f64,
    time: DateTime<FixedOffset>,
}

impl Default for StockPrice {
    fn default() -> Self {
        Self {
            ticker: "".to_string(),
            value: 0_f64,
            time: chrono::MIN_DATETIME.with_timezone(&chrono::FixedOffset::east(7 * 3600)),
        }
    }
}

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let host = std::env::var("INFLUXDB_HOST").unwrap();
    let org = std::env::var("INFLUXDB_ORG").unwrap();
    let token = std::env::var("INFLUXDB_TOKEN").unwrap();
    let client = Client::new(host, org, token);

    let qs = format!("from(bucket: \"stock-prices\") 
        |> range(start: -1w)
        |> filter(fn: (r) => r.ticker == \"{}\") 
        |> last()
    ", "AAPL");
    let query = Query::new(qs.to_string());
    let res: Vec<StockPrice> = client.query::<StockPrice>(Some(query))
        .await?;
    println!("{:?}", res);

    Ok(())
}
```

### Writing

```rust
async fn example() -> Result<(), Box<dyn std::error::Error>> {
    use futures::prelude::*;
    use influxdb2::models::DataPoint;
    use influxdb2::Client;

    let host = std::env::var("INFLUXDB_HOST").unwrap();
    let org = std::env::var("INFLUXDB_ORG").unwrap();
    let token = std::env::var("INFLUXDB_TOKEN").unwrap();
    let bucket = "bucket";
    let client = Client::new(host, org, token);
    
    let points = vec![
        DataPoint::builder("cpu")
            .tag("host", "server01")
            .field("usage", 0.5)
            .build()?,
        DataPoint::builder("cpu")
            .tag("host", "server01")
            .tag("region", "us-west")
            .field("usage", 0.87)
            .build()?,
    ];
                                                            
    client.write(bucket, stream::iter(points)).await?;
    
    Ok(())
}
```

### Delete API

```rust
let host = "some-host";
let org = "some-org";
let token = "some-token";
let client = Client::new(host, org, token);

let start = NaiveDate::from_ymd(2020, 1, 1).and_hms(0, 0, 0);
let stop = NaiveDate::from_ymd(2020, 12, 31).and_hms(23, 59, 59);
let predicate = Some("_measurement=\"some-measurement\"".to_owned());
client.delete(bucket, start, stop, predicate).await.unwrap();
```
