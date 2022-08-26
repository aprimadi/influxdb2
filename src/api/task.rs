//! Tasks API

use reqwest::Method;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{Client, Http, RequestError, ReqwestProcessing, Serializing};
use crate::models::{Tasks, TaskStatusType};

impl Client {
    /// List all tasks.
    pub async fn list_tasks(
        &self,
        request: ListTasksRequest,
    ) -> Result<Tasks, RequestError> {
        let qs = serde_qs::to_string(&request).unwrap();
        let url = match &qs[..] {
            "" => format!("{}/api/v2/tasks", self.url),
            _  => format!("{}/api/v2/tasks?{}", self.url, qs),
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
            .json::<Tasks>()
            .await
            .context(ReqwestProcessing)?;
        Ok(res)
    }

    /// Create a new task.
    pub async fn create_task(
        &self,
        request: CreateTaskRequest,
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

/// Request for list tasks api
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListTasksRequest {
    /// Return tasks after a specified task ID.
    pub after: Option<String>,
    /// The number of tasks to return. Default: 100. Valid values [1..500].
    pub limit: Option<u16>,
    /// Filter tasks to a specified name.
    pub name: Option<String>,
    /// Filter tasks to a specific organization name.
    pub org: Option<String>,
    /// Filter tasks to a specific organization ID.
    #[serde(rename = "orgID")]
    pub org_id: Option<String>,
    /// Filter tasks by status, either "inactive" or "active".
    pub status: Option<String>,
    /// Filter task by type. Default: "". Valid values: ["basic", "system"].
    #[serde(rename = "type")]
    pub type_: Option<TaskStatusType>,
    /// Filter tasks to a specific user ID.
    pub user: Option<String>,
}

/// Encapsulates task data that is sent on POST via the task API.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    /// The flux script to run this task
    pub flux: String,
    /// The ID of the organization that owns this task
    #[serde(rename = "orgID")]
    pub org_id: String,
    /// An optional description of the task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The name of the organization that owns this task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org: Option<String>,
    /// Task status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatusType>,
}

impl CreateTaskRequest {
    /// Returns instance of PostTaskRequest
    pub fn new(org_id: String, flux: String) -> Self {
        Self {
            flux,
            description: None,
            org: None,
            org_id,
            status: None,
        }
    }
}

