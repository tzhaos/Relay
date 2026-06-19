# ADR-0006: Extract Relay UI Kit and Gallery as a Standalone Project

Status: Accepted  
Date: 2026-06-19

## Context

`relay_ui_kit` is becoming a reusable GPUI component library instead of a
private implementation detail of the Relay app. Its showcase needs to remain
close to the components, but it should not be confused with Relay's product
runtime or with a separate examples tree.

## Decision

Extract a standalone workspace with this shape:

```text
relay-ui-kit/
├─ Cargo.toml
├─ crates/
│  ├─ relay_ui_kit/
│  └─ relay_gallery/
├─ docs/
│  ├─ DESIGN.md
│  ├─ UI_QA.md
│  └─ adr/
├─ README.md
├─ LICENSE
└─ .gitignore
```

`relay_gallery` is the canonical showcase/example app for the kit. Do not add
`examples/minimal_app` until the public API stabilizes and a minimal host app
would teach something the gallery does not already show.

## Consequences

- UI kit components must remain view-state agnostic and reusable outside Relay.
- Gallery scenes may use demo state, but should not render unsupported Relay
  product features.
- Relay can continue vendoring the crates while the standalone project matures.
