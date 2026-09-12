//! Hermes TUI Rust - Main Entry Point
//!
//! This is the main binary that launches the Rust TUI for Hermes Agent.
//! It communicates with the Hermes gateway via JSON-RPC over stdio.

use anyhow::{Context, Result};
use hermes_tui_rust::app::App;
use log::{error, info};

fn main() -> Result<()> {
    // Initialize logging to a file ONLY. Completely avoid terminal corruption.
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("hermes-tui.log")
        .context("Failed to open log file")?;

    // Simple custom logger that writes to the file
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .init();

    // Ensure nothing goes to stderr/stdout that could break Ratatui
    // (except what Ratatui itself handles)

    info!("--- Hermes TUI Session Start ---");

    // Create the app
    let mut app = App::new().context("Failed to initialize application")?;

    // Connect to gateway
    if let Err(e) = app.connect_gateway() {
        error!("Failed to connect to gateway: {}", e);
        // Continue without gateway connection for now
    } else {
        info!("Connected to Hermes gateway");
    }

    if let Err(e) = app.run() {
        error!("Application error: {}", e);
        // cleanup() runs via Drop. Fall through to disconnect_gateway below,
        // then Drop kills the child and restores the terminal.
    }
    // Cleanup
    let _ = app.disconnect_gateway();

    info!("Hermes TUI Rust shutting down gracefully");
    Ok(())
}
