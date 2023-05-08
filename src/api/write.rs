//! Write API

use crate::models::WriteDataPoint;
use crate::{Client, Http, RequestError, ReqwestProcessing};
use bytes::BufMut;
use futures::{Stream, StreamExt};
use reqwest::{Body, Method, StatusCode};
use snafu::ResultExt;
use std::io::{self, Write};

impl Client {
    /// Write line protocol data to the specified organization and bucket.
    /// This method writes with default timestamp precision (nanoseconds).
    /// Use write_line_protocol_with_precision if you want to write with a different precision.
    pub async fn write_line_protocol(
        &self,
        org: &str,
        bucket: &str,
        body: impl Into<Body> + Send,
    ) -> Result<(), RequestError> {
        self.write_line_protocol_with_precision(org, bucket, body, TimestampPrecision::Nanoseconds)
            .await
    }

    /// Write line protocol data to the specified organization and bucket.
    pub async fn write_line_protocol_with_precision(
        &self,
        org: &str,
        bucket: &str,
        body: impl Into<Body> + Send,
        precision: TimestampPrecision,
    ) -> Result<(), RequestError> {
        let body = body.into();
        let write_url = self.url("/api/v2/write");

        let response = self
            .request(Method::POST, &write_url)
            .query(&[
                ("bucket", bucket),
                ("org", org),
                ("precision", precision.api_short_name()),
            ])
            .body(body)
            .send()
            .await
            .context(ReqwestProcessing)?;

        if response.status() != StatusCode::NO_CONTENT {
            let status = response.status();
            let text = response.text().await.context(ReqwestProcessing)?;
            Http { status, text }.fail()?;
        }

        Ok(())
    }

    /// Write a `Stream` of `DataPoint`s to the specified bucket.
    ///
    /// This method writes with default timestamp precision (nanoseconds).
    /// Use write_with_precision if you want to write with a different precision.
    pub async fn write(
        &self,
        bucket: &str,
        body: impl Stream<Item = impl WriteDataPoint> + Send + Sync + 'static,
    ) -> Result<(), RequestError> {
        self.write_with_precision(bucket, body, TimestampPrecision::Nanoseconds)
            .await
    }

    /// Write a `Stream` of `DataPoint`s to the specified organization and
    /// bucket.
    pub async fn write_with_precision(
        &self,
        bucket: &str,
        body: impl Stream<Item = impl WriteDataPoint> + Send + Sync + 'static,
        timestamp_precision: TimestampPrecision,
    ) -> Result<(), RequestError> {
        let mut buffer = bytes::BytesMut::new();

        let body = body.map(move |point| {
            let mut w = (&mut buffer).writer();
            point.write_data_point_to(&mut w)?;
            w.flush()?;
            Ok::<_, io::Error>(buffer.split().freeze())
        });

        let body = Body::wrap_stream(body);

        self
            .write_line_protocol_with_precision(&self.org, bucket, body, timestamp_precision)
            .await
    }
}

/// Possible timestamp precisions.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TimestampPrecision {
    /// Seconds timestamp precision
    Seconds,
    /// Milliseconds timestamp precision
    Milliseconds,
    /// Microseconds timestamp precision
    Microseconds,
    /// Nanoseconds timestamp precision
    Nanoseconds,
}

impl TimestampPrecision {
    fn api_short_name(&self) -> &str {
        match self {
            Self::Seconds => "s",
            Self::Milliseconds => "ms",
            Self::Microseconds => "us",
            Self::Nanoseconds => "ns",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DataPoint;
    use futures::stream;
    use mockito::mock;

    #[tokio::test]
    async fn writing_points() {
        let org = "some-org";
        let bucket = "some-bucket";
        let token = "some-token";

        let mock_server = mock(
            "POST",
            format!("/api/v2/write?bucket={}&org={}&precision=ns", bucket, org).as_str(),
        )
        .match_header("Authorization", format!("Token {}", token).as_str())
        .match_body(
            "\
cpu,host=server01 usage=0.5
cpu,host=server01,region=us-west usage=0.87
",
        )
        .with_status(204)
        .create();

        let client = Client::new(mockito::server_url(), org, token);

        let points = vec![
            DataPoint::builder("cpu")
                .tag("host", "server01")
                .field("usage", 0.5)
                .build()
                .unwrap(),
            DataPoint::builder("cpu")
                .tag("host", "server01")
                .tag("region", "us-west")
                .field("usage", 0.87)
                .build()
                .unwrap(),
        ];

        // If the requests made are incorrect, Mockito returns status 501 and `write`
        // will return an error, which causes the test to fail here instead of
        // when we assert on mock_server. The error messages that Mockito
        // provides are much clearer for explaining why a test failed than just
        // that the server returned 501, so don't use `?` here.
        let result = client.write(bucket, stream::iter(points)).await;
        mock_server.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn writing_points_with_precision() {
        let org = "some-org";
        let bucket = "some-bucket";
        let token = "some-token";

        let mock_server = mock(
            "POST",
            format!("/api/v2/write?bucket={}&org={}&precision=s", bucket, org).as_str(),
        )
        .match_header("Authorization", format!("Token {}", token).as_str())
        .match_body(
            "\
cpu,host=server01 usage=0.5 1671095854
",
        )
        .with_status(204)
        .create();

        let client = Client::new(mockito::server_url(), org, token);

        let point = DataPoint::builder("cpu")
            .tag("host", "server01")
            .field("usage", 0.5)
            .timestamp(1671095854)
            .build()
            .unwrap();
        let points = vec![point];

        // If the requests made are incorrect, Mockito returns status 501 and `write`
        // will return an error, which causes the test to fail here instead of
        // when we assert on mock_server. The error messages that Mockito
        // provides are much clearer for explaining why a test failed than just
        // that the server returned 501, so don't use `?` here.
        let result = client
            .write_with_precision(bucket, stream::iter(points), TimestampPrecision::Seconds)
            .await;
        mock_server.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn status_code_correctly_interpreted() {
        let org = "org";
        let token = "token";
        let bucket = "bucket";

        let make_mock_server = |status| {
            mock(
                "POST",
                format!("/api/v2/write?bucket={}&org={}&precision=ns", bucket, org).as_str(),
            )
            .with_status(status)
            .create()
        };

        let write_with_status = |status| async move {
            let mock_server = make_mock_server(status);
            let client = Client::new(mockito::server_url(), org, token);
            let points: Vec<DataPoint> = vec![];
            let res = client.write(bucket, stream::iter(points)).await;
            mock_server.assert();
            res
        };

        // success status
        assert!(write_with_status(204).await.is_ok());

        // failing status
        for status in [200, 201, 400, 401, 404, 413, 429, 500, 503] {
            assert!(write_with_status(status).await.is_err());
        }
    }
}
