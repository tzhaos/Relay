# ADR-0001: Use GPUI for Native UI

Status: Accepted  
Date: 2026-06-18

## Context

Relay aims to be a Rust-native high-performance AI development workbench. The desired visual style follows Zed: native, fast, dense, restrained, and keyboard-first.

Zed proves that GPUI can support a serious editor-grade application.

## Decision

Use GPUI as the primary UI framework.

## Consequences

Positive:

- Native UI feel
- Strong alignment with Zed-like design direction
- Rust-first application stack
- Integrated app/entity/task model

Negative:

- GPUI is pre-1.0 and may change
- Documentation is limited compared with mature UI frameworks
- We must keep domain core independent from GPUI to reduce future migration risk

## Guardrails

- `relay_core` must not depend on GPUI.
- UI reads projections and dispatches commands.
- Business state is not owned by view structs.

