# Project Hermes: TUI Animation & Architecture Plan

This document tracks the animation and architecture plan for the Rust TUI. It has been updated to reflect the current implementation state as of June 19, 2026.

## Current Status

| Area | Status | Notes |
| ---- | ------ | ----- |
| Demand-driven rendering | ✅ Implemented | `src/engine.rs` gates polling and animation ticks through an active-animation counter. |
| DEC 2026 synchronized output | ✅ Implemented | `engine::draw_sync()` wraps frame draws in begin/end synchronized output sequences. |
| Gateway stdio transport | ✅ Implemented | `GatewayClient` spawns `python -m tui_gateway.entry` and reads JSON-RPC events through a background thread. |
| Multi-view workspace | ✅ Implemented | Dashboard, IDE, Kanban, and Chat views are routed by `ViewState`. |
| Animated borders and wave footer | ✅ Implemented | `src/ui/borders.rs` and `src/ui/wave.rs` render live motion. |
| Protocol-linked effects | ✅ Implemented | `src/ui/effects.rs` wraps `tachyonfx::EffectManager` and triggers effects for streaming, tools, view switches, and grand loader moments. |
| Local IDE pane | ✅ Implemented | `src/ui/file_tree.rs` and `src/ui/editor.rs` provide a local file tree and editor. |
| Subagent tracking | ✅ Implemented | `src/ui/subagent.rs` and `CardManager` render subagent/tool activity in chat. |
| Dashboard telemetry | ⚠️ Partial | CPU and memory are real via `sysinfo`; network and token-speed values remain placeholders. |
| Kanban | ⚠️ Mock | The board is visual only and is not connected to durable task state. |
| Tachyonfx buffer bridge | ⚠️ Compatibility shim | `src/ui/effects.rs` uses an unsafe transmute between Ratatui 0.28 and tachyonfx's Ratatui-core 0.1 buffer type. |

## Phase 1: Core Engine & Async Foundation

**Goal:** Establish a strictly bounded, backpressure-aware, demand-driven render loop that never starves the UI thread.

- [x] Use a background reader thread for stdio JSON-RPC events.
- [x] Gate polling and rendering through `engine::poll_timeout()` and the active-animation counter.
- [x] Use DEC mode `?2026h` / `?2026l` around frame draws for synchronized terminal output.
- [ ] Add production-grade E2E tests against a live Hermes gateway.
- [ ] Harden reconnect behavior and document the exact failure modes.

## Phase 2: Component Tree & Mutability Boundaries

**Goal:** Keep rendering pure where practical and route state mutation through the app update path.

- [x] `src/app.rs` owns mutable state and routes between views.
- [x] UI modules render from references where possible.
- [x] Chat cards, subagent lists, and dashboards render from state references.
- [x] The session sidebar uses an intentional raw-pointer borrow workaround; see `src/app.rs` comments.
- [ ] Continue moving complex render helpers into read-only component structs as the UI grows.

## Phase 3: Embedded Local Editor

**Goal:** Provide a useful local developer workspace inside the TUI.

- [x] File tree rooted at the current working directory.
- [x] File type icons and scrollable tree navigation.
- [x] Editor pane backed by `tui-textarea`.
- [x] Focus toggle between file tree and editor.
- [ ] Gateway-backed file edit application.
- [ ] LSP or diagnostics integration.
- [ ] Persistent editor history and undo/redo beyond the current textarea buffer.

## Phase 4: Native Motion & Protocol-Linked Effects

**Goal:** Provide constant visual feedback using pure math and `tachyonfx`.

- [x] Sine-wave block footer in `src/ui/wave.rs`.
- [x] Gradient border animation in `src/ui/borders.rs`.
- [x] Sixel GIF dashboard animation through `bebop.gif`.
- [x] `tachyonfx::EffectManager` wrapper in `src/ui/effects.rs`.
- [x] Streaming, tool, view-transition, and grand-loader effects.
- [ ] Resolve flicker for Sixel/GIF playback on high-latency terminal backends.
- [ ] Replace the tachyonfx/Ratatui buffer transmute with a safer long-term solution.

## Execution Checklist

### Foundation

- [x] Demand-driven render engine.
- [x] DEC 2026 synchronized output.
- [x] Gateway child-process lifecycle.
- [x] stdio JSON-RPC transport.

### Core UX

- [x] Multi-view routing.
- [x] Chat with streaming and tool cards.
- [x] Local IDE workspace.
- [x] Subagent sidebar.
- [x] Help overlay.

### Visual Polish

- [x] Animated borders.
- [x] Wave footer.
- [x] Sixel GIF.
- [x] Protocol-linked effects.
- [x] View transition animations.

### Remaining Hardening

- [ ] Real Kanban integration.
- [ ] Real network and token-speed telemetry.
- [ ] CLI/installer integration for the Rust TUI.
- [ ] Full E2E gateway smoke tests.
- [ ] Safer tachyonfx buffer bridge.

## Notes

- Motion must never block user input or gateway communication.
- Effects are intentionally demand-driven and should stop ticking when idle.
- The current implementation favors experimental velocity over production polish; future work should focus on hardening and integration rather than adding more visual effects.
