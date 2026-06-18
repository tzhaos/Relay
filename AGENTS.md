# Relay Agent Instructions

For any UI or product-surface work, read `DESIGN.md` before editing code.

Relay's UI direction is:

- Zed-like visual style: native, dense, quiet, low-chrome, editor-grade.
- Orca-like workflow layout: project/task list on the left, terminal-first work area in the center, files/diff/review context on the right.

When changing `crates/relay_ui`, keep UI behavior driven by projections and commands. Do not move git, PTY, browser, or persistence side effects into GPUI view code.

Before finishing UI work:

1. Run formatting and focused tests.
2. Run the app with `cargo run -p relay_app`.
3. Capture or request a screenshot at the target desktop size.
4. Compare against `docs/design/references/zed-style-reference.png` and `docs/design/references/orca-layout-reference.png`.
5. Fix visible alignment, density, overflow, and hierarchy issues before reporting done.
