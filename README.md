# influxdb2

This is a Rust client to InfluxDB using the [2.0 API][2api].

[2api]: https://v2.docs.influxdata.com/v2.0/reference/api/

## Setup

Add this to `cargo.toml`:

```toml
influxdb2 = "0.3"
influxdb2-structmap = "0.2"
influxdb2-derive = "0.1.1"
num-traits = "0.2"
```

## Usage

### Querying

```rust
use chrono::{DateTime, FixedOffset};
use influxdb2::{Client, FromDataPoint};
use influxdb2::models::Query;

#[derive(Debug, FromDataPoint)]
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
#[derive(Default, WriteDataPoint)]
#[measurement = "cpu_load_short"]
struct CpuLoadShort {
    #[influxdb(tag)]
    host: Option<String>,
    #[influxdb(tag)]
    region: Option<String>,
    #[influxdb(field)]
    value: f64,
    #[influxdb(timestamp)]
    time: i64,
}

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    use chrono::Utc;
    use futures::prelude::*;
    use influxdb2::models::DataPoint;
    use influxdb2::Client;
    use influxdb2_derive::WriteDataPoint;

    let host = std::env::var("INFLUXDB_HOST").unwrap();
    let org = std::env::var("INFLUXDB_ORG").unwrap();
    let token = std::env::var("INFLUXDB_TOKEN").unwrap();
    let bucket = std::env::var("INFLUXDB_BUCKET").unwrap();
    let client = Client::new(host, org, token);
    
    let points = vec![
        CpuLoadShort {
            host: Some("server01".to_owned()),
            region: Some("us-west".to_owned()),
            value: 0.64,
            time: Utc::now().timestamp_nanos(),
        },
        CpuLoadShort {
            host: Some("server02".to_owned()),
            region: None,
            value: 0.64,
            time: Utc::now().timestamp_nanos(),
        },
    ];

    client.write(bucket, stream::iter(points)).await?;
    
    Ok(())
}
```

## Supported Data Types

InfluxDB data point doesn't support every data types supported by Rust. So,
the derive macro only allows for a subset of data types which is also 
supported in InfluxDB. 

Supported struct field types:

- bool
- f64
- i64
- u64 - DEPRECATED, will be removed in version 0.4
- String
- Vec<u8>
- chrono::Duration
- DateTime<FixedOffset>

## Features

Implemented API

- [x] Query API
- [x] Write API
- [x] Delete API
- [ ] Bucket API (partial: only list, create, delete)
- [ ] Organization API (partial: only list)
- [ ] Task API (partial: only list, create, delete)

## TLS Implementations
This crate uses [reqwest](https://github.com/seanmonstar/reqwest) under the 
hood. You can choose between `native-tls` and `rustls` with the features 
provided with this crate. `native-tls` is chosen as the default, like reqwest 
does.

```toml
# Usage for native-tls (the default).
influxdb2 = "0.3"

# Usage for rustls.
influxdb2 = { version = "0.3", features = ["rustls"], default-features = false }
```

## Development Status

This project is still at alpha status and all the bugs haven't been ironed 
yet. With that said, use it at your own risk and feel free to create an issue 
or pull request.

