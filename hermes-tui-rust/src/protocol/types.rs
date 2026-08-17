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

/// Optional capability counts reported by the gateway in the ready message.
/// Older gateways don't send this; we fall back to a `Default` empty struct.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GatewayCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_count: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_count: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<Vec<String>>,
}

/// Gateway ready response - first message from gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayReadyResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sessions: Option<Vec<SessionListItem>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skin: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<ModelOptionProvider>>,
    /// Optional capability counts for the chat empty-state landing page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<GatewayCapabilities>,
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
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendered: Option<String>,
}
/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TokenUsage {
    #[serde(default)]
    pub prompt_tokens: usize,
    #[serde(default)]
    pub completion_tokens: usize,
    #[serde(default)]
    pub total_tokens: usize,
    #[serde(default)]
    pub cache_read_tokens: Option<usize>,
    #[serde(default)]
    pub cache_write_tokens: Option<usize>,
    #[serde(default)]
    pub prompt_category_tokens: Option<u32>,
    #[serde(default)]
    pub tool_call_tokens: Option<u32>,
    #[serde(default)]
    pub reasoning_tokens: Option<u32>,
    #[serde(default)]
    pub output_tokens: Option<u32>,
    #[serde(default)]
    pub failed_tool_call_tokens: Option<u32>,
}

impl TokenUsage {
    pub fn get_mock_detailed_usage(&self) -> (u32, u32, u32, u32, u32) {
        (
            self.prompt_category_tokens.unwrap_or(150),
            self.tool_call_tokens.unwrap_or(80),
            self.reasoning_tokens.unwrap_or(400),
            self.output_tokens.unwrap_or(200),
            self.failed_tool_call_tokens.unwrap_or(0),
        )
    }
}

/// Message complete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageComplete {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub text: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendered: Option<String>,
}
// Tool Types
// ============================================================================

/// Tool start notification
/// Tool start notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Tool name — gateway sends as "name"
    #[serde(rename = "name")]
    pub tool_name: String,
    /// Call ID — gateway sends as "`tool_id`"
    #[serde(rename = "tool_id")]
    pub call_id: String,
    /// Tool args as JSON string — gateway sends as "`args_text`"
    #[serde(rename = "args_text", default)]
    pub arguments: Option<String>,
}

/// Tool progress update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProgress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(alias = "tool_name", alias = "name")]
    pub call_id: String,
    #[serde(alias = "preview", alias = "text")]
    pub output: String,
}

/// Tool complete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolComplete {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Call ID — gateway sends as "`tool_id`"
    #[serde(rename = "tool_id")]
    pub call_id: String,
    /// Tool result — gateway sends as a JSON object
    #[serde(default)]
    pub result: serde_json::Value,
    /// Duration in seconds from gateway — stored as ms
    #[serde(rename = "duration_s", default, deserialize_with = "secs_to_ms")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ============================================================================
// Subagent Types
// ============================================================================

