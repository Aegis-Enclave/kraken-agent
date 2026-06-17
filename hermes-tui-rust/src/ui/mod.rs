//! UI Module - Terminal UI components
//!
//! This module contains all the UI components for the TUI.

pub mod banner;
pub mod cards;
pub mod chat;
pub mod completions;
pub mod composer;
pub mod dashboard;
pub mod effects;
pub mod gif;
pub mod hashline;
pub mod ide;
pub mod kanban;
pub mod model_picker;
pub mod prompts;
pub mod session_picker;
pub mod subagent;
pub mod toolbar;
pub mod wave;

// Re-export when modules are implemented
// pub use chat::*;
// pub use composer::*;
// pub use toolbar::*;
// pub use cards::*;
// pub use prompts::*;
pub use model_picker::ModelPicker;
