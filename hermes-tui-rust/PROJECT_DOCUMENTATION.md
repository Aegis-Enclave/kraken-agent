# Project Progress Report - June 19, 2026

## Overview

The `hermes-tui-rust` project is now a multi-view native terminal workspace for Hermes Agent. The current implementation centers on a Ratatui/Crossterm app that launches the Python TUI gateway, speaks JSON-RPC over stdio, and renders Dashboard, IDE, Kanban, and Chat views with animated UI polish.

## Key Accomplishments

### 1. View System & Layout

- Implemented `ViewState` in `src/app.rs` with routing for `Dashboard`, `IDE`, `Kanban`, and `Chat`.
- Added global view switching via number keys and a visual top tab/status context.
- Added a transition path for view changes using `transition_progress`, `EffectManager`, and the engine animation counter.
- Implemented a Chat sidebar for subagent monitoring and session context.
- Implemented animated, color-cycling borders for focused panes.

### 2. Autonomous Agent Integration

- Wired gateway subagent events (`SubagentStart`, `SubagentTool`, `SubagentComplete`) into `SubagentList`.
- Improved `CardManager` so tool calls and results render as asynchronous cards within the chat flow.
- Added a specialized `HashlineViewer` for file-diff and edit-operation output.
- Kept approval, clarification, sudo, and secret prompt overlays integrated with the main event loop.

### 3. IDE Workspace

- Added a local file tree rooted at the current working directory.
- Added an editor pane backed by `tui-textarea`.
- Implemented focus toggling between the file tree and editor with `Tab`.
- Opening a file from the tree loads its text into the editor pane.
- The IDE view now renders as a three-pane layout: file tree, editor, and chat.

### 4. Dashboard Telemetry

- Integrated `sysinfo` for real CPU and memory sampling.
- Added CPU and memory gauges plus sparkline histories.
- Added placeholder network throughput and token-speed sparklines for visual completeness; these are not yet backed by real gateway metrics.

### 5. System Resilience

- The Rust app spawns `python -m tui_gateway.entry` as a child process and connects to it through stdio.
- Gateway stderr is redirected to `hermes-tui.log` for background debugging.
- Reconnection logic exists for gateway process failures.
- Demand-driven polling sleeps deeply when no animation or event needs attention.

### 6. Visual Polish

- Replaced the old shader-state sketch with `src/ui/effects.rs` using `tachyonfx::EffectManager`.
- Added protocol-linked effects for streaming, tool execution, view transitions, and grand loader moments.
- Added view transition animations when switching top-level views.
- Retained the wave footer, animated borders, Sixel GIF logo, and help overlay.

## Remaining TODOs

- [ ] **Real Data Integration**
    - [x] Dashboard CPU and memory telemetry via `sysinfo`.
    - [x] IDE file tree and editor wired to the local project workspace.
    - [ ] Dashboard network telemetry backed by real network interface data.
    - [ ] Dashboard token-speed telemetry backed by real gateway/model metrics.
    - [ ] Kanban integration with Hermes' durable task API and real subagent task progress.
- [ ] **Hermes Integration**
    - [ ] Wire the Rust TUI into the main CLI entrypoint, installer, and `display.interface` config path.
    - [ ] Decide final user-facing flag/config name for Rust TUI selection.
- [ ] **UI Polish**
    - [ ] Refine Sixel/GIF playback to eliminate flickering on high-latency terminal backends.
    - [ ] Add safer long-term handling for tachyonfx's Ratatui-core buffer bridge or pin compatible versions.
- [ ] **Final Verification**
    - [ ] Conduct a full end-to-end smoke test against a live Hermes production gateway.
    - [ ] Add tests for IDE file navigation, view transitions, and telemetry sampling.

## Technical Notes

- **Architecture**: `src/app.rs` owns the main event loop and render state. It uses intentional raw-pointer access for the session sidebar draw path to avoid borrow conflicts while reading immutable session data.
- **Theme**: Defaulting to Gruvbox Dark with 24-bit color support.
- **Dependencies**: Added `serde_json` for flexible gateway metadata handling and `sysinfo` for CPU/memory telemetry.
- **Effects**: `tachyonfx` is active, but `src/ui/effects.rs` currently bridges Ratatui 0.28 and tachyonfx's Ratatui-core 0.1 buffer types with an unsafe transmute. Treat this as a compatibility shim that should be revisited before production hardening.
- **Protocol**: The Rust TUI mirrors the existing TypeScript TUI protocol through `src/protocol/types.rs`, `client.rs`, and `transport.rs`.
