//! Organization API

use reqwest::Method;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::models::Organizations;
use crate::{Client, Http, RequestError, ReqwestProcessing};

impl Client {
    /// List all organizations.
    pub async fn list_organizations(
        &self,
        request: ListOrganizationRequest,
    ) -> Result<Organizations, RequestError> {
        let url = self.url("/api/v2/orgs");

        let response = self
            .request(Method::GET, &url)
            .query(&request)
            .send()
            .await
            .context(ReqwestProcessing)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.context(ReqwestProcessing)?;
            let res = Http { status, text }.fail();
            return res;
        }

        let res = response
            .json::<Organizations>()
            .await
            .context(ReqwestProcessing)?;
        Ok(res)
    }
}

/// Request for list organization API
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOrganizationRequest {
    /// Whether to return results in descending order.
    pub descending: Option<bool>,
    /// Number of organizations to return.
    pub limit: Option<i64>,
    /// Offset of organization to return from.
    pub offset: Option<i64>,
    /// Filter by organization name.
    pub org: Option<String>,
    /// Filter by organization ID.
    #[serde(rename = "orgID")]
    pub org_id: Option<String>,
    /// Filter by specific user ID.
    #[serde(rename = "userID")]
    pub user_id: Option<String>,
}

impl ListOrganizationRequest {
    /// Create a new request for list organization API
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;

    #[tokio::test]
    async fn list_buckets() {
        let org_id = "0000111100001111".to_string();
        let token = "some-token";

        let mock_server = mock("GET", "/api/v2/orgs?limit=1&orgID=some-org")
            .match_header("Authorization", format!("Token {}", token).as_str())
            .create();

        let client = Client::new(mockito::server_url(), &org_id, token);

        let request = ListOrganizationRequest {
            limit: Some(1),
            org_id: Some("some-org".to_string()),
            ..ListOrganizationRequest::default()
        };

        let _result = client.list_organizations(request).await;

        mock_server.assert();
    }
}
