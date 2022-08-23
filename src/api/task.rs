//! Tasks API

use reqwest::Method;
use snafu::ResultExt;

use crate::{Client, Http, RequestError, ReqwestProcessing, Serializing};
use crate::models::PostTaskRequest;

impl Client {
    /// Create a new task in the organization specified by the 16-digit 
    /// hexadecimal `org_id`.
    pub async fn create_task(
        &self,
        request: PostTaskRequest,
    ) -> Result<(), RequestError> {
        let url = format!("{}/api/v2/tasks", self.url);
        let response = self
            .request(Method::POST, &url)
            .body(
                serde_json::to_string(&request)
                    .context(Serializing)?,
            )
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

