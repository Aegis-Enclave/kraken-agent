//! Handlers Module - Event handlers
//!
//! This module contains event handlers for keyboard, mouse, and other input.

pub mod input;
pub mod keys;
pub mod mouse;

// Re-export when modules are implemented
pub use input::*;
pub use keys::*;
pub use mouse::*;
