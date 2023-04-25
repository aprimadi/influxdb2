use influxdb2::models::{LanguageRequest, Query};
use influxdb2::FromDataPoint;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let influx_url = "http://localhost:8086";
    let token = "some-token";

    let client = influxdb2::Client::new(influx_url, "org", token);

    client.query_suggestions().await?;
    client.query_suggestions_name("some-name").await?;

    #[derive(FromDataPoint)]
    struct Measurement {
        value: f64,
    }
    impl Default for Measurement {
        fn default() -> Self {
            Self { value: 0f64 }
        }
    }

    client
        .query::<Measurement>(Some(Query::new("some-query".to_string())))
        .await?;

    client
        .query_analyze(Some(Query::new("some-query".to_string())))
        .await?;

    client
        .query_ast(Some(LanguageRequest::new("some-query".to_string())))
        .await?;

    Ok(())
}
