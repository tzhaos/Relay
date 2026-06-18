use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    command::TaskCommand,
    error::{TaskError, TaskResult},
    ids::{
        AgentSessionId, PreviewTargetId, ProjectId, ReviewCommentId, TaskId, TerminalSessionId,
        WorktreeId,
    },
    task_event::{
        AgentStarted, AgentStatusChanged, ChangedFilesUpdated, PreviewAttached, ProviderFailed,
        ReviewCommentAdded, ReviewDelivered, TaskArchived, TaskCreated, TaskEvent, TerminalStarted,
        WorktreeAttached,
    },
};

pub type Timestamp = OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Draft,
    CreatingWorktree,
    StartingAgent,
    Working,
    WaitingForUser,
    Blocked,
    Done,
    Stale,
    Reviewing,
    ReadyToCommit,
    Archived,
    Failed,
}

impl TaskStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::CreatingWorktree => "WORKTREE",
            Self::StartingAgent => "STARTING",
            Self::Working => "WORKING",
            Self::WaitingForUser => "WAITING",
            Self::Blocked => "BLOCKED",
            Self::Done => "DONE",
            Self::Stale => "STALE",
            Self::Reviewing => "REVIEW",
            Self::ReadyToCommit => "READY",
            Self::Archived => "ARCHIVED",
            Self::Failed => "FAILED",
        }
    }

    fn is_mutable(self) -> bool {
        !matches!(self, Self::Archived | Self::Failed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskSource {
    Manual,
    Imported { source: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentKind {
    Claude,
    Codex,
    Gemini,
    Custom(String),
}

impl AgentKind {
    pub fn label(&self) -> &str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::Custom(label) => label.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRuntimeStatus {
    Working,
    Blocked,
    Waiting,
    Done,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentStatusUpdate {
    pub state: AgentRuntimeStatus,
    pub prompt: String,
    pub agent_kind: Option<AgentKind>,
    pub observed_at: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Untracked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangedFile {
    pub path: String,
    pub status: ChangeStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorktreeSnapshot {
    pub id: WorktreeId,
    pub path: String,
    pub branch: String,
    pub base_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffSide {
    Old,
    New,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineIdentity {
    pub path: String,
    pub side: DiffSide,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub hunk_header: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedRange {
    pub start: LineIdentity,
    pub end: LineIdentity,
    pub selected_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewComment {
    pub id: ReviewCommentId,
    pub task_id: TaskId,
    pub path: String,
    #[serde(default)]
    pub line: Option<Box<LineIdentity>>,
    #[serde(default)]
    pub selected_range: Option<Box<SelectedRange>>,
    pub body: String,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveredReviewComment {
    pub id: ReviewCommentId,
    pub delivered_at: Timestamp,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffReview {
    pub comments: Vec<ReviewComment>,
    pub delivered_comments: Vec<DeliveredReviewComment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewTarget {
    pub id: PreviewTargetId,
    pub label: String,
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderFailure {
    pub provider: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub status: TaskStatus,
    pub project_id: ProjectId,
    pub worktree: Option<WorktreeSnapshot>,
    pub terminal_session_id: Option<TerminalSessionId>,
    pub agent_session_id: Option<AgentSessionId>,
    pub agent_kind: Option<AgentKind>,
    pub agent_prompt: String,
    pub changed_files: Vec<ChangedFile>,
    pub diff_review: DiffReview,
    pub preview_targets: Vec<PreviewTarget>,
    pub source: TaskSource,
    pub failure: Option<ProviderFailure>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub last_activity_at: Timestamp,
}

impl Task {
    pub fn create(command: crate::command::CreateTask) -> TaskResult<(Self, Vec<TaskEvent>)> {
        let title = command.title.trim();
        if title.is_empty() {
            return Err(TaskError::EmptyTitle);
        }

        let event = TaskEvent::TaskCreated(TaskCreated {
            id: command.id.unwrap_or_default(),
            project_id: command.project_id,
            title: title.to_string(),
            source: command.source,
            occurred_at: command.now,
        });
        let task = Self::replay([event.clone()].iter())?;
        Ok((task, vec![event]))
    }

    pub fn replay<'a>(events: impl IntoIterator<Item = &'a TaskEvent>) -> TaskResult<Self> {
        let mut task: Option<Self> = None;
        for event in events {
            match &mut task {
                Some(task) => task.apply(event)?,
                None => {
                    let TaskEvent::TaskCreated(created) = event else {
                        return Err(TaskError::MissingTaskCreated);
                    };
                    task = Some(Self::from_created(created));
                }
            }
        }
        task.ok_or(TaskError::MissingTaskCreated)
    }

    pub fn handle(&self, command: TaskCommand) -> TaskResult<Vec<TaskEvent>> {
        match command {
            TaskCommand::CreateTask(_) => Err(TaskError::InvalidStatus(self.status)),
            TaskCommand::AttachWorktree { snapshot, now } => {
                self.ensure_mutable()?;
                if self.worktree.is_some() {
                    return Err(TaskError::WorktreeAlreadyAttached);
                }
                if self.status != TaskStatus::Draft {
                    return Err(TaskError::InvalidStatus(self.status));
                }
                Ok(vec![TaskEvent::WorktreeAttached(WorktreeAttached {
                    task_id: self.id,
                    snapshot,
                    occurred_at: now,
                })])
            }
            TaskCommand::AttachTerminal { id, now } => {
                self.ensure_mutable()?;
                if self.worktree.is_none() {
                    return Err(TaskError::MissingWorktree);
                }
                if self.terminal_session_id.is_some() {
                    return Err(TaskError::TerminalAlreadyAttached);
                }
                Ok(vec![TaskEvent::TerminalStarted(TerminalStarted {
                    task_id: self.id,
                    id,
                    occurred_at: now,
                })])
            }
            TaskCommand::AttachAgent {
                id,
                kind,
                started_at,
            } => {
                self.ensure_mutable()?;
                if self.terminal_session_id.is_none() {
                    return Err(TaskError::MissingTerminal);
                }
                if self.agent_session_id.is_some() {
                    return Err(TaskError::AgentAlreadyAttached);
                }
                Ok(vec![TaskEvent::AgentStarted(AgentStarted {
                    task_id: self.id,
                    id,
                    kind,
                    occurred_at: started_at,
                })])
            }
            TaskCommand::ApplyAgentStatus(update) => {
                self.ensure_mutable()?;
                if self.agent_session_id.is_none() {
                    return Err(TaskError::MissingTerminal);
                }
                Ok(vec![TaskEvent::AgentStatusChanged(AgentStatusChanged {
                    task_id: self.id,
                    update,
                })])
            }
            TaskCommand::RefreshChangedFiles { files, now } => {
                self.ensure_mutable()?;
                Ok(vec![TaskEvent::ChangedFilesUpdated(ChangedFilesUpdated {
                    task_id: self.id,
                    files,
                    occurred_at: now,
                })])
            }
            TaskCommand::AddReviewComment(comment) => {
                self.ensure_mutable()?;
                if comment.task_id != self.id {
                    return Err(TaskError::ReviewCommentTaskMismatch);
                }
                Ok(vec![TaskEvent::ReviewCommentAdded(ReviewCommentAdded {
                    task_id: self.id,
                    comment,
                })])
            }
            TaskCommand::MarkReviewDelivered { comment_ids, now } => {
                self.ensure_mutable()?;
                Ok(vec![TaskEvent::ReviewDelivered(ReviewDelivered {
                    task_id: self.id,
                    comment_ids,
                    occurred_at: now,
                })])
            }
            TaskCommand::AttachPreview { target, now } => {
                self.ensure_mutable()?;
                Ok(vec![TaskEvent::PreviewAttached(PreviewAttached {
                    task_id: self.id,
                    target,
                    occurred_at: now,
                })])
            }
            TaskCommand::Archive { now } => {
                self.ensure_mutable()?;
                Ok(vec![TaskEvent::TaskArchived(TaskArchived {
                    task_id: self.id,
                    occurred_at: now,
                })])
            }
            TaskCommand::MarkFailed { failure, now } => {
                self.ensure_mutable()?;
                Ok(vec![TaskEvent::ProviderFailed(ProviderFailed {
                    task_id: self.id,
                    failure,
                    occurred_at: now,
                })])
            }
        }
    }

    pub fn apply(&mut self, event: &TaskEvent) -> TaskResult<()> {
        if event.task_id() != self.id {
            return Err(TaskError::TaskIdMismatch);
        }

        match event {
            TaskEvent::TaskCreated(_) => return Err(TaskError::InvalidStatus(self.status)),
            TaskEvent::WorktreeAttached(event) => {
                self.worktree = Some(event.snapshot.clone());
                self.status = TaskStatus::CreatingWorktree;
                self.touch(event.occurred_at);
            }
            TaskEvent::TerminalStarted(event) => {
                self.terminal_session_id = Some(event.id);
                self.status = TaskStatus::StartingAgent;
                self.touch(event.occurred_at);
            }
            TaskEvent::AgentStarted(event) => {
                self.agent_session_id = Some(event.id);
                self.agent_kind = Some(event.kind.clone());
                self.status = TaskStatus::StartingAgent;
                self.touch(event.occurred_at);
            }
            TaskEvent::AgentStatusChanged(event) => {
                self.status = match event.update.state {
                    AgentRuntimeStatus::Working => TaskStatus::Working,
                    AgentRuntimeStatus::Blocked => TaskStatus::Blocked,
                    AgentRuntimeStatus::Waiting => TaskStatus::WaitingForUser,
                    AgentRuntimeStatus::Done => TaskStatus::Done,
                    AgentRuntimeStatus::Stale => TaskStatus::Stale,
                };
                if let Some(kind) = &event.update.agent_kind {
                    self.agent_kind = Some(kind.clone());
                }
                self.agent_prompt = event.update.prompt.clone();
                self.touch(event.update.observed_at);
            }
            TaskEvent::ChangedFilesUpdated(event) => {
                self.changed_files = event.files.clone();
                self.touch(event.occurred_at);
            }
            TaskEvent::ReviewCommentAdded(event) => {
                self.diff_review.comments.push(event.comment.clone());
                self.status = TaskStatus::Reviewing;
                self.touch(event.comment.created_at);
            }
            TaskEvent::ReviewDelivered(event) => {
                for id in &event.comment_ids {
                    if !self
                        .diff_review
                        .delivered_comments
                        .iter()
                        .any(|entry| entry.id == *id)
                    {
                        self.diff_review
                            .delivered_comments
                            .push(DeliveredReviewComment {
                                id: *id,
                                delivered_at: event.occurred_at,
                            });
                    }
                }
                if self.status == TaskStatus::Reviewing {
                    self.status = TaskStatus::ReadyToCommit;
                }
                self.touch(event.occurred_at);
            }
            TaskEvent::PreviewAttached(event) => {
                self.preview_targets.push(event.target.clone());
                self.touch(event.occurred_at);
            }
            TaskEvent::TaskArchived(event) => {
                self.status = TaskStatus::Archived;
                self.touch(event.occurred_at);
            }
            TaskEvent::ProviderFailed(event) => {
                self.failure = Some(event.failure.clone());
                self.status = TaskStatus::Failed;
                self.touch(event.occurred_at);
            }
        }
        Ok(())
    }

    fn from_created(event: &TaskCreated) -> Self {
        Self {
            id: event.id,
            title: event.title.clone(),
            status: TaskStatus::Draft,
            project_id: event.project_id,
            worktree: None,
            terminal_session_id: None,
            agent_session_id: None,
            agent_kind: None,
            agent_prompt: String::new(),
            changed_files: Vec::new(),
            diff_review: DiffReview::default(),
            preview_targets: Vec::new(),
            source: event.source.clone(),
            failure: None,
            created_at: event.occurred_at,
            updated_at: event.occurred_at,
            last_activity_at: event.occurred_at,
        }
    }

    fn ensure_mutable(&self) -> TaskResult<()> {
        if self.status.is_mutable() {
            Ok(())
        } else {
            Err(TaskError::InvalidStatus(self.status))
        }
    }

    fn touch(&mut self, now: Timestamp) {
        self.updated_at = now;
        self.last_activity_at = now;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        AgentKind, AgentRuntimeStatus, AgentSessionId, AgentStatusUpdate, ChangeStatus,
        ChangedFile, CreateTask, ProjectId, ReviewComment, ReviewCommentId, TaskCommand, TaskError,
        TaskEvent, TaskProjection, TaskSource, TaskStatus, TerminalSessionId, WorktreeId,
        WorktreeSnapshot,
    };

    use super::{Task, Timestamp};

    fn now() -> Timestamp {
        Timestamp::UNIX_EPOCH
    }

    fn create_task(title: &str) -> (Task, Vec<TaskEvent>) {
        Task::create(CreateTask {
            id: None,
            project_id: ProjectId::new(),
            title: title.to_string(),
            source: TaskSource::Manual,
            now: now(),
        })
        .expect("task creation should succeed")
    }

    fn apply_command(task: &mut Task, log: &mut Vec<TaskEvent>, command: TaskCommand) {
        let events = task.handle(command).expect("command should be valid");
        for event in &events {
            task.apply(event).expect("event should apply");
        }
        log.extend(events);
    }

    fn attach_worktree(task: &mut Task, log: &mut Vec<TaskEvent>) {
        apply_command(
            task,
            log,
            TaskCommand::AttachWorktree {
                snapshot: WorktreeSnapshot {
                    id: WorktreeId::new(),
                    path: "/repo/worktree".to_string(),
                    branch: "task/domain".to_string(),
                    base_ref: Some("main".to_string()),
                },
                now: now(),
            },
        );
    }

    fn attach_terminal_and_agent(task: &mut Task, log: &mut Vec<TaskEvent>) {
        apply_command(
            task,
            log,
            TaskCommand::AttachTerminal {
                id: TerminalSessionId::new(),
                now: now(),
            },
        );
        apply_command(
            task,
            log,
            TaskCommand::AttachAgent {
                id: AgentSessionId::new(),
                kind: AgentKind::Codex,
                started_at: now(),
            },
        );
    }

    #[test]
    fn replay_should_rebuild_projection_from_event_log() {
        let (mut task, mut log) = create_task("Implement event log");
        attach_worktree(&mut task, &mut log);
        attach_terminal_and_agent(&mut task, &mut log);
        apply_command(
            &mut task,
            &mut log,
            TaskCommand::ApplyAgentStatus(AgentStatusUpdate {
                state: AgentRuntimeStatus::Working,
                prompt: "Wire projection".to_string(),
                agent_kind: Some(AgentKind::Codex),
                observed_at: now(),
            }),
        );
        apply_command(
            &mut task,
            &mut log,
            TaskCommand::RefreshChangedFiles {
                files: vec![ChangedFile {
                    path: "crates/relay_core/src/task.rs".to_string(),
                    status: ChangeStatus::Modified,
                }],
                now: now(),
            },
        );

        let replayed = Task::replay(log.iter()).expect("event log should replay");
        let projection = TaskProjection::from_task(&replayed);

        assert_eq!(projection.title, "Implement event log");
        assert_eq!(projection.status, TaskStatus::Working);
        assert!(projection.has_terminal);
        assert_eq!(projection.changed_file_count, 1);
    }

    #[test]
    fn terminal_should_require_worktree_boundary() {
        let (task, _) = create_task("Spawn terminal too early");
        let error = task
            .handle(TaskCommand::AttachTerminal {
                id: TerminalSessionId::new(),
                now: now(),
            })
            .expect_err("terminal cannot start without worktree");

        assert_eq!(error, TaskError::MissingWorktree);
    }

    #[test]
    fn agent_should_require_terminal_boundary() {
        let (mut task, mut log) = create_task("Spawn agent too early");
        attach_worktree(&mut task, &mut log);

        let error = task
            .handle(TaskCommand::AttachAgent {
                id: AgentSessionId::new(),
                kind: AgentKind::Claude,
                started_at: now(),
            })
            .expect_err("agent cannot start without terminal");

        assert_eq!(error, TaskError::MissingTerminal);
    }

    #[test]
    fn explicit_agent_status_should_drive_task_status() {
        let (mut task, mut log) = create_task("Map agent states");
        attach_worktree(&mut task, &mut log);
        attach_terminal_and_agent(&mut task, &mut log);

        apply_command(
            &mut task,
            &mut log,
            TaskCommand::ApplyAgentStatus(AgentStatusUpdate {
                state: AgentRuntimeStatus::Waiting,
                prompt: "Need input".to_string(),
                agent_kind: Some(AgentKind::Codex),
                observed_at: now(),
            }),
        );
        assert_eq!(task.status, TaskStatus::WaitingForUser);

        apply_command(
            &mut task,
            &mut log,
            TaskCommand::ApplyAgentStatus(AgentStatusUpdate {
                state: AgentRuntimeStatus::Done,
                prompt: "Finished".to_string(),
                agent_kind: Some(AgentKind::Codex),
                observed_at: now(),
            }),
        );
        assert_eq!(task.status, TaskStatus::Done);
    }

    #[test]
    fn archived_task_should_reject_later_mutation() {
        let (mut task, mut log) = create_task("Archive me");
        apply_command(&mut task, &mut log, TaskCommand::Archive { now: now() });

        let error = task
            .handle(TaskCommand::RefreshChangedFiles {
                files: Vec::new(),
                now: now(),
            })
            .expect_err("archived task is immutable");

        assert_eq!(error, TaskError::InvalidStatus(TaskStatus::Archived));
    }

    #[test]
    fn review_comment_should_belong_to_task() {
        let (task, _) = create_task("Review mismatch");
        let error = task
            .handle(TaskCommand::AddReviewComment(ReviewComment {
                id: ReviewCommentId::new(),
                task_id: crate::TaskId::new(),
                path: "src/lib.rs".to_string(),
                line: None,
                selected_range: None,
                body: "Wrong task".to_string(),
                created_at: now(),
            }))
            .expect_err("comment task id must match");

        assert_eq!(error, TaskError::ReviewCommentTaskMismatch);
    }
}
