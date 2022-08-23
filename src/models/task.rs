//! Task

use serde::{Deserialize, Serialize};

/// Task status
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TaskStatusType {
    /// Active task status
    Active,
    /// Inactive task status
    Inactive,
}

impl ToString for TaskStatusType {
    fn to_string(&self) -> String {
        match &self {
            TaskStatusType::Active => "active".to_owned(),
            TaskStatusType::Inactive => "inactive".to_owned(),
        }
    }
}

/// Encapsulates task data that is sent on POST via the task API.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PostTaskRequest {
    flux: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    org: Option<String>,
    #[serde(rename = "orgID")]
    org_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<TaskStatusType>,
}

impl PostTaskRequest {
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

