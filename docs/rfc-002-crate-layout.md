# RFC-002: Crate Layout and Module Boundaries

Status: Draft  
Owner: Relay  
Last updated: 2026-06-18

## Summary

Relay should be a Rust workspace composed of small, focused crates. The goal is to keep domain logic independent from GPUI and third-party infrastructure while still taking full advantage of the Rust ecosystem.

The high-level rule:

> Core defines Relay concepts. Providers adapt external crates and platform APIs. UI renders projections and dispatches commands.

## Target Workspace

```text
Relay
├─ Cargo.toml
├─ crates
│  ├─ relay_app
│  ├─ relay_ui
│  ├─ relay_core
│  ├─ relay_project
│  ├─ relay_worktree
│  ├─ relay_terminal
│  ├─ relay_agent
│  ├─ relay_diff
│  ├─ relay_preview
│  ├─ relay_persistence
│  └─ relay_infra
├─ docs
├─ fixtures
├─ tests
└─ xtask
```

## Crates

### relay_app

Binary entrypoint.

Responsibilities:

- Initialize logging
- Initialize storage paths
- Initialize GPUI application
- Register application services
- Open first window

Allowed dependencies:

- `relay_ui`
- `relay_core`
- `relay_persistence`
- `relay_infra`
- provider/service crates that are composed at startup, such as `relay_project`, `relay_terminal`, and `relay_agent`
- `gpui`

### relay_ui

GPUI interface layer.

Responsibilities:

- Zed-style native UI
- Orca-like three-column workbench
- Task list
- Terminal pane
- Diff/review pane
- Preview pane
- Command palette
- Keyboard focus and pane routing

Must not:

- Execute git commands directly
- Spawn PTYs directly
- Read/write SQLite directly
- Infer task state from scattered provider details

### relay_core

Pure domain model.

Responsibilities:

- IDs
- task aggregate
- task commands
- task events
- task status state machine
- task projections
- domain errors

Must not depend on:

- GPUI
- SQLite
- PTY libraries
- git libraries
- filesystem watcher libraries

### relay_project

Project/repo registry.

Responsibilities:

- Open repository/workspace root
- Store project metadata
- Resolve default worktree base
- Map projects to tasks
- Track local/remote execution host identity

### relay_worktree

Git worktree and file state.

Responsibilities:

- Create worktree
- Remove worktree safely
- List worktrees
- Read changed files
- Watch files
- Compute git status
- Provide worktree snapshots to core

Initial implementation should use git CLI behind provider boundaries.

### relay_terminal

PTY and terminal session runtime.

Responsibilities:

- Spawn shell/agent process
- Write input
- Stream output
- Resize
- Kill
- Track terminal session state
- Persist terminal snapshot when possible

### relay_agent

CLI agent runtime.

Responsibilities:

- Agent registry
- Agent detection
- Launch plan generation
- Prompt delivery strategy
- Agent status adapter
- Provider session metadata

Initial adapters:

- Claude Code
- Codex CLI
- Gemini CLI

### relay_diff

Diff and review model.

Responsibilities:

- Load changed file diff
- Parse hunks
- Map line comments to file/hunk/line
- Track sent/delivered comments
- Prepare review notes for agent

### relay_preview

Browser/preview abstraction.

Responsibilities:

- Localhost/file preview targets
- Browser selection payload
- Screenshot metadata
- Sanitization and budget enforcement

Deferred from MVP implementation but API should be planned.

### relay_persistence

Durable storage.

Responsibilities:

- SQLite connection
- migrations
- task event log
- task projection cache
- settings
- terminal/agent session metadata

### relay_infra

Shared infrastructure.

Responsibilities:

- platform paths
- logging/tracing
- error helpers
- config loader
- small utilities

## Dependency Direction

Recommended direction:

```text
app
├─ ui
│  └─ core
├─ project
│  ├─ core
│  └─ worktree
├─ agent
│  ├─ core
│  └─ terminal
├─ diff
│  └─ core
├─ persistence
│  └─ core
└─ infra
```

Forbidden:

- `core -> ui`
- `core -> persistence`
- `core -> terminal`
- `ui -> concrete provider internals`

## Provider Boundary Rule

All external libraries must be wrapped.

Examples:

- `portable-pty` is hidden behind `PtyProvider`
- `git` CLI is hidden behind `GitProvider`
- `notify` is hidden behind `FsWatcher`
- SQLite is hidden behind repositories

This gives Relay room to replace implementations without changing the domain model.

## Initial Test Strategy

- `relay_core`: pure unit tests and property-style state transition tests
- `relay_worktree`: integration tests with fixture repos
- `relay_terminal`: PTY smoke tests
- `relay_agent`: adapter launch plan tests
- `relay_persistence`: migration and event replay tests
- `relay_ui`: later GPUI view tests and screenshot checks
