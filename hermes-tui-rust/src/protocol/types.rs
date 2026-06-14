//! Protocol Types - JSON-RPC message types
//!
//! This module contains Rust structs that mirror the TypeScript types
//! in ui-tui/src/gatewayTypes.ts
//!
//! These types are used for serialization/deserialization of JSON-RPC messages
//! sent between the Rust TUI and the Hermes gateway.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Base message envelope for JSON-RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcMessage {
    #[serde(rename = "jsonrpc")]
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// ============================================================================
// Gateway Message Types
// ============================================================================

/// Message role for transcript messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// Transcript message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayTranscriptMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub role: MessageRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_count: Option<usize>,
}

/// Gateway ready response - first message from gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayReadyResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sessions: Option<Vec<SessionListItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<ModelOptionProvider>>,
}

/// Session list item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListItem {
    pub id: String,
    pub message_count: usize,
    pub preview: String,
    pub started_at: i64,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Model option provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOptionProvider {
    pub name: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_models: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_current: Option<bool>,
}

// ============================================================================
// Session Lifecycle Types
// ============================================================================

/// Session create request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toolsets: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree: Option<bool>,
}

/// Session create response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreateResponse {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<SessionInfo>,
}

/// Session resume request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResumeRequest {
    pub session_id: String,
}

/// Session resume response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResumeResponse {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<GatewayTranscriptMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<SessionInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub running: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inflight: Option<SessionInflightTurn>,
}

/// Inflight turn information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInflightTurn {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,
}

/// Session list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sessions: Option<Vec<SessionListItem>>,
}

// ============================================================================
// Message Types
// ============================================================================

/// Message delta (streaming)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<MessageRole>,
    pub delta: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
}

/// Message complete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageComplete {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub role: MessageRole,
    pub content: String,
}

// ============================================================================
// Tool Types
// ============================================================================

/// Tool start notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub tool_name: String,
    pub call_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<String, serde_json::Value>>,
}

/// Tool progress update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProgress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub call_id: String,
    pub output: String,
}

/// Tool complete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolComplete {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub call_id: String,
    pub result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ============================================================================
// Approval Types
// ============================================================================

/// Approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub request_id: String,
    pub tool_name: String,
    pub arguments: HashMap<String, serde_json::Value>,
    pub message: String,
}

/// Approval response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResponse {
    pub request_id: String,
    pub approved: bool,
    pub choice: String,
}

// ============================================================================
// Completion Types
// ============================================================================

/// Completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub display: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<String>,
}

/// Completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<CompletionItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace_from: Option<usize>,
}

/// Slash exec request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashExecRequest {
    pub session_id: String,
    pub command: String,
}

/// Slash exec response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashExecResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

// ============================================================================
// Config Types
// ============================================================================

/// Config get request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigGetRequest {
    pub key: String,
}

/// Config get response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigGetResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<String>,
}

// ============================================================================
// Prompt Types
// ============================================================================

/// Prompt submit request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSubmitRequest {
    pub session_id: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncate_before_user_ordinal: Option<usize>,
}

/// Prompt submit response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSubmitResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

// ============================================================================
// Error Types
// ============================================================================

/// Gateway error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Gateway stderr event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStderr {
    pub line: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
}

// ============================================================================
// Main Gateway Message Enum
// ============================================================================

/// All possible messages from the gateway (event payloads)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum GatewayMessageData {
    // Gateway lifecycle
    #[serde(rename = "gateway.ready")]
    Ready(GatewayReadyResponse),
    #[serde(rename = "gateway.stderr")]
    Stderr(GatewayStderr),
    #[serde(rename = "gateway.activity")]
    Activity(String),

    // Session lifecycle
    #[serde(rename = "session.create")]
    SessionCreate(SessionCreateResponse),
    #[serde(rename = "session.resume")]
    SessionResume(SessionResumeResponse),
    #[serde(rename = "session.list")]
    SessionList(SessionListResponse),
    #[serde(rename = "session.activate")]
    SessionActivate(SessionActivateResponse),
    #[serde(rename = "session.inflight")]
    SessionInflight(SessionInflightResponse),

    #[serde(rename = "session.info")]
    SessionInfo(serde_json::Value),

    // Messages
    #[serde(rename = "message.delta")]
    MessageDelta(MessageDelta),
    #[serde(rename = "message.complete")]
    MessageComplete(MessageComplete),

    // Tools
    #[serde(rename = "tool.start")]
    ToolStart(ToolStart),
    #[serde(rename = "tool.progress")]
    ToolProgress(ToolProgress),
    #[serde(rename = "tool.complete")]
    ToolComplete(ToolComplete),

    // Approvals
    #[serde(rename = "approval.request")]
    ApprovalRequest(ApprovalRequest),

    // Completions
    #[serde(rename = "complete.slash")]
    SlashCompletion(CompletionResponse),
    #[serde(rename = "complete.path")]
    PathCompletion(CompletionResponse),

    // Slash exec
    #[serde(rename = "slash.exec")]
    SlashExec(SlashExecResponse),

    // Config
    #[serde(rename = "config.get")]
    ConfigGet(ConfigGetResponse),

    // Prompt
    #[serde(rename = "prompt.submit")]
    PromptSubmit(PromptSubmitResponse),

    // Error
    #[serde(rename = "error")]
    Error(GatewayError),
}

