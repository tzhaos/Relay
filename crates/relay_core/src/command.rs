use serde::{Deserialize, Serialize};

use crate::{
    ids::{AgentSessionId, ProjectId, ReviewCommentId, TaskId, TerminalSessionId},
    task::{
        AgentKind, AgentStatusUpdate, ChangedFile, PreviewTarget, ProviderFailure, ReviewComment,
        TaskSource, Timestamp, WorktreeSnapshot,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateTask {
    pub id: Option<TaskId>,
    pub project_id: ProjectId,
    pub title: String,
    pub source: TaskSource,
    pub now: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskCommand {
    CreateTask(CreateTask),
    AttachWorktree {
        snapshot: WorktreeSnapshot,
        now: Timestamp,
    },
    AttachTerminal {
        id: TerminalSessionId,
        now: Timestamp,
    },
    AttachAgent {
        id: AgentSessionId,
        kind: AgentKind,
        started_at: Timestamp,
    },
    ApplyAgentStatus(AgentStatusUpdate),
    RefreshChangedFiles {
        files: Vec<ChangedFile>,
        now: Timestamp,
    },
    AddReviewComment(ReviewComment),
    MarkReviewDelivered {
        comment_ids: Vec<ReviewCommentId>,
        now: Timestamp,
    },
    AttachPreview {
        target: PreviewTarget,
        now: Timestamp,
    },
    Archive {
        now: Timestamp,
    },
    MarkFailed {
        failure: ProviderFailure,
        now: Timestamp,
    },
}
