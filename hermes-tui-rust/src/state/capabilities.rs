//! Gateway-reported capabilities displayed in the chat empty state.
//!
//! Populated from `GatewayReadyResponse`; falls back to `Default` when the
//! gateway does not report counts (older gateway versions).

/// Counts and lists used by the chat empty-state landing page.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// Number of tools available to the agent.
    pub tool_count: u16,
    /// Number of skills available to the agent.
    pub skill_count: u16,
    /// Names of connected MCP servers.
    pub mcp_servers: Vec<String>,
    /// Current model name.
    pub model_name: Option<String>,
    /// Current provider name.
    pub provider_name: Option<String>,
    /// Gateway version string.
    pub gateway_version: Option<String>,
}

impl Capabilities {
    /// Construct with explicit counts.
    #[must_use]
    pub fn new(tool_count: u16, skill_count: u16) -> Self {
        Self {
            tool_count,
            skill_count,
            ..Default::default()
        }
    }
}
