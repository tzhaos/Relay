use serde::{Deserialize, Serialize};

use crate::{
    ids::TaskId,
    task::{AgentKind, Task, TaskStatus, Timestamp},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusTone {
    Neutral,
    Accent,
    Warning,
    Danger,
    Muted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskProjection {
    pub id: TaskId,
    pub title: String,
    pub status: TaskStatus,
    pub status_label: String,
    pub status_tone: StatusTone,
    pub agent: Option<AgentKind>,
    pub meta: String,
    pub has_terminal: bool,
    pub changed_file_count: usize,
    pub review_comment_count: usize,
    pub pending_review_comment_count: usize,
    pub preview_target_count: usize,
    pub failure_summary: Option<String>,
    pub last_activity_at: Timestamp,
}

impl TaskProjection {
    pub fn from_task(task: &Task) -> Self {
        let delivered = task.diff_review.delivered_comments.len();
        let comment_count = task.diff_review.comments.len();
        let meta = task
            .worktree
            .as_ref()
            .map(|worktree| format!("{} / {}", task.project_id, worktree.branch))
            .unwrap_or_else(|| format!("{} / no worktree", task.project_id));

        Self {
            id: task.id,
            title: task.title.clone(),
            status: task.status,
            status_label: task.status.label().to_string(),
            status_tone: status_tone(task.status),
            agent: task.agent_kind.clone(),
            meta,
            has_terminal: task.terminal_session_id.is_some(),
            changed_file_count: task.changed_files.len(),
            review_comment_count: comment_count,
            pending_review_comment_count: comment_count.saturating_sub(delivered),
            preview_target_count: task.preview_targets.len(),
            failure_summary: task.failure.as_ref().map(|failure| failure.message.clone()),
            last_activity_at: task.last_activity_at,
        }
    }
}

fn status_tone(status: TaskStatus) -> StatusTone {
    match status {
        TaskStatus::Working | TaskStatus::StartingAgent => StatusTone::Accent,
        TaskStatus::CreatingWorktree | TaskStatus::WaitingForUser | TaskStatus::Reviewing => {
            StatusTone::Warning
        }
        TaskStatus::Blocked | TaskStatus::Failed => StatusTone::Danger,
        TaskStatus::Done | TaskStatus::Archived => StatusTone::Muted,
        TaskStatus::Draft | TaskStatus::ReadyToCommit => StatusTone::Neutral,
    }
}