/// Subagent event from the gateway (start, thinking, progress, tool, complete)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentEvent {
    pub goal: String,
    pub task_count: usize,
    pub task_index: usize,
    pub subagent_id: Option<String>,
    pub parent_id: Option<String>,
    pub child_session_id: Option<String>,
    pub depth: Option<usize>,
    pub model: Option<String>,
    pub tool_count: Option<usize>,
    pub toolsets: Option<Vec<String>>,
    pub tool_name: Option<String>,
    pub text: Option<String>,
    pub status: Option<String>,
    pub summary: Option<String>,
    pub duration_seconds: Option<f64>,
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

    #[serde(rename = "status.update")]
    StatusUpdate(serde_json::Value),

    #[serde(rename = "reasoning.available")]
    ReasoningAvailable(serde_json::Value),

    #[serde(rename = "reasoning.delta")]
    ReasoningDelta(serde_json::Value),

    #[serde(rename = "message.start")]
    MessageStart(serde_json::Value),

    #[serde(rename = "message.delta")]
    MessageDelta(MessageDelta),

    // Subagents
    #[serde(rename = "subagent.start")]
    SubagentStart(SubagentEvent),
    #[serde(rename = "subagent.thinking")]
    SubagentThinking(SubagentEvent),
    #[serde(rename = "subagent.progress")]
    SubagentProgress(SubagentEvent),
    #[serde(rename = "subagent.tool")]
    SubagentTool(SubagentEvent),
    #[serde(rename = "subagent.complete")]
    SubagentComplete(SubagentEvent),
    #[serde(rename = "message.complete")]
    MessageComplete(MessageComplete),

    #[serde(rename = "thinking.delta")]
    ThinkingDelta(serde_json::Value),

    #[serde(rename = "notice.upsert")]
    NoticeUpsert(serde_json::Value),

    #[serde(rename = "notice.clear")]
    NoticeClear(serde_json::Value),

    #[serde(rename = "notification.show")]
    NotificationShow(serde_json::Value),
    #[serde(rename = "notification.clear")]
    NotificationClear(serde_json::Value),

    // Tools
    #[serde(rename = "tool.start")]
    ToolStart(ToolStart),
    #[serde(rename = "tool.progress")]
    ToolProgress(ToolProgress),
    #[serde(rename = "tool.complete")]
    ToolComplete(ToolComplete),
    #[serde(rename = "tool.generating")]
    ToolGenerating(serde_json::Value),

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

    // Preview
    #[serde(rename = "preview.restart.progress")]
    PreviewRestartProgress(serde_json::Value),
    #[serde(rename = "preview.restart.complete")]
    PreviewRestartComplete(serde_json::Value),

    // Voice
    #[serde(rename = "voice.transcript")]
    VoiceTranscript(serde_json::Value),
    #[serde(rename = "voice.status")]
    VoiceStatus(serde_json::Value),

    // Browser
    #[serde(rename = "browser.progress")]
    BrowserProgress(serde_json::Value),

    // Background
    #[serde(rename = "background.complete")]
    BackgroundComplete(serde_json::Value),

    // Review
    #[serde(rename = "review.summary")]
    ReviewSummary(serde_json::Value),

    // Skin
    #[serde(rename = "skin.changed")]
    SkinChanged(serde_json::Value),

    // Clarify / Sudo / Secret Requests
    #[serde(rename = "clarify.request")]
    ClarifyRequest(ClarifyRequest),
    #[serde(rename = "sudo.request")]
    SudoRequest(SudoRequest),
    #[serde(rename = "secret.request")]
    SecretRequest(SecretRequest),

    // Respond Responses
    #[serde(rename = "session.close")]
    SessionClose(SessionCloseResponse),
    #[serde(rename = "approval.respond")]
    ApprovalRespond(ApprovalRespondResponse),
    #[serde(rename = "config.set")]
    ConfigSet(ConfigSetResponse),
    #[serde(rename = "terminal.resize")]
    TerminalResize(TerminalResizeResponse),
    #[serde(rename = "clarify.respond")]
    ClarifyRespond(ClarifyRespondResponse),
    #[serde(rename = "sudo.respond")]
    SudoRespond(SudoRespondResponse),
    #[serde(rename = "secret.respond")]
    SecretRespond(SecretRespondResponse),
    #[serde(rename = "session.interrupt")]
    SessionInterrupt(SessionInterruptResponse),
    #[serde(rename = "session.most_recent")]
    SessionMostRecent(SessionMostRecentResponse),
    #[serde(rename = "session.delete")]
    SessionDelete(SessionDeleteResponse),
    #[serde(rename = "session.cwd.set")]
    SessionCwdSet(SessionCwdSetResponse),
    #[serde(rename = "model.options")]
    ModelOptions(ModelOptionsResponse),

    // Error
    #[serde(rename = "error")]
    Error(GatewayError),
}

/// Clarify request event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyRequest {
    pub request_id: String,
    pub question: String,
    pub choices: Option<Vec<String>>,
}

/// Sudo request event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SudoRequest {
    pub request_id: String,
}

/// Secret request event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretRequest {
    pub request_id: String,
    pub env_var: String,
    pub prompt: String,
}

/// Clarify response params
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyResponse {
    pub request_id: String,
    pub answer: String,
}

/// Sudo response params
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SudoResponse {
    pub request_id: String,
    pub password: String,
}

/// Secret response params
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretResponse {
    pub request_id: String,
    pub value: String,
}

/// Session close response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionCloseResponse {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub success: Option<bool>,
}

/// Approval respond response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApprovalRespondResponse {
    #[serde(default)]
    pub success: Option<bool>,
}

/// Config set response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigSetResponse {
    #[serde(default)]
    pub success: Option<bool>,
}

/// Terminal resize response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerminalResizeResponse {
    #[serde(default)]
    pub success: Option<bool>,
}

/// Clarify respond response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClarifyRespondResponse {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub ok: Option<bool>,
}

/// Sudo respond response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SudoRespondResponse {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub ok: Option<bool>,
}

/// Secret respond response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecretRespondResponse {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub ok: Option<bool>,
}

/// Session interrupt response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionInterruptResponse {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub ok: Option<bool>,
}

/// Session most recent response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionMostRecentResponse {
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Session delete response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionDeleteResponse {
    #[serde(default)]
    pub deleted: Option<String>,
}

