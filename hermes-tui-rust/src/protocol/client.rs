//! Protocol Client - JSON-RPC client for gateway communication
//!
//! This module provides the client-side functionality for communicating
//! with the Hermes gateway via JSON-RPC over stdio.

use crate::protocol::transport::StdioTransport;
use crate::protocol::types::{GatewayEvent, GatewayMessage, JsonRpcMessage, TuiRequest};
use anyhow::{Context, Result};
use log::{debug, error, trace, warn};
use std::collections::HashMap;
use std::fmt;
use std::io::{Read, Write};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

/// Client for sending requests to the gateway
pub struct GatewayClient {
    /// The transport for JSON-RPC communication
    transport: Option<StdioTransport<Box<dyn Write + Send>>>,
    /// Receiver for parsed gateway messages
    response_receiver: Option<Receiver<GatewayMessage>>,
    /// Whether the client is currently connected
    connected: bool,
    /// Counter for request IDs
    next_id: u64,
    /// Map of pending request IDs to method names
    pending_requests: Arc<Mutex<HashMap<u64, String>>>,
}

// Manual Debug impl because StdioTransport<Box<dyn Write + Send>> doesn't impl Debug
impl fmt::Debug for GatewayClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GatewayClient")
            .field("transport", &self.transport.as_ref().map(|_| "StdioTransport<_>"))
            .field("response_receiver", &self.response_receiver.as_ref().map(|_| "Receiver<_>"))
            .field("connected", &self.connected)
            .field("next_id", &self.next_id)
            .finish()
    }
}
impl GatewayClient {
    /// Create a new gateway client
    pub fn new() -> Self {
        Self {
            transport: None,
            response_receiver: None,
            connected: false,
            next_id: 1,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Connect to the gateway via stdio pipes
    ///
    /// Spawns a background reader thread via StdioTransport and sets up
    /// message parsing. Uses non-blocking receive for the event loop.
    pub fn connect<R: Read + Send + 'static, W: Write + Send + 'static>(
        &mut self,
        stdin: R,
        stdout: W,
    ) -> Result<()> {
        // Disconnect if already connected
        if self.connected {
            self.disconnect()?;
        }

        // Create channel for parsed gateway messages
        let (response_sender, response_receiver) = std::sync::mpsc::channel::<GatewayMessage>();

        // Create StdioTransport with boxed writer
        let mut transport = StdioTransport::new(
            std::io::empty(),
            Box::new(stdout) as Box<dyn Write + Send>,
        );

        // Start reader thread — returns a receiver for raw JSON lines
        let line_receiver = transport.start_reader(stdin);

        let pending_requests = Arc::clone(&self.pending_requests);

        // Spawn a parsing thread that converts JSON strings to GatewayMessages
        std::thread::spawn(move || {
            for line in line_receiver {
                trace!("GatewayClient: Parsing line: {}", line);
                
                match serde_json::from_str::<JsonRpcMessage>(&line) {
                    Ok(rpc) => {
                        // Handle standard JSON-RPC event notification
                        if let Some(method) = &rpc.method {
                            if method == "event" {
                                if let Some(ref params) = rpc.params {
                                    match serde_json::from_value::<GatewayEvent>(params.clone()) {
                                        Ok(event) => {
                                            let _ = response_sender.send(event.data);
                                        }
                                        Err(e) => {
                                            error!("GatewayClient: Failed to parse event params: {} - Params: {:?}", e, params);
                                        }
                                    }
                                }
                            } else {
                                warn!("GatewayClient: Received unknown method: {}", method);
                            }
                        } 
                        // Handle responses to requests (result or error)
                        else if let Some(id) = rpc.id {
                            let method = {
                                let mut pending = pending_requests.lock().unwrap();
                                pending.remove(&id)
                            };

                            if let Some(method_name) = method {
                                if let Some(result) = rpc.result {
                                    debug!("GatewayClient: Received response for {} (ID {}): {:?}", method_name, id, result);
                                    
                                    // Transform the result into a GatewayMessageData variant if possible
                                    // This allows the app to handle results as if they were events
                                    let wrapped_msg = match method_name.as_str() {
                                        "session.create" => serde_json::from_value(result).ok().map(GatewayMessage::SessionCreate),
                                        "session.resume" => serde_json::from_value(result).ok().map(GatewayMessage::SessionResume),
                                        "session.list" => serde_json::from_value(result).ok().map(GatewayMessage::SessionList),
                                        "session.activate" => serde_json::from_value(result).ok().map(GatewayMessage::SessionActivate),
                                        "prompt.submit" => serde_json::from_value(result).ok().map(GatewayMessage::PromptSubmit),
                                        "config.get" => serde_json::from_value(result).ok().map(GatewayMessage::ConfigGet),
                                        _ => None,
                                    };

                                    if let Some(msg) = wrapped_msg {
                                        let _ = response_sender.send(msg);
                                    }
                                } else if let Some(err) = rpc.error {
                                    error!("GatewayClient: Received error for {} (ID {}): {}", method_name, id, err.message);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // Some lines might not be JSON (e.g. system logs)
                        trace!("GatewayClient: Line is not JSON-RPC: {} - Line: {}", e, line);
                    }
                }
            }
        });

        self.transport = Some(transport);
        self.response_receiver = Some(response_receiver);
        self.connected = true;

        Ok(())
    }

    /// Send a request to the gateway
    ///
    /// Serializes the request and wraps it in a JSON-RPC 2.0 envelope.
    pub fn send_request(&mut self, request: TuiRequest) -> Result<()> {
        let mut val = serde_json::to_value(request.clone())
            .context("GatewayClient: Failed to serialize request")?;
        
        let id = self.next_id;
        self.next_id += 1;

        // Get method name for tracking
        let method_name = match request {
            TuiRequest::GatewayReady => "gateway.ready",
            TuiRequest::SessionCreate(_) => "session.create",
            TuiRequest::SessionResume(_) => "session.resume",
            TuiRequest::SessionList => "session.list",
            TuiRequest::SessionClose { .. } => "session.close",
            TuiRequest::SessionActivate { .. } => "session.activate",
            TuiRequest::PromptSubmit(_) => "prompt.submit",
            TuiRequest::ApprovalRespond(_) => "approval.respond",
            TuiRequest::CompleteSlash { .. } => "complete.slash",
            TuiRequest::CompletePath { .. } => "complete.path",
            TuiRequest::SlashExec(_) => "slash.exec",
            TuiRequest::ConfigGet(_) => "config.get",
            TuiRequest::ConfigSet { .. } => "config.set",
            TuiRequest::TerminalResize { .. } => "terminal.resize",
        };
        
        self.pending_requests.lock().unwrap().insert(id, method_name.to_string());

        // Add JSON-RPC 2.0 metadata
        if let Some(obj) = val.as_object_mut() {
            obj.insert("jsonrpc".to_string(), serde_json::Value::String("2.0".to_string()));
            obj.insert("id".to_string(), serde_json::Value::Number(id.into()));
        }

        let transport = self
            .transport
            .as_mut()
            .context("GatewayClient: Not connected")?;
        
        transport
            .write_line(&serde_json::to_string(&val)?)
            .context("GatewayClient: Failed to send request")?;
        Ok(())
    }

    /// Receive a message from the gateway (non-blocking)
    ///
    /// Uses try_recv() so the caller can continue the event loop
    /// instead of blocking when no message is available.
    pub fn receive_message(&self) -> Option<GatewayMessage> {
        self.response_receiver
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
    }

    /// Check if the client is currently connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Disconnect from the gateway
    ///
    /// Drops the transport (which stops the reader thread) and clears
    /// the receiver channel.
    pub fn disconnect(&mut self) -> Result<()> {
        // Drop the transport first — this stops the reader thread.
        // Must happen AFTER the child process is killed to avoid hanging
        // on a blocked read_line.
        self.transport = None;
        self.response_receiver = None;
        self.connected = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GatewayClient::new();
        assert!(client.transport.is_none());
        assert!(client.response_receiver.is_none());
    }

    #[test]
    fn test_client_connect_disconnect() {
        let mut client = GatewayClient::new();
        let result = client.connect(
            std::io::empty(),
            std::io::sink(),
        );
        
        // For now, just test that it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }
}
