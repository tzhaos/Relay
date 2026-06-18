# Relay Design Contract

Status: Active  
Owner: Relay  
Last updated: 2026-06-18

Relay should align with Orca's product structure and visual language while keeping GPUI/Zed's native desktop crispness.

This file is the source of truth for UI implementation. Use it before editing `crates/relay_ui` and before asking an agent to implement interface changes.

## References

- Visual style reference: `docs/design/references/zed-style-reference.png` at 1922x1115.
- Layout and workflow reference: `docs/design/references/orca-layout-reference.png` at 1875x1025.
- Product direction: `docs/rfc-004-ui-workbench.md`.

The Zed screenshot is a GPUI/native quality reference, not the primary visual direction. Use its crisp 1 px dividers, compact chrome, editor-grade typography, and stable density where they improve the Orca-style shell.

The Orca screenshots are the primary product and visual references. Match the left app navigation and project/worktree tree, terminal-centered work area, right file/context panel with toolbar/search/segmented controls, task/worktree mental model, and persistent multi-agent workspace controls.

Do not copy proprietary icons, exact assets, or branding from either product.

## Target Surface

Relay is a Rust + GPUI desktop workbench for CLI-agent development.

Primary screen:

```text
top app bar: custom titlebar, app identity, project, branch, real actions
left rail: project, active worktree, tasks, agent state
center work area: terminal first, preview second
right context: searchable files, diff, review notes, task metadata
bottom/status area: runtime status, focus hints, review/change state
```

The first viewport target is desktop-first:

- Primary QA size: 1920x1080.
- Secondary QA size: 1440x900.
- Minimum usable width: 1180 px.
- Mobile is not a goal for the native desktop shell.

## Layout Contract

Use stable dimensions instead of content-driven panel sizing.

- Window default: 1180x780 until real window state persistence exists; 1440x900+ is the primary design target when display space allows.
- Top app bar: 40-44 px.
- Left rail target: 280-320 px.
- Right context target: 340-380 px.
- Center work area: fills remaining space and owns the largest visual mass.
- Pane headers: 40-42 px.
- Bottom/status strip, when present: 28-32 px.
- Rows: 28-36 px for navigation rows, 56-72 px for task rows.
- Panel dividers: 1 px.
- Main spacing scale: 4, 8, 12, 16, 24.
- No card grid dashboard on the main screen.
- No nested cards.
- No marketing hero layout.

The center terminal must be visually dominant. The right context pane is always available at desktop sizes and should not feel like a modal drawer.

## Visual Contract

Use an Orca-like native desktop palette.

Current palette intent:

- App background: near white, not pure decorative gray.
- Chrome surfaces: light warm gray, with visible-but-soft controls.
- Terminal surface: dark neutral slate.
- Text: high-contrast neutral.
- Muted text: soft gray.
- Accent: sparse green for active/running state.
- Warning: muted amber for attention.
- Danger: muted red for destructive/failure state.

Rules:

- Prefer 1 px borders over shadows.
- Use shadows only for command palette, popovers, and floating tools.
- Radius should stay small-to-medium: 6-10 px for inputs and selected rows.
- Use rounded selected rows, search fields, badges, and segmented controls where Orca does.
- Avoid saturated gradients and decorative blobs.
- Avoid large empty hero-like whitespace.
- Avoid one-note purple, blue, beige, or espresso palettes.
- Keep letter spacing at 0.
- Do not scale type with viewport width.

## Typography

Default UI text should feel native and dense.

- UI font: system UI stack.
- Terminal/code font: Consolas on Windows, otherwise a monospace fallback.
- Body size: 13-14 px.
- Small metadata: 11-12 px.
- Pane title: 13-14 px medium.
- App/title text: 14-15 px medium or semibold.
- Avoid oversized headings inside panels.

Every label must fit its container at the target viewport. If content can be long, truncate or place metadata on a second line.

## Interaction Contract

Relay is keyboard-first.

Required interaction model:

- Click task row to activate a task.
- Terminal focus returns when switching tasks.
- Context tabs switch between Files, Diff, and Review.
- Terminal/Preview route switch preserves task state.
- Command palette is the long-term home for global actions, but it must not be shown as an actionable entry until it exists.
- Focus state must be visible but quiet.

UI must dispatch commands instead of mutating domain state directly. GPUI view code can render projections and send `WorkbenchCommand`; it must not spawn git, PTY, agent, browser, database, or filesystem side effects directly.

Do not delete foundational fake features to make the screen look cleaner. If a displayed control is part of Relay's core workbench model, implement the smallest real behavior behind it. For example, task creation must persist a task event, file search must filter files, and context tabs must switch real projections. Remove only late-stage product areas that Relay does not support yet, such as automation, mobile, and remote collaboration.

## Component Contract

Left rail:

- Top section: Relay identity and the current workspace.
- No Automation, Mobile, or Remote entries until those modules exist.
- Project groups with folder icons and count badges.
- Active project/worktree rows use rounded selection, green status dot, branch/type badge, and path metadata.
- Task creation is a real persisted action, not a visual placeholder.
- Task rows show title, status, agent, branch/path summary, and changed/review counts when available.

Center:

- Terminal is the default route.
- Terminal background should be dark and visually uninterrupted.
- Terminal header shows task/session state and cwd.
- Preview route is available but secondary.

Right context:

- Header includes project/context title, real file filter, and Files/Diff/Review segmented tabs.
- Files tab uses a compact tree.
- Diff tab uses readable hunks without large decorative cards.
- Review tab shows pending comments and delivery state.

Top/bottom chrome:

- Keep compact and utilitarian.
- Use text only where an icon would be ambiguous.
- Avoid feature explanations in the UI itself.

## Implementation Workflow

Use this loop for every UI task:

1. Read `DESIGN.md` and the relevant files under `crates/relay_ui`.
2. Identify which reference applies:
   - Product structure and visual language come from Orca.
   - Native density and GPUI implementation quality come from Zed.
3. Make the smallest UI/code changes that satisfy the requested behavior.
4. Keep dimensions explicit where alignment matters.
5. Run `cargo fmt`.
6. Run focused tests, usually `cargo test -p relay_ui` or the crate touched.
7. Run `cargo run -p relay_app`.
8. Capture the Relay window screenshot.
9. Compare screenshot against the references and `docs/design/UI_QA.md`.
10. Iterate until alignment, density, and hierarchy are acceptable.

For screenshot capture on Windows, after the app is running:

```powershell
pwsh scripts/capture-relay-window.ps1 -TitleRegex "Relay" -Output docs/design/qa/latest-relay-window.png
```

If automated capture fails, manually capture the full app window and save it under `docs/design/qa/`.

## Acceptance Checklist

A UI change is not done until:

- The three-column layout remains stable at 1920x1080 and 1440x900.
- Left and right panes keep their intended widths.
- The center terminal remains dominant.
- Text does not overlap, clip awkwardly, or resize panels.
- Active task, active route, and active context tab are visually obvious.
- There are no card mosaics, nested cards, oversized headings, or decorative gradients.
- `cargo fmt` and focused tests pass.
- A screenshot was captured or the reason it could not be captured is reported.
