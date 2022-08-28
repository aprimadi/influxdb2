#![deny(rustdoc::broken_intra_doc_links, rust_2018_idioms)]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    clippy::explicit_iter_loop,
    clippy::use_self,
    clippy::clone_on_ref_ptr,
    clippy::future_not_send
)]

//! # influxdb2
//! 
//! This is a Rust client to InfluxDB using the [2.0 API][2api].
//! 
//! [2api]: https://v2.docs.influxdata.com/v2.0/reference/api/
//! 
//! This project is a fork from the 
//! https://github.com/influxdata/influxdb_iox/tree/main/influxdb2_client project.
//! At the time of this writing, the query functionality of the influxdb2 client 
//! from the official repository isn't working. So, I created this client to use 
//! it in my project.
//! 
//! ## Usage
//! 
//! ### Querying
//! 
//! ```rust
//! use chrono::{DateTime, FixedOffset};
//! use influxdb2::{Client, FromDataPoint};
//! use influxdb2::models::Query;
//! 
//! #[derive(Debug, FromDataPoint)]
//! pub struct StockPrice {
//!     ticker: String,
//!     value: f64,
//!     time: DateTime<FixedOffset>,
//! }
//!
//! impl Default for StockPrice {
//!     fn default() -> Self {
//!         Self {
//!             ticker: "".to_string(),
//!             value: 0_f64,
//!             time: chrono::MIN_DATETIME.with_timezone(&chrono::FixedOffset::east(7 * 3600)),
//!         }
//!     }
//! }
//! 
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let host = std::env::var("INFLUXDB_HOST").unwrap();
//!     let org = std::env::var("INFLUXDB_ORG").unwrap();
//!     let token = std::env::var("INFLUXDB_TOKEN").unwrap();
//!     let client = Client::new(host, org, token);
//! 
//!     let qs = format!("from(bucket: \"stock-prices\") 
//!         |> range(start: -1w)
//!         |> filter(fn: (r) => r.ticker == \"{}\") 
//!         |> last()
//!     ", "AAPL");
//!     let query = Query::new(qs.to_string());
//!     let res: Vec<StockPrice> = client.query::<StockPrice>(Some(query))
//!         .await?;
//!     println!("{:?}", res);
//!
//!     Ok(())
//! }
//! ```
//! 
//! ### Writing
//! 
//! ```rust
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     use futures::prelude::*;
//!     use influxdb2::models::DataPoint;
//!     use influxdb2::Client;
//!
//!     let host = std::env::var("INFLUXDB_HOST").unwrap();
//!     let org = std::env::var("INFLUXDB_ORG").unwrap();
//!     let token = std::env::var("INFLUXDB_TOKEN").unwrap();
//!     let bucket = "bucket";
//!     let client = Client::new(host, org, token);
//!     
//!     let points = vec![
//!         DataPoint::builder("cpu")
//!             .tag("host", "server01")
//!             .field("usage", 0.5)
//!             .build()?,
//!         DataPoint::builder("cpu")
//!             .tag("host", "server01")
//!             .tag("region", "us-west")
//!             .field("usage", 0.87)
//!             .build()?,
//!     ];
//!                                                             
//!     client.write(bucket, stream::iter(points)).await?;
//!     
//!     Ok(())
//! }
//! ```


use reqwest::Method;
use snafu::Snafu;

/// Errors that occur while making requests to the Influx server.
#[derive(Debug, Snafu)]
pub enum RequestError {
    /// While making a request to the Influx server, the underlying `reqwest`
    /// library returned an error that was not an HTTP 400 or 500.
    #[snafu(display("Error while processing the HTTP request: {}", source))]
    ReqwestProcessing {
        /// The underlying error object from `reqwest`.
        source: reqwest::Error,
    },
    /// The underlying `reqwest` library returned an HTTP error with code 400
    /// (meaning a client error) or 500 (meaning a server error).
    #[snafu(display("HTTP request returned an error: {}, `{}`", status, text))]
    Http {
        /// The `StatusCode` returned from the request
        status: reqwest::StatusCode,
        /// Any text data returned from the request
        text: String,
    },

    /// While serializing data as JSON to send in a request, the underlying
    /// `serde_json` library returned an error.
    #[snafu(display("Error while serializing to JSON: {}", source))]
    Serializing {
        /// The underlying error object from `serde_json`.
        source: serde_json::error::Error,
    },

    /// While deserializing response from the Influx server, the underlying
    /// parsing library returned an error.
    #[snafu(display("Error while parsing response: {}", text))]
    Deserializing {
        /// Error description.
        text: String,
    },
}

/// Client to a server supporting the InfluxData 2.0 API.
#[derive(Debug, Clone)]
pub struct Client {
    /// The base URL this client sends requests to
    pub url: String,
    /// The organization tied to this client
    pub org: String,
    auth_header: Option<String>,
    reqwest: reqwest::Client,
}

impl Client {
    /// Create a new client pointing to the URL specified in
    /// `protocol://server:port` format and using the specified token for
    /// authorization.
    ///
    /// # Example
    ///
    /// ```
    /// let client = influxdb2::Client::new("http://localhost:8888", "org", "my-token");
    /// ```
    pub fn new(
        url: impl Into<String>, 
        org: impl Into<String>, 
        auth_token: impl Into<String>
    ) -> Self {
        let token = auth_token.into();
        let auth_header = if token.is_empty() {
            None
        } else {
            Some(format!("Token {}", token))
        };

        Self {
            url: url.into(),
            org: org.into(),
            auth_header,
            reqwest: reqwest::Client::new(),
        }
    }

    /// Consolidate common request building code
    fn request(&self, method: Method, url: &str) -> reqwest::RequestBuilder {
        let mut req = self.reqwest.request(method, url);

        if let Some(auth) = &self.auth_header {
            req = req.header("Authorization", auth);
        }

        req
    }
}

pub mod common;

pub mod api;
pub mod models;

// Re-exports
pub use influxdb2_structmap::FromMap;
pub use influxdb2_derive::FromDataPoint;
