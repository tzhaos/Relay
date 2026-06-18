# Relay UI Design Workflow

This directory makes the Relay UI direction executable for humans and agents.

Use these files together:

- `../../DESIGN.md`: the active design contract.
- `UI_QA.md`: visual QA checklist for each UI change.
- `references/zed-style-reference.png`: visual style reference.
- `references/orca-layout-reference.png`: layout and workflow reference.
- `prompts/ui-implementation.md`: reusable prompt for UI implementation tasks.

## Roles

Zed is the style north star:

- native desktop feel
- compact editor density
- low chrome
- crisp 1 px separators
- quiet neutral surfaces
- sparse accent color

Orca is the workflow/layout north star:

- project and task rail on the left
- terminal-first work area in the center
- file/diff/review context on the right
- worktree and agent state visible while working

Relay should not become a literal clone of either product. The goal is product quality, density, and workflow clarity.

## Standard Loop

1. Read `../../DESIGN.md`.
2. Inspect the current UI code in `../../crates/relay_ui`.
3. Implement the requested UI behavior or alignment change.
4. Run `cargo fmt`.
5. Run focused tests.
6. Run `cargo run -p relay_app`.
7. Capture the app window.
8. Compare with the reference images and `UI_QA.md`.
9. Iterate before reporting done.

## Screenshot Capture

On Windows, with Relay already running:

```powershell
pwsh scripts/capture-relay-window.ps1 -TitleRegex "Relay" -Output docs/design/qa/latest-relay-window.png
```

If the script cannot find the window, capture the full window manually and save it in `docs/design/qa/`.
