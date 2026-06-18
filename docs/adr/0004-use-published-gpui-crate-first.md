# ADR-0004: Use Published GPUI Crate First

Status: Accepted  
Date: 2026-06-18

## Context

Relay initially tried to depend on the local Zed checkout at `D:\Workspace\zed\crates\gpui`. That tracks Zed mainline directly, but the current checkout uses unstable Rust APIs that do not compile with the installed stable toolchain.

The published `gpui` crate on crates.io provides the public application API Relay needs for Step 1, including `Application`, `App`, `Render`, window creation, layout, and styling primitives.

## Decision

Use `gpui = "0.2.2"` from crates.io for the initial implementation.

Do not depend on `gpui_platform` directly. App startup should use `gpui::Application::new().run(...)`.

## Consequences

Positive:

- Step 1 builds on stable Rust without patching Zed source.
- Relay depends on GPUI's published public API instead of internal workspace crates.
- Future upgrades can be evaluated through Cargo version changes and focused API diffs.

Negative:

- The crates.io release may lag Zed mainline.
- Some examples or APIs from the local Zed checkout may not match the released crate.

## Guardrails

- Keep all GPUI usage inside `relay_ui` and `relay_app`.
- Keep core/task/worktree/agent/diff crates independent of GPUI.
- Record any GPUI API mismatch as an ADR or migration note before adopting Zed mainline internals.
