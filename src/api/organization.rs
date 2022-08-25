//! Organization API

use reqwest::Method;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{Client, Http, RequestError, ReqwestProcessing};
use crate::models::Organizations;

impl Client {
    /// List all organizations.
    pub async fn list_organizations(
        &self,
        request: ListOrganizationRequest,
    ) -> Result<Organizations, RequestError> {
        let qs = serde_qs::to_string(&request).unwrap();
        let url = match &qs[..] {
            "" => format!("{}/api/v2/orgs", self.url),
            _  => format!("{}/api/v2/orgs?{}", self.url, qs),
        };
        
        let response = self
            .request(Method::GET, &url)
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
    
    #[test]
    fn serialize_list_organization_request() {
        let req = ListOrganizationRequest::new();
        let qs = serde_qs::to_string(&req).unwrap();
        assert_eq!(qs, "");
        
        let mut req = ListOrganizationRequest::new();
        req.org = Some("Sahamee".to_owned());
        let qs = serde_qs::to_string(&req).unwrap();
        assert_eq!(qs, "org=Sahamee");
    }
}
