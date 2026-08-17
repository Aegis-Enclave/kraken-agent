# Hermes TUI Rust Architecture

This document describes the current architecture of `hermes-tui-rust`. It is based on the implementation in `src/`, not the older speculative design.

## 1. System Boundary

`hermes-tui-rust` is a standalone Rust binary that renders a native terminal UI for Hermes Agent. It communicates with the Python TUI gateway through JSON-RPC over stdio.

```text
User terminal
  │
  ▼
Rust TUI binary
  │  Ratatui/Crossterm rendering
  │  JSON-RPC requests/responses over child-process stdio
  ▼
Python gateway: python -m tui_gateway.entry
  │
  ▼
Hermes Agent core
```

The Rust app currently discovers the gateway by looking for `tui_gateway/` in the current directory or parent directory. It prefers `venv/bin/python3` when present, otherwise uses `HERMES_PYTHON` or `python3`.

## 2. Major Modules

```text
src/
├── main.rs                 # Binary entry point and file-only logging setup
├── app.rs                  # Main event loop, view routing, gateway lifecycle, rendering
├── engine.rs               # Demand-driven animation gating and DEC 2026 draw sync
├── error.rs                # TUI error types
├── protocol/               # JSON-RPC gateway protocol and stdio transport
│   ├── client.rs           # GatewayClient: requests, pending IDs, response parsing
│   ├── transport.rs        # StdioTransport: background stdin reader, stdout writer
│   └── types.rs            # Request/response/event enum definitions
├── state/                  # Local TUI state
│   ├── config.rs           # Theme and color configuration
│   ├── messages.rs         # Message history and message rendering data
│   └── session.rs          # Session metadata and session list state
└── ui/                     # Ratatui components
    ├── banner.rs           # Dashboard banner
    ├── borders.rs          # Animated gradient borders
    ├── cards.rs            # Tool/message card manager and renderer
    ├── chat.rs             # Chat transcript rendering
    ├── completions.rs      # Slash/path completion popups
    ├── composer.rs         # Composer textarea rendering
    ├── dashboard.rs        # Dashboard view
    ├── editor.rs           # Editor pane using tui-textarea
    ├── effects.rs          # tachyonfx EffectManager wrapper
    ├── file_tree.rs        # Local filesystem tree
    ├── gif.rs              # Sixel GIF rendering
    ├── hashline.rs         # Hashline/diff viewer
    ├── help.rs             # Help overlay
    ├── ide.rs              # IDE view composition
    ├── kanban.rs           # Mock Kanban board
    ├── model_picker.rs     # Model selection popup
    ├── prompts.rs          # Approval/clarify/secret prompts
    ├── session_picker.rs   # Session picker popup
    ├── subagent.rs         # Subagent activity list
    ├── toolbar.rs          # Top toolbar/status line
    └── wave.rs             # Animated footer wave
```

## 3. Application Event Loop

`src/app.rs` is the main state owner. It owns:

- `ViewState`: `Dashboard`, `IDE`, `Kanban`, `Chat`.
- Gateway lifecycle and reconnect logic.
- Chat message state and tool cards.
- IDE file tree/editor state.
- Subagent list.
- Completion popup state.
- Prompt overlay state.
- Rendering effects and transition progress.

The event loop uses Crossterm polling with a demand-driven timeout from `engine::poll_timeout()`. When no animation is active, the loop can sleep deeply instead of redrawing at a fixed 60 FPS cadence.

The render path is synchronized with `engine::draw_sync()`, which wraps frame drawing in DEC 2026 begin/end sequences for terminals that support synchronized output.

## 4. Gateway Protocol

The Rust protocol layer mirrors the existing Hermes TUI protocol.

### `src/protocol/types.rs`

Defines Rust-side JSON-RPC shapes for:

- Gateway readiness.
- Session create/resume/list/close/activate/delete.
- Prompt submission.
- Approval, clarification, sudo, and secret responses.
- Slash and path completions.
- Terminal resize.
- Model options.
- Gateway events such as message deltas, tool starts/progress/completion, and subagent activity.

### `src/protocol/transport.rs`

`StdioTransport<W>` owns:

- A writer for stdout.
- A background reader thread that reads stdin lines and sends them through `std::sync::mpsc`.
- Cleanup that drops the reader handle rather than joining a blocked reader.

### `src/protocol/client.rs`

`GatewayClient` owns:

- The stdio transport.
- A receiver for parsed gateway messages.
- Pending JSON-RPC request IDs.
- Request serialization and response mapping.

The client injects a `payload: null` field when parsing gateway event notifications that omit `payload`, because the tagged enum parser expects that field.

## 5. State Model

### View state

`ViewState` is the top-level routing enum:

- `Dashboard`
- `IDE`
- `Kanban`
- `Chat`

Switching views updates `transition_progress`, starts an animation counter, and triggers `EffectManager::trigger_view_transition()`.

