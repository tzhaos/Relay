# ADR-0002: Use Provider Boundaries Around External Libraries

Status: Accepted  
Date: 2026-06-18

## Context

Relay should use small, high-quality Rust crates instead of rebuilding PTY, Git, diff, watcher, SQLite, and search infrastructure from scratch.

At the same time, third-party APIs should not leak into the core domain model.

## Decision

Wrap external libraries and platform APIs behind Relay-owned traits.

Examples:

- `PtyProvider`
- `GitProvider`
- `WorktreeProvider`
- `FsWatcher`
- `DiffEngine`
- `TaskRepository`
- `PreviewProvider`

## Consequences

Positive:

- We can move quickly by using existing crates.
- We can replace implementations later.
- Core domain stays stable.
- Tests can use fake providers.

Negative:

- More up-front interface design.
- Provider layer can become too abstract if designed before real usage.

## Guardrails

- Keep provider interfaces small.
- Add methods only when needed by an actual workflow.
- Prefer concrete request/response structs over generic maps.
- Provider errors must convert into task events.

