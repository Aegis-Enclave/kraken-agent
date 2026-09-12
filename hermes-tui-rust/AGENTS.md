# Memory

## Project Overview

Hermes TUI Rust is a native Ratatui/Crossterm terminal UI for Hermes Agent. It runs as a standalone Rust binary from `hermes-tui-rust/`, spawns the Python `tui_gateway.entry` gateway as a child process, and communicates over stdio JSON-RPC.

Use this README for the current implementation summary and `ARCHITECTURE.md` for module-level design.

## Code Style Guidelines

- Use descriptive variable names.
- Follow existing patterns in the codebase.
- Extract complex conditions into meaningful boolean variables.
- Keep UI rendering pure where practical; mutate state in the event loop/update phase, then render from references.
- Comment any intentional borrow-checker workaround, especially raw-pointer session sidebar access.
- Do not add dependencies without checking `Cargo.toml` and the existing import style.

## Architecture Notes

- `src/app.rs` is the main state owner and event loop. It routes `ViewState` between Dashboard, IDE, Kanban, and Chat.
- `src/protocol/client.rs` and `src/protocol/transport.rs` implement JSON-RPC stdio transport with a background reader thread.
- `src/ui/ide.rs`, `src/ui/editor.rs`, and `src/ui/file_tree.rs` form the local IDE workspace.
- `src/ui/dashboard.rs` renders real CPU/memory telemetry from `sysinfo`; network and token-speed values are currently placeholder/mock data.
- `src/ui/effects.rs` wraps `tachyonfx::EffectManager` and currently uses a transmute bridge between Ratatui 0.28 buffers and tachyonfx's Ratatui-core 0.1 buffer type.
- `src/engine.rs` owns demand-driven animation tick gating and DEC 2026 synchronized frame drawing.

## Common Workflows

```bash
# From the Hermes repository root
cd hermes-tui-rust

# Run the Rust TUI against the local Python TUI gateway
cargo run

# Check tests
cargo test

# Format
cargo fmt

# Lint
cargo clippy -- -W clippy::all

# Build release binary
cargo build --release
```

When running locally, start from `hermes-tui-rust/` or the Hermes repository root so the app can discover `tui_gateway/`.
