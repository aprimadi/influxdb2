//! Task

use serde::{Deserialize, Serialize};

/// Task status
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TaskStatusType {
    /// Active task status
    #[serde(rename = "active")]
    Active,
    /// Inactive task status
    #[serde(rename = "inactive")]
    Inactive,
}

/// Task schema
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    /// Task ID
    pub id: String,
    /// Task name
    pub name: String,
    /// The ID of the organization that owns this task
    #[serde(rename = "orgID")]
    pub org_id: String,
    /// The FLUX script to run this task
    pub flux: String,
    /// The ID of the user who owns this task
    #[serde(rename = "ownerID")]
    pub owner_id: Option<String>,
    /// The name of the organization that owns this task
    pub org: Option<String>,
    /// Task status
    pub status: Option<TaskStatusType>,
    /// The type of task, this can be used for filtering tasks on list actions.
    #[serde(rename = "type")]
    pub type_: Option<String>,
    /// The ID of the authorization used when this task communicates with the
    /// query engine
    #[serde(rename = "authorizationID")]
    pub authorization_id: Option<String>,
    /// An optional description of the task
    pub description: Option<String>,
    /// A task repetition schedule in the form '* * * * * *', parsed from Flux
    pub cron: Option<String>,
    /// A simple task repetition schedule, parsed from Flux
    pub every: Option<String>,
    /// Task error on last run
    pub last_run_error: Option<String>,
    /// Status of task on last run
    pub last_run_status: Option<String>,
    /// Timestamp of latest scheduled, completed run, RFC3339
    pub latest_completed: Option<String>,
    /// Duration to delay after the schedule, before executing the task;
    /// parsed from flux
    pub offset: Option<String>,
    /// Links
    pub links: Option<TaskLinks>,
    /// Task Labels
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<crate::models::Label>,
    /// Task created timestamp
    pub created_at: Option<String>,
    /// Task updated timestamp
    pub updated_at: Option<String>,
}

/// Task Links
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TaskLinks {
    /// Link to self
    #[serde(rename = "self", skip_serializing_if = "Option::is_none")]
    pub self_: Option<String>,
    /// Link to labels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<String>,
    /// Link to logs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<String>,
    /// Link to members
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<String>,
    /// Link to owners
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owners: Option<String>,
    /// Link to runs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runs: Option<String>,
}

/// Tasks
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Tasks {
    /// Links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<crate::models::Links>,
    /// List of tasks
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<Task>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_task_status_type() {
        let v = serde_json::to_string(&TaskStatusType::Active).unwrap();
        assert_eq!(v, "\"active\"");
        let v = serde_json::to_string(&TaskStatusType::Inactive).unwrap();
        assert_eq!(v, "\"inactive\"");
    }
}
