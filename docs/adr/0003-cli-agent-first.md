# ADR-0003: CLI Agent First

Status: Accepted  
Date: 2026-06-18

## Context

Orca's strongest product insight is that developers want their existing CLI agents to behave exactly as they do in a real terminal.

Many AI IDEs break or wrap terminal behavior. Relay should not.

## Decision

Relay will be CLI-agent-first.

The first runtime path is:

```text
Task -> Worktree -> PTY -> CLI Agent -> Status Adapter -> Diff Review
```

## Consequences

Positive:

- Claude Code, Codex, Gemini and custom CLIs can work without a vendor-specific API.
- Native slash commands and shortcuts are preserved.
- Users can trust that the terminal is real.

Negative:

- PTY behavior is harder than simple chat UI.
- Status detection varies by agent.
- Prompt injection must be handled carefully.

## Guardrails

- Do not rewrite agent UI as chat.
- Do not require vendor APIs for the MVP.
- Do not break TUI shortcuts.
- Agent adapters may observe and assist, but must not own terminal truth.

