# UI Kit Motion and Architecture Review

Date: 2026-06-19  
Scope: `relay_ui_kit`, `relay_gallery`, and the integration boundary with `relay_ui`

## Summary

This milestone adds a small GPUI-native motion layer and the first feedback
component set for terminal and agent workflows. The implementation is useful,
not decorative: motion is limited to short entry, fade, pulse, and spinner
states; feedback components cover launch failure, missing runtime dependency,
loading, progress, skeleton, and transient restore notices.

The larger architecture issue is now clear: `relay_gallery` is proving the UI
kit, but the real `relay_ui` crate still owns a separate component and theme
surface. If that split remains, Relay can end up with a polished gallery and a
different product UI.

## What Changed

- Added `relay_ui_kit::motion`:
  - `MotionDuration`
  - `MotionDirection`
  - `MotionExt`
- Added `relay_ui_kit::feedback`:
  - `LoadingSpinner`
  - `ProgressBar`
  - `Skeleton`
  - `InlineError`
  - `Banner`
  - `Toast`
- Added gallery coverage in the Settings scene for realistic terminal and
  agent feedback states.
- Removed the unused `anyhow` dependency from `relay_ui_kit`; this keeps the
  library lighter and avoids binary-style error handling in a reusable crate.

## Current Strengths

- `relay_ui_kit` is mostly view-state agnostic. Components are `RenderOnce`
  builders and do not depend on the Relay app shell.
- The module layout is now directionally healthy: shell, terminal, command,
  viewer, git, rows, controls, feedback, and motion are separated by role.
- The gallery is correctly positioned as the canonical showcase/example app,
  not as a second runtime.
- Motion uses GPUI's `Animation` and `AnimationExt`; there is no custom timer or
  animation scheduler.
- Viewer components already cover file, markdown, code, and diff surfaces,
  which fits Relay's "terminal plus context" product shape without trying to
  become a full code editor.

## Main Risks

### 1. Real UI and UI Kit Divergence

`relay_gallery` consumes `relay_ui_kit`, but `relay_ui` still uses
`crates/relay_ui/src/components.rs` and `crates/relay_ui/src/theme.rs`.

Risk: the gallery improves while the real app keeps a separate visual language,
duplicated button/badge/tone logic, and separate fixes for hover/focus/error
states.

Recommendation: add `relay_ui_kit` as a dependency of `relay_ui` and migrate in
thin slices:

1. Feedback and status primitives: `Banner`, `InlineError`, `Toast`, `Badge`,
   `StatusDot`.
2. Title/shell primitives: `TitleBar`, `TopToolbar`, `StatusBar`, `SplitPane`.
3. Terminal/task rows: `TerminalSurface`, `TerminalToolbar`,
   `TerminalSessionRow`, `TaskRow`, `TreeRow`.
4. Context panes: `FileView`, `MarkdownView`, `CodeView`, `DiffView`.

Do not migrate all panes at once. The app already has real runtime behavior,
and a broad UI rewrite would make regressions hard to isolate.

### 2. Theme Boundary Is Duplicated

`relay_ui_kit::Theme` and `relay_ui::RelayTheme` overlap. This is currently
acceptable during extraction, but it should not become permanent.

Recommendation: make `relay_ui` install and read the UI kit theme, or add a
small adapter while migrating. Avoid copying color tokens between crates.

### 3. Command Side-Effect Boundary Needs Naming Clarity

`WorkspaceViewModel::apply_command` intentionally treats runtime commands as
no-ops, while `AppShell::dispatch` handles side effects through
`TaskDataSource`. This is not fake implementation by itself, but the naming
makes the boundary easy to misunderstand.

Recommendation: split or rename the command surface:

- `WorkbenchViewCommand` for pure view model changes.
- `WorkbenchRuntimeCommand` for project, task, agent, terminal, review, and
  preview effects.

This would make it harder to accidentally add a clickable UI action that only
updates local projection state.

### 4. Terminal Renderer Is Still a Boundary, Not a Finished Terminal

ADR-0005 is still the right direction: do not add `rioterm` directly as a
terminal application dependency. Relay needs an embedded terminal pane with a
clear boundary:

```text
PTY/runtime state -> terminal model -> GPUI terminal surface
```

Recommendation: keep `portable-pty` and renderer concerns separate. Evaluate
Rio lower-level pieces only after resize, scrollback, ANSI/VT handling,
selection, copy/paste, and keyboard behavior are tested inside GPUI.

### 5. Open-Source Extraction Needs a Clean Public Contract

`relay_ui_kit` can be extracted as an open-source project, but the public API
should stay conservative:

- Keep domain-heavy components generic enough to be useful outside Relay.
- Keep Relay-specific demo data in `relay_gallery`, not in component APIs.
- Do not add `examples/minimal_app` until it teaches something the gallery does
  not already cover.
- Keep `anyhow` out of the library crate unless it is only used in test helpers
  or binaries.

## Component Coverage Assessment

Ready enough to consume:

- Buttons, badges, status dots
- Text input, checkbox, radio, toggle
- Menu, overlay, command palette, launcher
- App shell, title bar, toolbar, split pane, pane surface, status bar
- Terminal surface, terminal tabs, terminal toolbar, terminal rows, quick launch
- File, markdown, code, and diff viewers
- Branch selector and branch action menu
- Feedback and motion components added in this milestone

Still missing or incomplete for production:

- Modal/dialog/sheet family
- Context menu and nested submenu behavior
- Virtualized tree/list for large workspaces
- Toast host/stack manager instead of standalone toast body
- Accessibility/focus contracts for every interactive composite
- Screenshot-backed gallery QA for 1440x900 and 1920x1080 after each visual
  milestone

## Refactor Plan

### Phase 1: Stop Divergence

- Add `relay_ui_kit` dependency to `relay_ui`.
- Replace local `Tone`, badge, inline error, and simple button usage first.
- Keep `AppShell::dispatch` and runtime behavior unchanged.

### Phase 2: Shell and Terminal Unification

- Replace hand-built title bar/status bar pieces with UI kit shell components.
- Replace terminal chrome with `TerminalSurface`, `TerminalToolbar`, and
  `TerminalSessionRow`.
- Keep the existing terminal runtime projection as the data source.

### Phase 3: Context Pane Unification

- Move file/diff/markdown/code context presentation to UI kit viewers.
- Keep review submission and preview side effects in `relay_ui`.

### Phase 4: Public Extraction

- Mirror the stable crates into the standalone `relay-ui-kit` workspace.
- Add README, license, CI commands, screenshots, and design docs.
- Publish only after gallery and at least one host app consume the same public
  API without private Relay assumptions.

## Verification For This Milestone

- `cargo fmt --check`
- `cargo test -p relay_ui_kit`
- `cargo build -p relay_gallery`
