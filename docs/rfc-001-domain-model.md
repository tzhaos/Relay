# RFC-001: Relay Domain Model

Status: Draft  
Owner: Relay  
Last updated: 2026-06-18

## Summary

Relay is a task-native AI development workbench. The central domain object is `Task`, not editor tab, chat thread, terminal pane, or git branch.

Each task owns the runtime context needed for agentic development:

- Git worktree
- terminal session
- agent session
- changed files
- diff review
- preview targets
- event history

The UI renders projections from this domain model. It should not assemble truth from unrelated terminal, file, git, and agent stores.

## Product Position

Relay is:

- Zed-like in native UI feel and visual restraint
- Orca-like in layout, workflow, and user-facing feature set
- Relay-native in runtime and task model

The domain model must support multiple tasks running concurrently, each with isolated worktree and agent state.

## Aggregate Roots

### Task

`Task` is the primary aggregate root.

Responsibilities:

- Own lifecycle state
- Link project, worktree, terminal, agent, diff, preview, and source metadata
- Accept domain commands
- Emit append-only task events
- Produce task projection for UI

Initial shape:

```rust
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub status: TaskStatus,
    pub project_id: ProjectId,
    pub worktree_id: Option<WorktreeId>,
    pub terminal_session_id: Option<TerminalSessionId>,
    pub agent_session_id: Option<AgentSessionId>,
    pub changed_files: Vec<ChangedFile>,
    pub diff_review: DiffReview,
    pub preview_targets: Vec<PreviewTarget>,
    pub source: TaskSource,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub last_activity_at: Timestamp,
}
```

### Project

`Project` represents a repository or workspace root registered in Relay.

Responsibilities:

- Identify repo root
- Own available worktrees
- Store project-level settings
- Resolve execution host
- Provide default task creation context

### Worktree

`Worktree` represents an isolated git worktree attached to a task.

Responsibilities:

- Track path, branch, base ref, and repo
- Track dirty state and changed files
- Enforce safety around deletion and branch cleanup
- Provide cwd for terminal and agent sessions

### TerminalSession

`TerminalSession` represents a PTY-backed process session.

Responsibilities:

- Own PTY lifecycle
- Stream input/output
- Track geometry
- Track foreground process when available
- Persist resumable terminal snapshot if supported

It must not be responsible for business-level agent state unless an agent adapter explicitly maps terminal events into agent events.

### AgentSession

`AgentSession` represents one CLI agent process launched inside a task.

Responsibilities:

- Record agent kind
- Record launch plan
- Track explicit or inferred status
- Track provider session metadata when available
- Route prompt/review delivery

### DiffReview

`DiffReview` represents review state over the task's current worktree changes.

Responsibilities:

- Track changed files and hunks
- Store line-level comments
- Track delivery state for comments sent to agent
- Support commit message and PR description generation later

### PreviewTarget

`PreviewTarget` represents a browser/file/localhost context attached to a task.

Responsibilities:

- Track preview URL or file
- Track selection payloads
- Track screenshots or DOM snippets if available
- Enforce data budget and secret redaction

## Task Status

Initial statuses:

```rust
pub enum TaskStatus {
    Draft,
    CreatingWorktree,
    StartingAgent,
    Working,
    WaitingForUser,
    Blocked,
    Done,
    Reviewing,
    ReadyToCommit,
    Archived,
    Failed,
}
```

Status meaning:

- `Draft`: task record exists but no worktree/agent has started.
- `CreatingWorktree`: task is creating or attaching a git worktree.
- `StartingAgent`: terminal exists and CLI agent launch is in progress.
- `Working`: agent is actively processing.
- `WaitingForUser`: agent is waiting for user input or next instruction.
- `Blocked`: agent or provider reports a blocking condition.
- `Done`: agent turn or task run has completed.
- `Reviewing`: user is reviewing diffs/comments.
- `ReadyToCommit`: worktree changes are ready for commit or PR.
- `Archived`: task is hidden from active workflow.
- `Failed`: task cannot continue without repair.

## Commands

Commands are user/system intent. They may be rejected if invalid.

```rust
pub enum TaskCommand {
    CreateTask(CreateTask),
    AttachWorktree(WorktreeSnapshot),
    AttachTerminal(TerminalSessionId),
    AttachAgent(AgentSessionId),
    ApplyAgentStatus(AgentStatusUpdate),
    RefreshChangedFiles(Vec<ChangedFile>),
    AddReviewComment(ReviewComment),
    MarkReviewDelivered(Vec<ReviewCommentId>),
    AttachPreview(PreviewTarget),
    Archive,
    MarkFailed(ProviderFailure),
}
```

## Events

Events are append-only facts. The event log is the durable source of truth.

```rust
pub enum TaskEvent {
    TaskCreated(TaskCreated),
    WorktreeCreated(WorktreeCreated),
    WorktreeAttached(WorktreeAttached),
    TerminalStarted(TerminalStarted),
    AgentStarted(AgentStarted),
    AgentStatusChanged(AgentStatusChanged),
    ChangedFilesUpdated(ChangedFilesUpdated),
    ReviewCommentAdded(ReviewCommentAdded),
    ReviewDelivered(ReviewDelivered),
    PreviewAttached(PreviewAttached),
    TaskArchived(TaskArchived),
    ProviderFailed(ProviderFailure),
}
```

## Projection

`TaskProjection` is the UI read model.

It should include only UI-ready information:

- task title
- status badge
- active agent kind
- terminal presence
- changed file count
- review comment count
- preview availability
- last activity time
- failure/warning summary

The UI must not reach into provider internals to infer this state.

## Invariants

- A task may have zero or one primary worktree.
- A task may have zero or more terminal sessions later, but MVP uses one primary terminal.
- A task may have zero or one primary agent session in MVP.
- Worktree deletion requires clean state or explicit force path.
- Provider failures must be represented as events.
- Agent status must become stale if no update arrives within configured threshold.
- Review comments must belong to a task and a file revision context.

## Deferred

Not in the first implementation:

- Multiple agents per task
- Full LSP/editor domain
- GitHub/Linear first-class issue objects
- Mobile companion state
- Collaboration/multiplayer data model

