use crate::task::TaskStatus;

pub type TaskResult<T> = Result<T, TaskError>;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TaskError {
    #[error("task title cannot be empty")]
    EmptyTitle,
    #[error("event log does not contain a TaskCreated event")]
    MissingTaskCreated,
    #[error("event belongs to a different task")]
    TaskIdMismatch,
    #[error("command cannot be applied while task is {0:?}")]
    InvalidStatus(TaskStatus),
    #[error("task must have a worktree before starting a terminal")]
    MissingWorktree,
    #[error("task must have a terminal before starting an agent")]
    MissingTerminal,
    #[error("task already has a worktree")]
    WorktreeAlreadyAttached,
    #[error("task already has a terminal")]
    TerminalAlreadyAttached,
    #[error("task already has an agent")]
    AgentAlreadyAttached,
    #[error("review comment does not belong to this task")]
    ReviewCommentTaskMismatch,
}
