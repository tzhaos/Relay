# Relay UI Implementation Prompt

Use this prompt when asking an agent to implement a Relay UI change.

```text
You are working in the Relay repo.

Before editing code:
- Read DESIGN.md.
- Read docs/design/UI_QA.md.
- Inspect the relevant files in crates/relay_ui.

Design direction:
- Zed is the visual style reference.
- Orca is the layout and workflow reference.
- Do not clone exact assets or branding.

Implementation constraints:
- Keep GPUI view code inside crates/relay_ui.
- UI reads projections and dispatches commands.
- UI must not spawn git, PTY, agent, browser, database, or filesystem side effects directly.
- Use compact editor-grade spacing, 1 px dividers, quiet neutral surfaces, and stable pane dimensions.
- Do not create dashboard card mosaics, nested cards, hero sections, decorative gradients, or oversized panel headings.

After implementation:
- Run cargo fmt.
- Run focused tests for touched crates.
- Run cargo run -p relay_app.
- Capture the Relay window screenshot.
- Compare against docs/design/references/zed-style-reference.png and docs/design/references/orca-layout-reference.png.
- Fix visible alignment, overflow, density, and hierarchy issues before reporting done.

Final response must include:
- changed files
- commands run
- screenshot path or screenshot limitation
- remaining visual deviations, if any
```
