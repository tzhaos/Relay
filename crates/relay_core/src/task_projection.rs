use serde::{Deserialize, Serialize};

use crate::{
    ids::{PreviewTargetId, ReviewCommentId, TaskId, TerminalSessionId},
    task::{
        AgentKind, ChangedFile, DiffSide, PreviewTarget, ReviewComment, Task, TaskStatus, Timestamp,
    },
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
    pub agent_prompt: String,
    pub meta: String,
    pub has_terminal: bool,
    pub terminal_session_id: Option<TerminalSessionId>,
    pub worktree_path: Option<String>,
    #[serde(default)]
    pub worktree_branch: Option<String>,
    pub changed_files: Vec<ChangedFile>,
    pub changed_file_count: usize,
    pub review_comments: Vec<ReviewCommentProjection>,
    pub review_comment_count: usize,
    pub pending_review_comment_count: usize,
    pub preview_targets: Vec<PreviewTargetProjection>,
    pub preview_target_count: usize,
    pub failure_summary: Option<String>,
    pub last_activity_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewCommentProjection {
    pub id: ReviewCommentId,
    pub path: String,
    pub line_label: String,
    pub body: String,
    pub delivered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewTargetProjection {
    pub id: PreviewTargetId,
    pub label: String,
    pub uri: String,
}

impl TaskProjection {
    pub fn from_task(task: &Task) -> Self {
        let delivered_comment_ids = task
            .diff_review
            .delivered_comments
            .iter()
            .map(|entry| entry.id)
            .collect::<std::collections::HashSet<_>>();
        let delivered = delivered_comment_ids.len();
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
            agent_prompt: task.agent_prompt.clone(),
            meta,
            has_terminal: task.terminal_session_id.is_some(),
            terminal_session_id: task.terminal_session_id,
            worktree_path: task.worktree.as_ref().map(|worktree| worktree.path.clone()),
            worktree_branch: task
                .worktree
                .as_ref()
                .map(|worktree| worktree.branch.clone()),
            changed_files: task.changed_files.clone(),
            changed_file_count: task.changed_files.len(),
            review_comments: task
                .diff_review
                .comments
                .iter()
                .map(|comment| {
                    ReviewCommentProjection::from_comment(
                        comment,
                        delivered_comment_ids.contains(&comment.id),
                    )
                })
                .collect(),
            review_comment_count: comment_count,
            pending_review_comment_count: comment_count.saturating_sub(delivered),
            preview_targets: task
                .preview_targets
                .iter()
                .map(PreviewTargetProjection::from_target)
                .collect(),
            preview_target_count: task.preview_targets.len(),
            failure_summary: task.failure.as_ref().map(|failure| failure.message.clone()),
            last_activity_at: task.last_activity_at,
        }
    }
}

impl PreviewTargetProjection {
    fn from_target(target: &PreviewTarget) -> Self {
        Self {
            id: target.id,
            label: target.label.clone(),
            uri: target.uri.clone(),
        }
    }
}

impl ReviewCommentProjection {
    fn from_comment(comment: &ReviewComment, delivered: bool) -> Self {
        Self {
            id: comment.id,
            path: comment.path.clone(),
            line_label: comment
                .line
                .as_ref()
                .map(|line| {
                    let line_number = match line.side {
                        DiffSide::Old => line.old_line,
                        DiffSide::New => line.new_line,
                    };
                    line_number
                        .map(|number| format!("{:?} line {}", line.side, number))
                        .unwrap_or_else(|| "file".to_string())
                })
                .unwrap_or_else(|| "file".to_string()),
            body: comment.body.clone(),
            delivered,
        }
    }
}

fn status_tone(status: TaskStatus) -> StatusTone {
    match status {
        TaskStatus::Working | TaskStatus::StartingAgent => StatusTone::Accent,
        TaskStatus::CreatingWorktree | TaskStatus::WaitingForUser | TaskStatus::Reviewing => {
            StatusTone::Warning
        }
        TaskStatus::Blocked | TaskStatus::Failed | TaskStatus::Stale => StatusTone::Danger,
        TaskStatus::Done | TaskStatus::Archived => StatusTone::Muted,
        TaskStatus::Draft | TaskStatus::ReadyForAgent | TaskStatus::ReadyToCommit => {
            StatusTone::Neutral
        }
    }
}
