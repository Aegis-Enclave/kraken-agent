# Hermes TUI: Visual Design & Aesthetics Architecture

This document records the visual language and layout decisions for the current Rust TUI implementation. It is no longer a speculative plan: the app already has a working multi-view workspace, animated borders, dashboard telemetry, a local IDE pane, a subagent sidebar, and protocol-linked effects.

## 1. Layout & Composition

The current Rust TUI is a full-screen Ratatui application with a top-level view router.

### Active views

- **Dashboard**: branded landing page with animated GIF, CPU/memory gauges, CPU/memory sparklines, placeholder network/token-speed indicators, tools/MCP/skills panels, and activity lists.
- **IDE**: three-column workspace with a local file tree, `tui-textarea` editor pane, and chat pane.
- **Kanban**: mock task board with Backlog, Executing, and Verified columns. It is visual only and not connected to durable task state.
- **Chat**: primary conversation interface with streaming messages, tool cards, hashlines, subagent sidebar, composer, toolbar, and prompt overlays.

### Layout rules

- Use `.spacing(1)` or `.spacing(2)` between major panes so the interface does not feel cramped.
- Active panes use thicker/brighter borders; inactive panes recede with dark gray borders.
- The IDE view currently uses a fixed three-column split: file tree (`Length(30)`), editor (`Percentage(40)`), chat (`Min(40)`).
- The Chat view owns the richest state: transcript, tool cards, hashline viewer, subagent list, composer, prompts, model/session pickers, and completion popups.

## 2. Color & Theme

The Rust TUI defaults to a Gruvbox dark palette. Current code uses explicit RGB values in Ratatui components rather than a centralized theme token system.

Common colors:

- Gruvbox yellow: `Rgb(250, 189, 47)` for active focus, warnings, and primary accents.
- Gruvbox green: `Rgb(131, 165, 152)` for gauges and success-like telemetry.
- Gruvbox pink/purple: `Rgb(211, 134, 155)` for secondary accents and reasoning/tool emphasis.
- Gruvbox dark gray: `Rgb(40, 40, 40)` for backgrounds and inactive borders.

This is acceptable for the experimental Rust branch, but future production hardening should centralize these values behind the same theme/skin system used by the Python TUI.

## 3. Typography & Glyphs

The UI intentionally uses terminal glyphs and emoji-style icons for fast recognition:

- `📁`, `🦀`, `⚙️`, `📝`, `📜`, `📄` for file tree item types.
- `▂▃▄▅▆▇█` for wave footer and block charts.
- `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏` for lightweight spinners.
- `▸` / `▾` are used conceptually for collapsible panels, though the current Rust UI renders most panels directly rather than with a persistent collapsible banner.

The design assumes modern Unicode terminal support. If a terminal lacks emoji or wide glyphs, the UI may still render but visual density can shift.

## 4. Motion Design

Motion is used for state feedback, not decoration.

### Implemented motion

- **Animated borders**: gradient borders pulse when panes are active or the gateway is running.
- **Wave footer**: sine-wave block characters visualize prompt/tool/reason/output/failure activity.
- **Sixel GIF**: `bebop.gif` renders on the dashboard.
- **View transitions**: switching between top-level views triggers a short effect sequence through `EffectManager`.
- **Tool/stream effects**: streaming deltas and tool starts trigger `tachyonfx` effects.
- **Demand-driven rendering**: the engine sleeps deeply when no animation is active.

### Effect manager

`src/ui/effects.rs` wraps `tachyonfx::EffectManager` and exposes:

- `trigger_stream_effect()`
- `trigger_tool_effect()`
- `trigger_view_transition()`
- `trigger_grand_loader()`
- `apply(frame.buffer_mut(), area, delta_ms)`

The current implementation uses an unsafe transmute to bridge Ratatui 0.28 buffers into tachyonfx's Ratatui-core 0.1 buffer type. This works as a compatibility shim but should be treated as a production risk until versions are aligned or a safer bridge is introduced.

## 5. Component Visual Roles

### Dashboard

The dashboard is the branded landing and observability view. It combines:

- Hermes ASCII banner.
- Sixel GIF.
- CPU and memory gauges from `sysinfo`.
- CPU/memory sparklines.
- Placeholder network and token-speed indicators.
- Tool/MCP/skills/sessions panels.

### IDE

The IDE is a local developer workspace:

- Left pane: file tree rooted at the current working directory.
- Middle pane: editable file buffer.
- Right pane: chat transcript and tool cards.

It is not yet an LSP-backed IDE or a gateway file-edit surface.

### Chat

The chat pane is the primary agent surface. It renders:

- User/assistant/system messages.
- Streaming assistant deltas.
- Tool cards with running/completed/failed states.
- Hashline diffs.
- Subagent sidebar with status icons.
- Composer, completion popups, and prompt overlays.

### Kanban

The Kanban view is a mock board. It should not be documented as connected task management until it reads Hermes' durable Kanban API.

## 6. Performance Constraints

- Keep rendering read-only where possible.
- Do not add unbounded queues for UI events.
- Keep animations behind the engine animation counter so idle CPU stays low.
- Avoid applying global effects over Sixel regions until flicker is resolved.
- Do not increase dependency versions without checking whether the tachyonfx/Ratatui buffer bridge still compiles.

## 7. Known Visual Debt

- Hardcoded RGB colors should eventually move into the shared Hermes theme/skin system.
- Sixel/GIF playback can flicker on high-latency terminal backends.
- Kanban is mock data.
- Network and token-speed dashboard metrics are placeholder/mock values.
- The tachyonfx buffer transmute is a compatibility shim, not a long-term architecture.

## 8. Fallback Behavior

If effects are disabled or the terminal cannot safely render them, the UI should remain usable with static borders, text gauges, and normal Ratatui widgets. Motion must never block user input or gateway communication.
