//! Protocol module - JSON-RPC protocol implementation
//!
//! This module handles communication with the Hermes gateway via JSON-RPC over stdio.

pub mod client;
pub mod transport;
pub mod types;

pub use client::*;
pub use transport::*;
pub use types::*;
