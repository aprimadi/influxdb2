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

#[derive(structmap_derive::FromMap)]
pub struct StockPrice {
    ticker: String,
    value: f64,
    time: DateTime<FixedOffset>,
}

#[tokio::main]
async fn main() {
    let host = std::env::var("INFLUXDB_HOST").unwrap();
    let org = std::env::var("INFLUXDB_ORG").unwrap();
    let token = std::env::var("INFLUXDB_TOKEN").unwrap();
    let client = Client::new(host, org, token);

    let qs = format!("from(bucket: \"stock-prices\") 
        |> range(start: -1w)
        |> filter(fn: (r) => r.ticker == \"{}\") 
        |> last()
    ", ticker);
    let query = Query::new(qs.to_string());
    let res: Vec<StockPrice> = self.client.query::<StockPrice>(&self.org, Some(query))
        .await
        .unwrap();
    println!("{}", res);
}
```

### Writing

```rust
#[tokio::main]
async fn main() {
    let host = std::env::var("INFLUXDB_HOST").unwrap();
    let org = std::env::var("INFLUXDB_ORG").unwrap();
    let token = std::env::var("INFLUXDB_TOKEN").unwrap();
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
}
```

