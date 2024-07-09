//! Authorizations (tokens) API.

use reqwest::Method;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::models::authorization::{Authorization, Authorizations, Status};
use crate::models::permission::Permission;
use crate::{Client, Http, RequestError, ReqwestProcessing, Serializing};

impl Client {
    /// List all authorization matching specified parameters
    pub async fn list_authorizations(
        &self,
        request: ListAuthorizationsRequest,
    ) -> Result<Authorizations, RequestError> {
        let url = self.url("/api/v2/authorizations");

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

        response.json().await.context(ReqwestProcessing)
    }

    /// Create a new authorization in the organization.
    pub async fn create_authorization(
        &self,
        request: CreateAuthorizationRequest,
    ) -> Result<Authorization, RequestError> {
        let create_bucket_url = self.url("/api/v2/authorizations");

        let response = self
            .request(Method::POST, &create_bucket_url)
            .body(serde_json::to_string(&request).context(Serializing)?)
            .send()
            .await
            .context(ReqwestProcessing)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.context(ReqwestProcessing)?;
            let res = Http { status, text }.fail();
            return res;
        }

        response
            .json::<Authorization>()
            .await
            .context(ReqwestProcessing)
    }
}

/// Request for listing authorizations.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListAuthorizationsRequest {
    /// Only returns authorizations that belong to the specified organization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org: Option<String>,
    /// Only returns authorizations that belong to the specified organization ID.
    #[serde(rename = "orgID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    /// Only returns the authorization that match the token value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Only returns the authorization scoped to the specified user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Only returns the authorization scoped to the specified user ID.
    #[serde(rename = "userID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

/// Request for creating an authorization.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAuthorizationRequest {
    /// A description of the token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// An organization ID. Specifies the organization that owns the authorization.
    #[serde(rename = "orgID")]
    pub org_id: String,
    /// A list of permissions for an authorization (at least 1 required).
    pub permissions: Vec<Permission>,
    /// Status of the token after creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    /// A user ID. Specifies the user that the authorization is scoped to.
    #[serde(rename = "userID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}