/// Session cwd set response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionCwdSetResponse {
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub success: Option<bool>,
}

/// Model options response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelOptionsResponse {
    #[serde(default)]
    pub providers: Option<Vec<ModelOptionProvider>>,
}

/// Complete slash/path responses
pub type SlashCompletionResponse = CompletionResponse;
pub type PathCompletionResponse = CompletionResponse;

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
    CompleteSlash { text: String },

    #[serde(rename = "complete.path")]
    CompletePath { word: String },

    #[serde(rename = "clarify.respond")]
    ClarifyRespond(ClarifyResponse),

    #[serde(rename = "sudo.respond")]
    SudoRespond(SudoResponse),

    #[serde(rename = "secret.respond")]
    SecretRespond(SecretResponse),

    #[serde(rename = "session.interrupt")]
    SessionInterrupt { session_id: String },

    #[serde(rename = "session.most_recent")]
    SessionMostRecent,

    #[serde(rename = "session.delete")]
    SessionDelete { session_id: String },

    #[serde(rename = "session.cwd.set")]
    SessionCwdSet { session_id: String, cwd: String },

    #[serde(rename = "model.options")]
    ModelOptions,

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
            capabilities: None,
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
            session_id: Some("test-key".to_string()),
            text: "Hello".to_string(),
            rendered: None,
        };

        let serialized = serde_json::to_string(&delta).unwrap();
        let deserialized: MessageDelta = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.session_id, Some("test-key".to_string()));
        assert_eq!(deserialized.text, "Hello");
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
            session_id: Some("test-key".to_string()),
            call_id: "call-123".to_string(),
            result: serde_json::json!({"success": true}),
            duration_ms: None,
            error: None,
        };

        let serialized = serde_json::to_string(&tool).unwrap();
        let deserialized: ToolComplete = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.call_id, "call-123");
        assert_eq!(deserialized.result, serde_json::json!({"success": true}));
        assert_eq!(deserialized.duration_ms, None);
    }

    #[test]
    fn test_secs_to_ms_conversion() {
        // Verify secs_to_ms deserializer: 0.15s → 150ms
        let json = r#"{"tool_id":"t1","result":null,"duration_s":0.15}"#;
        let tool: ToolComplete = serde_json::from_str(json).unwrap();
        assert_eq!(tool.duration_ms, Some(150));
    }

    #[test]
    fn test_tool_progress_deserialization_aliases() {
        let json = r#"{"tool_name":"test-tool","preview":"some output"}"#;
        let progress: ToolProgress = serde_json::from_str(json).unwrap();
        assert_eq!(progress.call_id, "test-tool");
        assert_eq!(progress.output, "some output");

        let json2 = r#"{"name":"test-tool-2","text":"other output"}"#;
        let progress2: ToolProgress = serde_json::from_str(json2).unwrap();
        assert_eq!(progress2.call_id, "test-tool-2");
        assert_eq!(progress2.output, "other output");
    }

    #[test]
    fn test_complete_slash_path_serialization() {
        let slash = TuiRequest::CompleteSlash {
            text: "hello".to_string(),
        };
        let serialized = serde_json::to_string(&slash).unwrap();
        assert!(serialized.contains(r#""text":"hello""#));

        let path = TuiRequest::CompletePath {
            word: "world".to_string(),
        };
        let serialized_path = serde_json::to_string(&path).unwrap();
        assert!(serialized_path.contains(r#""word":"world""#));
    }

    #[test]
    fn test_gateway_ready_missing_fields() {
        let json = r#"{}"#;
        let ready: GatewayReadyResponse = serde_json::from_str(json).unwrap();
        assert!(ready.sessions.is_none());
        assert!(ready.models.is_none());
    }

    #[test]
    fn test_clarify_sudo_secret_serialization() {
        let clarify = GatewayMessage::ClarifyRequest(ClarifyRequest {
            request_id: "req-1".to_string(),
            question: "sure?".to_string(),
            choices: Some(vec!["yes".to_string()]),
        });
        let serialized = serde_json::to_string(&clarify).unwrap();
        let deserialized: GatewayMessage = serde_json::from_str(&serialized).unwrap();
        match deserialized {
            GatewayMessage::ClarifyRequest(req) => {
                assert_eq!(req.request_id, "req-1");
                assert_eq!(req.question, "sure?");
            }
            _ => panic!("Wrong variant"),
        }
    }
}

/// Deserialize gateway's `duration_s` (float seconds) to `Option<u64>` milliseconds
pub fn secs_to_ms<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
    let secs: Option<f64> = Option::deserialize(d)?;
    Ok(secs.map(|s| (s * 1000.0).round() as u64))
}
