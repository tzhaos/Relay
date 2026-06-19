# ADR-0005: Evaluate Rio Before Adopting Rioterm

Status: Accepted  
Date: 2026-06-19

## Context

Relay is a terminal-first workbench. A real terminal renderer is foundational for
CLI agents, TUI behavior, scrollback, selection, resize handling, and keyboard
semantics.

Rio is an active GPU terminal project. Its crates include the `rioterm` package,
which presents Rio as a terminal application. As of `rioterm` 0.4.7, crates.io
reports `rust-version = "1.96.0"`.

Relay currently uses the published `gpui = "0.2.2"` crate and should keep the
terminal UI embedded inside the GPUI workbench.

## Decision

Do not add `rioterm` as a direct Relay dependency now.

Treat Rio as a terminal-rendering spike candidate. The spike should evaluate the
lower-level Rio crates and architecture, not only the `rioterm` application
entry point.

For now, keep Relay's terminal boundary as:

```text
PTY/runtime state -> terminal model -> GPUI terminal surface
```

## Consequences

Positive:

- Relay avoids pulling in a full terminal application when it needs an embedded
  workbench pane.
- The current toolchain remains unblocked.
- Future Rio adoption can focus on reusable terminal model/rendering pieces.

Negative:

- The current GPUI terminal surface remains a lightweight UI surface until a
  real terminal model is integrated.
- A Rio spike is still needed before Relay can claim Rio-grade terminal
  rendering.

## Guardrails

- Do not fake terminal behavior in product surfaces.
- Do not couple terminal runtime state to a specific renderer crate.
- Do not adopt a terminal crate until resize, scrollback, ANSI/VT behavior,
  selection, copy/paste, and keyboard handling are tested inside GPUI.
- Keep `portable-pty` and renderer concerns separated.