### Chat state

Chat state includes:

- Message history.
- Streaming delta handling.
- Tool cards.
- Hashline viewer.
- Subagent list.
- Completion popups.
- Prompt overlays.
- Model and session pickers.

Tool cards are managed by `CardManager` and rendered in the chat flow with running/completed/failed/pending states.

### IDE state

IDE state includes:

- File tree root and selected file.
- Editor textarea content.
- Focus pane (`Tree` or `Editor`).
- Scroll positions for both panes.

The IDE currently reads the local filesystem. It does not yet apply gateway file edits or LSP diagnostics.

### Session state

`SessionManager` tracks session metadata from the gateway and the currently selected session. Rendering the session sidebar intentionally uses raw pointers in `src/app.rs` to avoid borrow conflicts while reading immutable session data. That workaround is commented in the code and should be revisited if the state model changes.

## 6. UI Rendering

The UI modules are Ratatui components. The intended style is:

- Mutate state in the event/update path.
- Render from references where practical.
- Keep complex render helpers pure.
- Document borrow-checker workarounds.

### Dashboard

`src/ui/dashboard.rs` renders:

- Hermes ASCII banner.
- Sixel GIF via `AnimatedGif`.
- CPU and memory gauges from `sysinfo`.
- CPU and memory sparkline histories.
- Placeholder network and token-speed sparklines.
- Tools, MCP servers, skills, and session/activity panels.

### IDE

`src/ui/ide.rs` composes:

- `FileTree` on the left.
- `EditorPane` in the middle.
- Chat transcript on the right.

`src/ui/file_tree.rs` scans the current directory and renders files/directories with icons. `src/ui/editor.rs` wraps `tui-textarea` and renders the selected file buffer.

### Kanban

`src/ui/kanban.rs` renders a mock board with Backlog, Executing, and Verified columns. It is not connected to durable Hermes task state.

### Chat

`src/ui/chat.rs` renders the conversation and delegates specialized rendering to:

- `CardComponent` for tool cards.
- `HashlineViewer` for diffs.
- `SubagentList` for subagent activity.
- `Composer` for the input area.
- Completion, prompt, model, and session popups.

### Help

`src/ui/help.rs` renders a modal help overlay with global, view, chat, and IDE keybindings.

## 7. Animation and Effects

### Demand-driven engine

`src/engine.rs` owns an `ACTIVE_ANIMATIONS` counter.

- `animation_start()` increments the counter.
- `animation_end()` decrements the counter.
- `poll_timeout()` returns a short timeout while animations are active and a longer sleep timeout while idle.
- `draw_sync()` wraps Ratatui frame drawing in DEC 2026 sequences.

### Effects manager

`src/ui/effects.rs` wraps `tachyonfx::EffectManager`. It supports:

- Streaming text effects.
- Tool execution effects.
- View transition effects.
- Grand loader effects.

Current implementation note: `EffectManager::apply()` uses an unsafe transmute to bridge Ratatui 0.28's buffer type to tachyonfx's Ratatui-core 0.1 buffer type. This is a compatibility shim and should be revisited before production hardening.

### Other motion

- `src/ui/borders.rs` renders animated gradient borders.
- `src/ui/wave.rs` renders a sine-wave block footer.
- `src/ui/gif.rs` renders Sixel GIF frames.

## 8. Configuration and Theme

`src/state/config.rs` defines:

- `TuiConfig`
- `ThemeConfig`
- `ThemeColors`
- `ChatColors`
- `ComposerColors`
- `ToolbarColors`
- `SerialColor`

The default theme is Gruvbox Dark. The Rust TUI currently uses explicit RGB values in UI modules; future production work should centralize these values behind the shared Hermes theme/skin system.

## 9. Build and Test

The crate is a Rust library plus binary.

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -W clippy::all
cargo doc --open
```

The app writes logs to `hermes-tui.log` in the current directory so stdout/stderr remain available for terminal rendering.

## 10. Current Limitations

- The Rust TUI is not yet wired into the main `hermes` CLI.
- The Kanban board is mock data.
- Network and token-speed dashboard metrics are placeholder/mock values.
- The IDE file tree/editor is local-only.
- The tachyonfx/Ratatui buffer bridge uses an unsafe transmute.
- Full E2E validation against a production Hermes gateway is still pending.

## 11. Future Work

Recommended next work, in priority order:

1. Wire the Rust TUI into the main CLI/installer/config path.
2. Connect Kanban to Hermes' durable task API.
3. Replace dashboard placeholder network/token-speed metrics with real telemetry.
4. Add gateway-backed file edit application to the IDE view.
5. Resolve Sixel/GIF flicker on high-latency terminal backends.
6. Replace or pin the tachyonfx/Ratatui compatibility shim.
7. Add E2E tests for gateway lifecycle, view routing, chat/tool rendering, and IDE file navigation.
