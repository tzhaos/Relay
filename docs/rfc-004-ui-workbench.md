# RFC-004: UI Workbench

Status: Draft  
Owner: Relay  
Last updated: 2026-06-18

## Summary

Relay UI should feel like Zed but work like Orca.

Visual and interaction style:

- Native
- Fast
- Dense
- Low chrome
- Keyboard-first
- Calm and utilitarian

Workflow and layout:

- Orca-like task/worktree/agent list
- terminal-centered agent operation
- file/diff/review context always visible
- preview/browser context attached to task

## Layout

Initial workbench:

```text
┌──────────────────────────────────────────────────────────┐
│ Top Bar: project / branch / active task / command palette │
├──────────────┬──────────────────────────┬────────────────┤
│ Task List    │ Terminal / Preview        │ Files / Diff   │
│ Worktrees    │ Agent CLI                 │ Review Notes   │
│ Agent State  │ Logs                      │ Metadata       │
└──────────────┴──────────────────────────┴────────────────┘
```

## Panels

### Left: Task List

Responsibilities:

- show active project
- group tasks by status
- show task title
- show worktree branch/path summary
- show agent badge
- show changed file count
- show waiting/done/failed state

Status groups:

- Working
- Waiting
- Reviewing
- Done
- Failed
- Archived

### Center: Terminal / Preview

Responsibilities:

- host primary task terminal
- show native CLI agent
- switch to preview target when needed
- preserve terminal state when switching tasks

### Right: Files / Diff / Review

Responsibilities:

- show changed files
- show diff hunks
- allow line comments
- show review delivery state
- later show commit/PR draft

## UI State Model

UI reads:

- `WorkspaceProjection`
- `TaskProjection`
- `TerminalProjection`
- `DiffProjection`

UI sends:

- application commands
- focus commands
- pane commands

UI must not:

- spawn git/PTY operations directly
- mutate task domain state directly
- infer agent status by scraping terminal text

## View Model

```rust
pub struct WorkspaceProjection {
    pub active_project: Option<ProjectProjection>,
    pub active_task_id: Option<TaskId>,
    pub task_groups: Vec<TaskGroupProjection>,
    pub panes: PaneProjection,
}

pub struct TaskProjection {
    pub id: TaskId,
    pub title: String,
    pub status: TaskStatus,
    pub agent: Option<AgentKind>,
    pub branch: Option<String>,
    pub changed_file_count: usize,
    pub review_comment_count: usize,
    pub has_terminal: bool,
    pub has_preview: bool,
    pub last_activity_at: Timestamp,
    pub warning: Option<String>,
}
```

## Visual Direction

Use Zed as the visual north star:

- restrained contrast
- compact rows
- clear focus rings
- minimal borders
- no dashboard-card mosaic
- no marketing hero treatment
- native text rendering
- keyboard-first interaction

Relay should not copy Zed's exact assets or proprietary details. The goal is similar product quality and density, not pixel cloning.

## Commands

Initial commands:

- open project
- create task
- create task with agent
- launch agent in active task
- send note to agent
- refresh changed files
- archive task
- toggle diff/review pane
- focus task list
- focus terminal
- focus diff

## MVP Acceptance

MVP UI is acceptable when:

- user can open project
- user can create task worktree
- user can launch agent in terminal
- task list updates status
- changed files appear on the right
- diff is readable
- switching between tasks preserves context
- keyboard focus does not get lost