/// A wrapper for events from the gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayEvent {
    pub session_id: Option<String>,
    #[serde(flatten)]
    pub data: GatewayMessageData,
}

/// Compatibility alias to avoid breaking too much code at once
pub type GatewayMessage = GatewayMessageData;

/// Session activate response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionActivateResponse {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<GatewayTranscriptMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<SessionInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub running: Option<bool>,
}

/// Session inflight response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInflightResponse {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inflight: Option<SessionInflightTurn>,
}

// ============================================================================
// Request Types (from TUI to Gateway)
// ============================================================================

/// All possible requests from TUI to gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum TuiRequest {
    #[serde(rename = "gateway.ready")]
    GatewayReady,

    #[serde(rename = "session.create")]
    SessionCreate(SessionCreateRequest),

    #[serde(rename = "session.resume")]
    SessionResume(SessionResumeRequest),

    #[serde(rename = "session.list")]
    SessionList,

    #[serde(rename = "session.close")]
    SessionClose { session_id: String },

    #[serde(rename = "session.activate")]
    SessionActivate { session_id: String },

    #[serde(rename = "prompt.submit")]
    PromptSubmit(PromptSubmitRequest),

    #[serde(rename = "approval.respond")]
    ApprovalRespond(ApprovalResponse),

    #[serde(rename = "complete.slash")]
    CompleteSlash { query: String },

    #[serde(rename = "complete.path")]
    CompletePath { path: String },

    #[serde(rename = "slash.exec")]
    SlashExec(SlashExecRequest),

    #[serde(rename = "config.get")]
    ConfigGet(ConfigGetRequest),

    #[serde(rename = "config.set")]
    ConfigSet { key: String, value: String },

    #[serde(rename = "terminal.resize")]
    TerminalResize { cols: u16, rows: u16 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_message_serialization() {
        let msg = GatewayMessage::Ready(GatewayReadyResponse {
            sessions: Some(vec![]),
            config: None,
            skin: None,
            models: None,
        });
        
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: GatewayMessage = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            GatewayMessage::Ready(_) => {}
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_message_delta_serialization() {
        let delta = MessageDelta {
            session_key: "test-key".to_string(),
            role: Some(MessageRole::Assistant),
            delta: "Hello".to_string(),
            done: Some(false),
        };
        
        let serialized = serde_json::to_string(&delta).unwrap();
        let deserialized: MessageDelta = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.session_key, "test-key");
        assert_eq!(deserialized.delta, "Hello");
    }

    #[test]
    fn test_session_list_item_serialization() {
        let item = SessionListItem {
            id: "session-123".to_string(),
            message_count: 5,
            preview: "Test session".to_string(),
            started_at: 1234567890,
            title: "Test".to_string(),
            source: None,
        };
        
        let serialized = serde_json::to_string(&item).unwrap();
        let deserialized: SessionListItem = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.id, "session-123");
        assert_eq!(deserialized.message_count, 5);
    }

    #[test]
    fn test_tool_complete_serialization() {
        let tool = ToolComplete {
            session_key: "test-key".to_string(),
            call_id: "call-123".to_string(),
            result: "{\"success\": true}".to_string(),
            duration_ms: Some(150),
            error: None,
        };
        
        let serialized = serde_json::to_string(&tool).unwrap();
        let deserialized: ToolComplete = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.call_id, "call-123");
        assert_eq!(deserialized.result, "{\"success\": true}");
        assert_eq!(deserialized.duration_ms, Some(150));
    }
}
