#![recursion_limit = "1024"]
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

use reqwest::{Method, Url};
use secrecy::{ExposeSecret, Secret};
use snafu::{ResultExt, Snafu};

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

#[cfg(feature = "gzip")]
#[derive(Debug, Clone)]
enum Compression {
    None,
    Gzip,
}

/// Client to a server supporting the InfluxData 2.0 API.
#[derive(Debug, Clone)]
pub struct Client {
    /// The base URL this client sends requests to
    pub base: Url,
    /// The organization tied to this client
    pub org: String,
    auth_header: Option<Secret<String>>,
    reqwest: reqwest::Client,
    #[cfg(feature = "gzip")]
    compression: Compression,
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
        auth_token: impl Into<String>,
    ) -> Self {
        // unwrap is used to maintain backwards compatibility
        // panicking was present earlier as well inside of reqwest
        ClientBuilder::new(url, org, auth_token).build().unwrap()
    }

    /// Consolidate common request building code
    fn request(&self, method: Method, url: &str) -> reqwest::RequestBuilder {
        let mut req = self.reqwest.request(method, url);

        if let Some(auth) = &self.auth_header {
            req = req.header("Authorization", auth.expose_secret());
        }

        req
    }

    /// Join base Url of the client to target API endpoint into valid Url
    fn url(&self, endpoint: &str) -> String {
        let mut url = self.base.clone();
        url.set_path(endpoint);
        url.into()
    }
}

/// Errors that occur when building the client
#[derive(Debug, Snafu)]
pub enum BuildError {
    /// While constructing the reqwest client an error occurred
    #[snafu(display("Error while building the client: {}", source))]
    ReqwestClientError {
        /// Reqwest internal error
        source: reqwest::Error,
    },
}
/// ClientBuilder builds the `Client`
#[derive(Debug)]
pub struct ClientBuilder {
    /// The base URL this client sends requests to
    pub base: Url,
    /// The organization tied to this client
    pub org: String,
    auth_header: Option<Secret<String>>,
    reqwest: reqwest::ClientBuilder,
    #[cfg(feature = "gzip")]
    compression: Compression,
}

impl ClientBuilder {
    /// Construct a new `ClientBuilder`.
    pub fn new(
        url: impl Into<String>,
        org: impl Into<String>,
        auth_token: impl Into<String>,
    ) -> Self {
        Self::with_builder(reqwest::ClientBuilder::new(), url, org, auth_token)
    }

    /// Construct a new `ClientBuilder` with a provided [`reqwest::ClientBuilder`].
    ///
    /// Can be used to pass custom `reqwest` parameters, such as TLS configuration.
    pub fn with_builder(
        builder: reqwest::ClientBuilder,
        url: impl Into<String>,
        org: impl Into<String>,
        auth_token: impl Into<String>,
    ) -> Self {
        let token = auth_token.into();
        let auth_header = if token.is_empty() {
            None
        } else {
            Some(format!("Token {}", token).into())
        };

        let url: String = url.into();
        let base = Url::parse(&url).expect(&format!("Invalid url was provided: {}", &url));

        Self {
            base,
            org: org.into(),
            auth_header,
            reqwest: builder,
            #[cfg(feature = "gzip")]
            compression: Compression::None,
        }
    }

    /// Enable gzip compression on the write and write_with_precision calls
    #[cfg(feature = "gzip")]
    pub fn gzip(mut self, enable: bool) -> ClientBuilder {
        self.reqwest = self.reqwest.gzip(enable);
        self.compression = Compression::Gzip;
        self
    }

    /// Build returns the influx client
    pub fn build(self) -> Result<Client, BuildError> {
        Ok(Client {
            base: self.base,
            org: self.org,
            auth_header: self.auth_header,
            reqwest: self.reqwest.build().context(ReqwestClientError)?,
            #[cfg(feature = "gzip")]
            compression: self.compression,
        })
    }
}

pub mod common;

pub mod api;
pub mod models;
pub mod writable;

// Re-exports
pub use influxdb2_derive::FromDataPoint;
pub use influxdb2_structmap::FromMap;

#[cfg(test)]
mod tests {
    use crate::Client;

    #[test]
    fn url_invalid_panic() {
        let result = std::panic::catch_unwind(|| Client::new("/3242/23", "some-org", "some-token"));
        assert!(result.is_err());
    }

    #[test]
    /// Reproduction of https://github.com/aprimadi/influxdb2/issues/6
    fn url_ignores_double_slashes() {
        let base = "http://influxdb.com/";
        let client = Client::new(base, "some-org", "some-token");

        assert_eq!(format!("{}api/v2/write", base), client.url("/api/v2/write"));

        assert_eq!(client.url("/api/v2/write"), client.url("api/v2/write"));
    }
}
