use serde::{Deserialize, Serialize};

use crate::{
    ids::{AgentSessionId, ProjectId, ReviewCommentId, TaskId, TerminalSessionId},
    task::{
        AgentKind, AgentStatusUpdate, ChangedFile, PreviewTarget, ProviderFailure, ReviewComment,
        TaskSource, Timestamp, WorktreeSnapshot,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskCreated {
    pub id: TaskId,
    pub project_id: ProjectId,
    pub title: String,
    pub source: TaskSource,
    pub occurred_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorktreeAttached {
    pub task_id: TaskId,
    pub snapshot: WorktreeSnapshot,
    pub occurred_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalStarted {
    pub task_id: TaskId,
    pub id: TerminalSessionId,
    pub occurred_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentStarted {
    pub task_id: TaskId,
    pub id: AgentSessionId,
    pub kind: AgentKind,
    pub occurred_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentStatusChanged {
    pub task_id: TaskId,
    pub update: AgentStatusUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangedFilesUpdated {
    pub task_id: TaskId,
    pub files: Vec<ChangedFile>,
    pub occurred_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewCommentAdded {
    pub task_id: TaskId,
    pub comment: ReviewComment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewDelivered {
    pub task_id: TaskId,
    pub comment_ids: Vec<ReviewCommentId>,
    pub occurred_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewAttached {
    pub task_id: TaskId,
    pub target: PreviewTarget,
    pub occurred_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskArchived {
    pub task_id: TaskId,
    pub occurred_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderFailed {
    pub task_id: TaskId,
    pub failure: ProviderFailure,
    pub occurred_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskEvent {
    TaskCreated(TaskCreated),
    WorktreeAttached(WorktreeAttached),
    TerminalStarted(TerminalStarted),
    AgentStarted(AgentStarted),
    AgentStatusChanged(AgentStatusChanged),
    ChangedFilesUpdated(ChangedFilesUpdated),
    ReviewCommentAdded(ReviewCommentAdded),
    ReviewDelivered(ReviewDelivered),
    PreviewAttached(PreviewAttached),
    TaskArchived(TaskArchived),
    ProviderFailed(ProviderFailed),
}

impl TaskEvent {
    pub fn task_id(&self) -> TaskId {
        match self {
            Self::TaskCreated(event) => event.id,
            Self::WorktreeAttached(event) => event.task_id,
            Self::TerminalStarted(event) => event.task_id,
            Self::AgentStarted(event) => event.task_id,
            Self::AgentStatusChanged(event) => event.task_id,
            Self::ChangedFilesUpdated(event) => event.task_id,
            Self::ReviewCommentAdded(event) => event.task_id,
            Self::ReviewDelivered(event) => event.task_id,
            Self::PreviewAttached(event) => event.task_id,
            Self::TaskArchived(event) => event.task_id,
            Self::ProviderFailed(event) => event.task_id,
        }
    }
}
