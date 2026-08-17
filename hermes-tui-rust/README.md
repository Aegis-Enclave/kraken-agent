# Hermes TUI Rust

A native Rust terminal UI for Hermes Agent, built with Ratatui and Crossterm. It is an experimental TUI implementation that speaks the same JSON-RPC-over-stdio protocol as the existing Hermes TUI gateway.

## Status

🚧 **Under development** — the Rust TUI can be built and run from this directory, but it is not yet wired into the main `hermes` CLI entrypoint or the managed installer path.

## Overview

The project provides a fast, native terminal surface for Hermes Agent sessions. It launches the Python TUI gateway as a child process, connects through stdio JSON-RPC, and renders the conversation and workspace state with Ratatui.

Current focus areas:

- Multi-view terminal workspace: Dashboard, IDE, Kanban, and Chat.
- Live chat rendering with streaming messages, tool cards, approval/clarify/secret prompts, and subagent activity.
- Local IDE workspace: a file tree rooted at the current directory and an editor pane that loads selected files.
- Dashboard telemetry: real CPU and memory samples from `sysinfo`, plus placeholder network and token-speed sparklines.
- Visual polish: animated borders, Sixel GIF rendering, view-switch transitions, wave footer, and `tachyonfx` post-processing effects.
- Demand-driven rendering: the event loop sleeps deeply when no animation or user/gateway event needs attention.

## Implemented Features

- [x] Core chat interface with streaming and Markdown support.
- [x] Multi-view switching between Dashboard, IDE, Kanban, and Chat.
- [x] Animated Sixel GIF logo support using `bebop.gif`.
- [x] Gruvbox theme default with high-contrast 24-bit colors.
- [x] Slash-command and path completion popups.
- [x] Model picker, session picker, approval prompts, clarification prompts, and secret prompts.
- [x] Mouse support, scrolling, and bracketed paste.
- [x] Tool cards with asynchronous status/result rendering.
- [x] Hashline viewer for file-diff style output.
- [x] Subagent sidebar tracking for start/tool/complete events.
- [x] Local file tree and editor pane in the IDE view.
- [x] CPU and memory telemetry from `sysinfo`.
- [x] View transition animations and global effect manager.
- [x] Help overlay with global, view, chat, and IDE keybindings.

## Current Limitations

- The Kanban view is still a mock board; it is not connected to Hermes' durable Kanban API.
- The IDE file tree reads the local filesystem only. It does not yet apply gateway file edits or LSP-backed changes.
- Network throughput and token-speed dashboard sparklines are placeholder/mock values.
- The Rust TUI is not yet exposed through `hermes --tui-rust`, `display.interface: rust`, or the managed installer.
- Full end-to-end validation against a live Hermes production gateway is still pending.

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│ Hermes Core (Python)                                        │
│  tui_gateway.entry / tui_gateway.server                     │
│  JSON-RPC sessions, agents, tools, approvals, completions    │
└─────────────────────────────────────────────────────────────┘
                          │ stdio JSON-RPC
                          ▼
┌─────────────────────────────────────────────────────────────┐
│ hermes-tui-rust (Rust / Ratatui / Crossterm)                │
│  App event loop, ViewState router, GatewayClient            │
│  Chat, Dashboard, IDE, Kanban, prompts, effects             │
└─────────────────────────────────────────────────────────────┘
```

The Rust side is intentionally standalone for now:

- `src/app.rs` owns the main event loop, view state, gateway lifecycle, rendering, and input routing.
- `src/protocol/*` serializes gateway messages and transports JSON-RPC over stdio.
- `src/ui/*` contains Ratatui components for every visible pane.
- `src/state/*` stores local TUI state such as themes, input mode, focus panes, sessions, and message history.
- `src/engine.rs` gates demand-driven animation ticks and wraps frame draws with DEC 2026 synchronized output.

## Project Structure

```text
hermes-tui-rust/
├── Cargo.toml
├── README.md
├── ARCHITECTURE.md
├── DESIGN.md
├── MODERN_TUI_ANIMATIONS.md
├── PROJECT_DOCUMENTATION.md
├── AGENTS.md
├── build.rs
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── engine.rs
│   ├── error.rs
│   ├── protocol/
│   │   ├── client.rs
│   │   ├── transport.rs
│   │   └── types.rs
│   ├── state/
│   │   ├── config.rs
│   │   ├── messages.rs
│   │   └── session.rs
│   ├── ui/
│   │   ├── banner.rs
│   │   ├── borders.rs
│   │   ├── cards.rs
│   │   ├── chat.rs
│   │   ├── completions.rs
│   │   ├── composer.rs
│   │   ├── dashboard.rs
│   │   ├── editor.rs
│   │   ├── effects.rs
│   │   ├── file_tree.rs
│   │   ├── gif.rs
│   │   ├── hashline.rs
│   │   ├── help.rs
│   │   ├── ide.rs
│   │   ├── kanban.rs
│   │   ├── model_picker.rs
│   │   ├── prompts.rs
│   │   ├── session_picker.rs
│   │   ├── subagent.rs
│   │   ├── toolbar.rs
│   │   └── wave.rs
│   └── utils/
│       ├── ansi.rs
│       ├── clipboard.rs
│       └── text.rs
└── tests/
```

## Running Locally

Run from the Hermes repository root so the Rust app can find `tui_gateway/`:

```bash
cd hermes-tui-rust
cargo run
```

For a release build:

```bash
cargo build --release
./target/release/hermes-tui-rust
```

The app writes its own logs and gateway stderr to `hermes-tui.log` in the current directory.

## Development

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

### Format and lint

```bash
cargo fmt
cargo clippy -- -W clippy::all
```

### Documentation

```bash
cargo doc --open
```

## Keybindings

| View | Key | Action |
| ---- | --- | ------ |
| Global | `Alt+A` | Enter tmux-style prefix mode |
| Global | `?` | Toggle help overlay |
| Global | `Ctrl+C` | Quit |
| Views | `1` / `2` / `3` / `4` | Dashboard / IDE / Kanban / Chat |
| Chat | `i` | Insert mode |
| Chat | `Esc` | Normal mode |
| Chat | `Tab` | Next pane |
| IDE | `Tab` | Toggle focus between file tree and editor |
| IDE | `Enter` | Open selected file in the editor |
| IDE | `j/k` or `↓/↑` | Move in the focused tree/editor pane |

## Quality Gates

Before merging changes:

1. `cargo fmt` is clean.
2. `cargo clippy -- -W clippy::all` passes.
3. `cargo test` passes.
4. Docs accurately describe the current implementation.
5. No new unresolved borrow-checker workaround is introduced without a comment explaining why it is needed.

## License

MIT License. See the repository `LICENSE` file.

## Acknowledgments

- Inspired by the oh-my-pi terminal UI.
- Built with [ratatui](https://github.com/ratatui-org/ratatui), [crossterm](https://github.com/crossterm-rs/crossterm), `tui-textarea`, `sysinfo`, and `tachyonfx`.
- Part of [Hermes Agent](https://github.com/NousResearch/hermes-agent).
