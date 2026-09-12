//! Error module - Custom error types for the TUI
//!
//! This module provides comprehensive error handling for the Hermes TUI Rust.
//! All errors are unified through the `TuiError` enum with conversions from
//! underlying error types.

use std::io;
use std::num::ParseIntError;
use std::string::FromUtf8Error;

use serde_json::Error as SerdeJsonError;
use thiserror::Error;

/// Result type alias for convenience
pub type TuiResult<T> = Result<T, TuiError>;

/// Main error enum for the TUI
///
/// This enum provides a unified error type that can represent all errors
/// that can occur in the TUI, with conversions from underlying error types.
#[derive(Debug, Error)]
pub enum TuiError {
    /// I/O error (reading from stdin, writing to stdout, file operations)
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] SerdeJsonError),

    /// Terminal error from crossterm
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// Protocol error (invalid message format, unexpected message type, etc.)
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// State error (invalid state transition, missing data, etc.)
    #[error("State error: {0}")]
    State(String),

    /// Rendering error (failed to render UI component)
    #[error("Render error: {0}")]
    Render(String),

    /// Configuration error (invalid config file, missing required setting, etc.)
    #[error("Configuration error: {0}")]
    Config(String),

    /// Connection error (gateway not available, connection lost, etc.)
    #[error("Connection error: {0}")]
    Connection(String),

    /// Timeout error (operation timed out)
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// Invalid input error (user provided invalid input)
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Not implemented error (feature not yet implemented)
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

impl TuiError {
    /// Create a new protocol error
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol(message.into())
    }

    /// Create a new state error
    pub fn state(message: impl Into<String>) -> Self {
        Self::State(message.into())
    }

    /// Create a new render error
    pub fn render(message: impl Into<String>) -> Self {
        Self::Render(message.into())
    }

    /// Create a new config error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Create a new connection error
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection(message.into())
    }

    /// Create a new timeout error
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout(message.into())
    }

    /// Create a new invalid input error
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    /// Create a new not implemented error
    pub fn not_implemented(message: impl Into<String>) -> Self {
        Self::NotImplemented(message.into())
    }

    /// Check if this is a connection error
    #[must_use]
    pub fn is_connection(&self) -> bool {
        matches!(self, Self::Connection(_))
    }

    /// Check if this is a protocol error
    #[must_use]
    pub fn is_protocol(&self) -> bool {
        matches!(self, Self::Protocol(_))
    }

    /// Check if this is a state error
    #[must_use]
    pub fn is_state(&self) -> bool {
        matches!(self, Self::State(_))
    }

    /// Get the error message as a string
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::Io(e) => e.to_string(),
            Self::Json(e) => e.to_string(),
            Self::Terminal(s) => s.clone(),
            Self::Protocol(s) => s.clone(),
            Self::State(s) => s.clone(),
            Self::Render(s) => s.clone(),
            Self::Config(s) => s.clone(),
            Self::Connection(s) => s.clone(),
            Self::Timeout(s) => s.clone(),
            Self::InvalidInput(s) => s.clone(),
            Self::NotImplemented(s) => s.clone(),
        }
    }
}

// Conversion from ParseIntError
impl From<ParseIntError> for TuiError {
    fn from(err: ParseIntError) -> Self {
        Self::InvalidInput(err.to_string())
    }
}

// Conversion from FromUtf8Error
impl From<FromUtf8Error> for TuiError {
    fn from(err: FromUtf8Error) -> Self {
        Self::Protocol(err.to_string())
    }
}

// Conversion from anyhow::Error for compatibility
impl From<anyhow::Error> for TuiError {
    fn from(err: anyhow::Error) -> Self {
        Self::State(err.to_string())
    }
}

// Conversion from log errors
impl From<log::SetLoggerError> for TuiError {
    fn from(err: log::SetLoggerError) -> Self {
        Self::State(format!("Failed to initialize logger: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let tui_err: TuiError = io_err.into();
        assert!(matches!(tui_err, TuiError::Io(_)));
        assert!(tui_err.to_string().contains("File not found"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err = serde::de::Error::custom("Invalid JSON");
        let tui_err: TuiError = TuiError::Json(json_err);
        assert!(matches!(tui_err, TuiError::Json(_)));
    }

    #[test]
    fn test_protocol_error() {
        let err = TuiError::protocol("Invalid message format");
        assert!(matches!(err, TuiError::Protocol(_)));
        assert!(err.to_string().contains("Invalid message format"));
    }

    #[test]
    fn test_state_error() {
        let err = TuiError::state("Session not found");
        assert!(matches!(err, TuiError::State(_)));
        assert!(err.to_string().contains("Session not found"));
    }

    #[test]
    fn test_connection_error() {
        let err = TuiError::connection("Gateway not available");
        assert!(matches!(err, TuiError::Connection(_)));
        assert!(err.is_connection());
        assert!(!err.is_protocol());
    }

    #[test]
    fn test_not_implemented_error() {
        let err = TuiError::not_implemented("Feature X");
        assert!(matches!(err, TuiError::NotImplemented(_)));
    }

    #[test]
    fn test_error_message() {
        let err = TuiError::protocol("Test error");
        assert_eq!(err.message(), "Test error");
    }

    #[test]
    fn test_is_methods() {
        let io_err = TuiError::Io(io::Error::new(io::ErrorKind::Other, "test"));
        let protocol_err = TuiError::protocol("test");
        let state_err = TuiError::state("test");

        assert!(!io_err.is_connection());
        assert!(!io_err.is_protocol());
        assert!(!io_err.is_state());

        assert!(!protocol_err.is_connection());
        assert!(protocol_err.is_protocol());
        assert!(!protocol_err.is_state());

        assert!(!state_err.is_connection());
        assert!(!state_err.is_protocol());
        assert!(state_err.is_state());
    }

    #[test]
    fn test_from_utf8_error() {
        let err = String::from_utf8(b"invalid utf-8\xFF".to_vec()).unwrap_err();
        let tui_err: TuiError = err.into();
        assert!(matches!(tui_err, TuiError::Protocol(_)));
    }

    #[test]
    fn test_from_parse_int_error() {
        let err: ParseIntError = "99999999999999999999".parse::<i32>().unwrap_err();
        let tui_err: TuiError = err.into();
        assert!(matches!(tui_err, TuiError::InvalidInput(_)));
    }
}
