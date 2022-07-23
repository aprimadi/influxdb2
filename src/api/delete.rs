//! Delete API

use chrono::NaiveDateTime;
use reqwest::Method;
use snafu::ResultExt;

use crate::{Client, Http, RequestError, ReqwestProcessing};

impl Client {
    /// Delete data points from a bucket matching specified parameters.
    pub async fn delete(
        &self,
        bucket: &str,
        start: NaiveDateTime,
        stop: NaiveDateTime,
        predicate: Option<String>,
    ) -> Result<(), RequestError> {
        let delete_url = format!("{}/api/v2/delete", self.url);
        
        let body = serde_json::json!({
            "start": start.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "stop": stop.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "predicate": predicate,
        }).to_string();
        
        let response = self
            .request(Method::POST, &delete_url)
            .query(&[("bucket", bucket), ("org", &self.org)])
            .body(body)
            .send()
            .await
            .context(ReqwestProcessing)?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.context(ReqwestProcessing)?;
            Http { status, text }.fail()?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use mockito::mock;
    
    #[tokio::test]
    async fn delete_points() {
        let org = "some-org";
        let bucket = "some-bucket";
        let token = "some-token";
        
        let mock_server = mock(
                "POST",
                format!("/api/v2/delete?bucket={}&org={}", bucket, org).as_str(),
            )
            .match_header("Authorization", format!("Token {}", token).as_str())
            .match_body(
                "{\"predicate\":null,\"start\":\"2020-01-01T00:00:00Z\",\"stop\":\"2021-01-01T00:00:00Z\"}"
            )
            .create();
        
        let client = Client::new(&mockito::server_url(), org, token);
        
        let start = NaiveDate::from_ymd(2020, 1, 1).and_hms(0, 0, 0);
        let stop = NaiveDate::from_ymd(2021, 1, 1).and_hms(0, 0, 0);
        let _result = client.delete(bucket, start, stop, None).await;
        
        mock_server.assert();
    }
}