# RFC-003: Agent Runtime

Status: Draft  
Owner: Relay  
Last updated: 2026-06-18

## Summary

Relay should treat CLI agents as first-class runtime processes while preserving their native terminal behavior.

The product promise:

> If an agent works in a terminal, it should work in Relay without losing shortcuts, slash commands, output format, or TUI behavior.

## Goals

- Launch agent CLIs inside task worktrees
- Preserve native PTY behavior
- Support multiple agent providers through adapters
- Track explicit agent status when available
- Provide stale/fallback status when explicit status is unavailable
- Deliver prompts and review notes safely

## Non-Goals

- Reimplement Claude/Codex/Gemini UI
- Wrap agent output in a custom chat protocol
- Depend on a single agent vendor
- Require agents to expose an API before they can run

## Core Types

```rust
pub enum AgentKind {
    Claude,
    Codex,
    Gemini,
    Custom(String),
}

pub enum AgentStatus {
    Starting,
    Working,
    Waiting,
    Blocked,
    Done,
    Failed,
    Stale,
}

pub enum PromptDelivery {
    Argv,
    Flag { name: String },
    StdinAfterStart,
    DraftPasteAfterReady,
}

pub struct AgentLaunchPlan {
    pub agent: AgentKind,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: PathBuf,
    pub expected_process: Option<String>,
    pub prompt_delivery: PromptDelivery,
}
```

## Adapter API

```rust
pub trait AgentAdapter {
    fn kind(&self) -> AgentKind;
    fn detect(&self, env: &RuntimeEnvironment) -> Task<Result<AgentAvailability>>;
    fn launch_plan(&self, request: AgentLaunchRequest) -> Result<AgentLaunchPlan>;
    fn parse_terminal_event(&self, event: &TerminalEvent) -> Option<AgentStatusUpdate>;
    fn format_followup(&self, message: AgentMessage) -> Result<AgentInput>;
}
```

## Agent Registry

`AgentRegistry` owns adapters.

Responsibilities:

- Register built-in adapters
- Register custom user adapters later
- Detect available agents
- Resolve default agent
- Provide launch plan for a task

Initial built-ins:

- Claude Code: `claude`
- Codex CLI: `codex`
- Gemini CLI: `gemini`

## Launch Flow

```text
User selects agent
  -> ApplicationService::launch_agent(task_id, agent_kind)
  -> TaskService verifies task/worktree
  -> AgentRegistry resolves adapter
  -> Adapter builds AgentLaunchPlan
  -> PtyProvider spawns terminal session
  -> TaskEvent::TerminalStarted
  -> TaskEvent::AgentStarted
  -> UI subscribes to task projection and terminal output
```

## Status Flow

Preferred sources:

1. Explicit hook or structured signal
2. Adapter-specific terminal event parsing
3. Foreground process inspection
4. Time-based stale fallback

Status must never be permanently stuck in `Working`. If no update arrives within a configured threshold, projection should show `Stale` or `Unknown` instead of pretending the agent is still active.

## Prompt Delivery

Prompt delivery must be adapter-specific.

Examples:

- Claude may support argv or prefill-style launch behavior.
- Codex may support argv-style first prompt but requires trust handling.
- Some TUIs may need stdin-after-start.

Review comments sent back to the agent must be formatted as explicit review context, not pasted as raw internal data.

## Safety Rules

- Never inject prompt text into shell command strings without escaping.
- Prefer args/env over shell concatenation.
- Large prompts should use post-start delivery.
- Review payloads must include file path, line/range, selected text, and user comment.
- Browser payloads must be sanitized before reaching agent prompts.

## Initial MVP

MVP must support:

- Detect Claude/Codex/Gemini availability
- Start selected agent in task worktree
- Send initial prompt
- Show status as starting/working/waiting/done/stale
- Stop/kill session safely when task is archived

Deferred:

- mobile follow-up
- agent teams
- background orchestration
- remote hooks
- vendor usage tracking

