//! Hermes TUI Rust - A Rust-based TUI for Hermes Agent
//!
//! This library provides a fast, native terminal experience for Hermes Agent,
//! with full compatibility to the existing JSON-RPC protocol.

// #![warn(missing_docs)]  // TODO: Enable when documentation is complete
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::similar_names)]
#![allow(clippy::module_name_repetitions)]
#![allow(missing_docs)] // Temporary during active development

pub mod app;
pub mod engine;
pub mod error;
pub mod handlers;
pub mod protocol;
pub mod state;
pub mod ui;
pub mod utils;

/// Re-export common types for convenience
pub use app::App;
pub use error::{TuiError, TuiResult};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");
