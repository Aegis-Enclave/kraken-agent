//! Protocol Transport - stdio transport implementation
//!
//! This module provides the low-level transport for JSON-RPC messages
//! over stdio. It handles reading JSON lines from stdin and writing
//! JSON lines to stdout.

use anyhow::{Context, Result};
use log::{debug, error, trace};
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

/// Transport for reading/writing JSON-RPC messages over stdio
///
/// This transport uses a background thread to read lines from stdin
/// and send them to a channel. This allows the main thread to handle
/// UI rendering while messages are being read asynchronously.
#[derive(Debug)]
pub struct StdioTransport<W: Write + Send + 'static> {
    /// The writer for stdout
    writer: W,
    /// Handle to the reader thread (for cleanup)
    reader_handle: Option<thread::JoinHandle<()>>,
}

impl<W: Write + Send + 'static> StdioTransport<W> {
    /// Create a new stdio transport without starting the reader thread
    pub fn new(_stdin: impl Read + Send + 'static, stdout: W) -> Self {
        Self {
            writer: stdout,
            reader_handle: None,
        }
    }

    /// Start the background reader thread
    ///
    /// This spawns a thread that reads lines from stdin and sends them
    /// to a channel. Returns a receiver that can be used to get incoming
    /// messages.
    pub fn start_reader(&mut self, stdin: impl Read + Send + 'static) -> Receiver<String> {
        let (sender, receiver) = channel::<String>();

        let reader = BufReader::new(stdin);

        let handle = thread::spawn(move || {
            let mut buf_reader = reader;
            loop {
                let mut line = String::new();
                match buf_reader.read_line(&mut line) {
                    Ok(0) => {
                        // EOF - connection closed
                        trace!("Stdio transport: EOF received, connection closed");
                        break;
                    }
                    Ok(_) => {
                        // Successfully read a line
                        trace!("Stdio transport: Read line of {} bytes", line.len());
                        // Try to send the line to the channel
                        if sender.send(line).is_err() {
                            // Channel receiver was dropped, exit thread
                            debug!(
                                "Stdio transport: Channel receiver dropped, exiting reader thread"
                            );
                            break;
                        }
                    }
                    Err(e) => {
                        // Error reading from stdin
                        error!("Stdio transport: Error reading from stdin: {e}");
                        break;
                    }
                }
            }
        });

        self.reader_handle = Some(handle);

        receiver
    }

    /// Write a JSON line to stdout
    ///
    /// This writes the given string as a line to stdout, followed by a newline.
    /// The string should be a valid JSON serialization.
    pub fn write_line(&mut self, line: &str) -> Result<()> {
        writeln!(self.writer, "{line}").context("Failed to write line to stdout")?;

        // Flush to ensure the line is sent immediately
        self.writer.flush().context("Failed to flush stdout")?;

        trace!("Stdio transport: Wrote line of {} bytes", line.len());
        Ok(())
    }

    /// Write a JSON-RPC message to stdout
    ///
    /// Serializes the message to JSON and writes it as a line.
    pub fn write_message<T: serde::Serialize>(&mut self, message: &T) -> Result<()> {
        let json = serde_json::to_string(message).context("Failed to serialize message to JSON")?;
        self.write_line(&json)
    }

    /// Stop the reader thread and clean up resources
    pub fn stop(&mut self) {
        // We take the handle but DON'T join it here. Joining a thread
        // that's blocked on reading from a pipe (which might be closed or
        // not) can lead to deadlocks during cleanup.
        // The thread will exit on its own when the pipe is closed or
        // the sender/receiver are dropped.
        let _ = self.reader_handle.take();
    }
}

impl<W: Write + Send + 'static> Drop for StdioTransport<W> {
    fn drop(&mut self) {
        self.stop();
    }
}

// ============================================================================
// Default Transport (for stdio)
// ============================================================================

/// Default transport using real stdin/stdout
pub type DefaultTransport = StdioTransport<std::io::Stdout>;

impl DefaultTransport {
    /// Create a transport using real stdin/stdout
    #[must_use]
    pub fn from_stdio() -> Self {
        Self::new(std::io::stdin(), std::io::stdout())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use std::io::{empty, sink};

    #[test]
    fn test_transport_creation() {
        let stdin = empty();
        let stdout = sink();
        let transport = StdioTransport::new(stdin, stdout);

        // Transport should be created successfully
        assert!(transport.reader_handle.is_none());
    }

    #[test]
    fn test_write_line() {
        let stdin = empty();
        let stdout = sink();
        let mut transport = StdioTransport::new(stdin, stdout);

        // Just verify it doesn't panic
        transport.write_line("test output").unwrap();
    }

    #[test]
    fn test_write_message() {
        let stdin = empty();
        let stdout = sink();
        let mut transport = StdioTransport::new(stdin, stdout);

        #[derive(Serialize)]
        struct TestMessage {
            field: String,
        }

        let msg = TestMessage {
            field: "value".to_string(),
        };
        transport.write_message(&msg).unwrap();
    }
}
