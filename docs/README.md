# Relay Docs

This directory contains the initial architecture baseline for Relay.

Relay is a Rust + GPUI high-performance AI development workbench:

- Zed-like native UI style
- Orca-like workflow and layout
- Task-native runtime model
- CLI-agent-first execution

## Start Here

- [UI Design Contract](../DESIGN.md)
- [UI Design Workflow](design/README.md)
- [Architecture Investigation](architecture-investigation.html)
- [UI Kit Motion and Architecture Review](reviews/2026-06-19-ui-kit-motion-architecture.md)
- [RFC-001: Domain Model](rfc-001-domain-model.md)
- [RFC-002: Crate Layout](rfc-002-crate-layout.md)
- [RFC-003: Agent Runtime](rfc-003-agent-runtime.md)
- [RFC-004: UI Workbench](rfc-004-ui-workbench.md)

## Architecture Decision Records

- [ADR-0001: Use GPUI for Native UI](adr/0001-use-gpui.md)
- [ADR-0002: Use Provider Boundaries Around External Libraries](adr/0002-use-provider-boundaries.md)
- [ADR-0003: CLI Agent First](adr/0003-cli-agent-first.md)
- [ADR-0004: Use Published GPUI Crate First](adr/0004-use-published-gpui-crate-first.md)
- [ADR-0005: Evaluate Rio Before Adopting Rioterm](adr/0005-evaluate-rio-terminal-rendering.md)
- [ADR-0006: Extract Relay UI Kit and Gallery as a Standalone Project](adr/0006-extract-ui-kit-gallery.md)

## Current Direction

Step 1 has started with a compiling Rust workspace:

- `crates/relay_app`: application bootstrap
- `crates/relay_ui`: GPUI app shell and Zed-like theme tokens
- `crates/relay_infra`: platform paths and tracing setup

The first implementation path is:

```text
create task
  -> create git worktree
  -> spawn CLI agent in PTY
  -> record task/agent status
  -> show changed files and diff
```

The first useful version should prove that Relay can behave like a high-performance Rust-native Orca without losing the native CLI agent experience.
