pub mod command;
pub mod error;
pub mod ids;
pub mod task;
pub mod task_event;
pub mod task_projection;

pub use command::{CreateTask, TaskCommand};
pub use error::{TaskError, TaskResult};
pub use ids::{
    AgentSessionId, PreviewTargetId, ProjectId, ReviewCommentId, TaskId, TerminalSessionId,
    WorktreeId,
};
pub use task::{
    AgentKind, AgentRuntimeStatus, AgentStatusUpdate, ChangeStatus, ChangedFile, DiffReview,
    DiffSide, LineIdentity, PreviewTarget, ProviderFailure, ReviewComment, SelectedRange, Task,
    TaskSource, TaskStatus, Timestamp, WorktreeSnapshot,
};
pub use task_event::{
    AgentStarted, AgentStatusChanged, ChangedFilesUpdated, PreviewAttached, ProviderFailed,
    ReviewCommentAdded, ReviewDelivered, TaskArchived, TaskCreated, TaskEvent, TerminalStarted,
    TerminalStopped, WorktreeAttached, WorktreeRemoved,
};
pub use task_projection::{
    DiffFileProjection, DiffHunkProjection, DiffLineProjection, DiffLineProjectionKind,
    DiffStatsProjection, PreviewTargetProjection, ReviewCommentProjection, StatusTone,
    TaskCommitDraftProjection, TaskDiffProjection, TaskProjection,
};
