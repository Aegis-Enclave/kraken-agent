//! Session module - Session state management
//!
//! This module provides the data structures and operations for managing
//! chat sessions in the TUI.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::protocol::types::{SessionListItem, TokenUsage};
use crate::state::messages::MessageHistory;

/// Session information
#[derive(Debug, Clone)]
pub struct Session {
    /// Unique session ID
    pub id: String,
    /// Session name/title (optional)
    pub name: Option<String>,
    /// Message history for this session
    pub messages: MessageHistory,
    /// When this session was created
    pub created_at: DateTime<Utc>,
    /// When this session was last updated
    pub updated_at: DateTime<Utc>,
    /// Whether this session is currently active
    pub is_active: bool,
    /// Whether the agent is currently running in this session
    pub is_running: bool,
    /// Optional model name for this session
    pub model: Option<String>,
    /// Optional provider name for this session
    pub provider: Option<String>,
    /// Preview text (first few characters of the first message)
    pub preview: Option<String>,
    /// Cumulative token usage for this session
    pub total_usage: TokenUsage,
    /// Cumulative estimated cost for this session (in USD)
    pub total_cost: f64,
}

impl Session {
    /// Create a new session with the given ID
    pub fn new(id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            name: None,
            messages: MessageHistory::new(),
            created_at: now,
            updated_at: now,
            is_active: false,
            is_running: false,
            model: None,
            provider: None,
            preview: None,
            total_usage: TokenUsage::default(),
            total_cost: 0.0,
        }
    }

    /// Create a new session with a name
    pub fn with_name(id: impl Into<String>, name: impl Into<String>) -> Self {
        let mut session = Self::new(id);
        session.name = Some(name.into());
        session.update_preview();
        session
    }

    /// Add a message to this session
    pub fn add_message(&mut self, message: impl Into<crate::state::messages::Message>) {
        self.messages.push(message.into());
        self.updated_at = Utc::now();
        self.update_preview();
    }

    /// Add token usage to this session
    pub fn add_usage(&mut self, usage: &TokenUsage) {
        self.total_usage.prompt_tokens += usage.prompt_tokens;
        self.total_usage.completion_tokens += usage.completion_tokens;
        self.total_usage.total_tokens += usage.total_tokens;

        if let Some(read) = usage.cache_read_tokens {
            self.total_usage.cache_read_tokens =
                Some(self.total_usage.cache_read_tokens.unwrap_or(0) + read);
        }
        if let Some(write) = usage.cache_write_tokens {
            self.total_usage.cache_write_tokens =
                Some(self.total_usage.cache_write_tokens.unwrap_or(0) + write);
        }

        if let Some(prompt_cat) = usage.prompt_category_tokens {
            self.total_usage.prompt_category_tokens =
                Some(self.total_usage.prompt_category_tokens.unwrap_or(0) + prompt_cat);
        }
        if let Some(tool_call) = usage.tool_call_tokens {
            self.total_usage.tool_call_tokens =
                Some(self.total_usage.tool_call_tokens.unwrap_or(0) + tool_call);
        }
        if let Some(reasoning) = usage.reasoning_tokens {
            self.total_usage.reasoning_tokens =
                Some(self.total_usage.reasoning_tokens.unwrap_or(0) + reasoning);
        }
        if let Some(output) = usage.output_tokens {
            self.total_usage.output_tokens =
                Some(self.total_usage.output_tokens.unwrap_or(0) + output);
        }
        if let Some(failed) = usage.failed_tool_call_tokens {
            self.total_usage.failed_tool_call_tokens =
                Some(self.total_usage.failed_tool_call_tokens.unwrap_or(0) + failed);
        }

        // Very rough cost calculation (e.g. $10 per 1M tokens)
        // In a real app, this would be model-specific
        let cost_per_token = 0.00001;
        self.total_cost += usage.total_tokens as f64 * cost_per_token;
    }

    /// Update the preview text based on the first message
    pub fn update_preview(&mut self) {
        if let Some(first_msg) = self.messages.get(0) {
            let content = first_msg.content();
            self.preview = Some(content.chars().take(50).collect());
        }
    }

    /// Get the message count
    #[must_use]
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Clear all messages
    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.preview = None;
    }

    /// Mark this session as active
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// Mark this session as inactive
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    /// Set the running state
    pub fn set_running(&mut self, running: bool) {
        self.is_running = running;
        self.updated_at = Utc::now();
    }

    /// Set the model for this session
    pub fn set_model(&mut self, model: impl Into<String>) {
        self.model = Some(model.into());
    }

    /// Set the provider for this session
    pub fn set_provider(&mut self, provider: impl Into<String>) {
        self.provider = Some(provider.into());
    }

    /// Set the name for this session
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Get the last message
    #[must_use]
    pub fn last_message(&self) -> Option<&crate::state::messages::Message> {
        self.messages.last()
    }

    /// Get all messages
    #[must_use]
    pub fn messages(&self) -> &MessageHistory {
        &self.messages
    }

    /// Get mutable access to messages
    pub fn messages_mut(&mut self) -> &mut MessageHistory {
        &mut self.messages
    }
}

/// Session manager - manages multiple sessions
#[derive(Debug, Default)]
pub struct SessionManager {
    /// All sessions by ID
    sessions: HashMap<String, Session>,
    /// Currently active session ID
    current_session_id: Option<String>,
    /// Next session ID counter (for generating unique IDs)
    next_session_id: u64,
}

impl SessionManager {
    /// Create a new session manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            current_session_id: None,
            next_session_id: 1,
        }
    }

    /// Generate a new unique session ID
    pub fn generate_session_id(&mut self) -> String {
        let id = format!("session-{}", self.next_session_id);
        self.next_session_id += 1;
        id
    }

    /// Create a new session
    pub fn create_session(&mut self) -> &mut Session {
        let id = self.generate_session_id();
        let session = Session::new(id.clone());

        self.sessions.insert(id.clone(), session);
        self.current_session_id = Some(id.clone());

        self.sessions.get_mut(&id).unwrap()
    }

    /// Create a new session with a name
    pub fn create_session_with_name(&mut self, name: impl Into<String>) -> &mut Session {
        let id = self.generate_session_id();
        let session = Session::with_name(id.clone(), name);

        self.sessions.insert(id.clone(), session);
        self.current_session_id = Some(id.clone());

        self.sessions.get_mut(&id).unwrap()
    }

    /// Get the current session
    #[must_use]
    pub fn current_session(&self) -> Option<&Session> {
        self.current_session_id
            .as_ref()
            .and_then(|id| self.sessions.get(id))
    }

    /// Get the current session (mutable)
    pub fn current_session_mut(&mut self) -> Option<&mut Session> {
        self.current_session_id
            .as_ref()
            .and_then(|id| self.sessions.get_mut(id))
    }

    /// Switch to a different session
    pub fn switch_session(&mut self, session_id: &str) -> Option<&mut Session> {
        if self.sessions.contains_key(session_id) {
            // Deactivate current session
            if let Some(current_id) = &self.current_session_id {
                if let Some(current) = self.sessions.get_mut(current_id) {
                    current.deactivate();
                }
            }

            // Activate new session
            self.current_session_id = Some(session_id.to_string());
            if let Some(session) = self.sessions.get_mut(session_id) {
                session.activate();
            }

            self.sessions.get_mut(session_id)
        } else {
            None
        }
    }

    /// Get a session by ID
    #[must_use]
    pub fn get_session(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id)
    }

    /// Get a session by ID (mutable)
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(session_id)
    }

    /// Get all sessions
    #[must_use]
    pub fn sessions(&self) -> &HashMap<String, Session> {
        &self.sessions
    }

    /// Get all session IDs
    #[must_use]
    pub fn session_ids(&self) -> Vec<&String> {
        self.sessions.keys().collect()
    }

    /// Add a session (for restoring from gateway)
    pub fn add_session(&mut self, session: Session) -> Option<Session> {
        self.sessions.insert(session.id.clone(), session)
    }

    /// Remove a session
    pub fn remove_session(&mut self, session_id: &str) -> Option<Session> {
        // Don't allow removing the current session
        if Some(session_id) == self.current_session_id.as_deref() {
            return None;
        }

        self.sessions.remove(session_id)
    }

    /// Clear all sessions
    pub fn clear_all(&mut self) {
        self.sessions.clear();
        self.current_session_id = None;
        self.next_session_id = 1;
    }

    /// Number of sessions
    #[must_use]
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Check if there are no sessions
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    /// Check if a session exists
    #[must_use]
    pub fn contains_session(&self, session_id: &str) -> bool {
        self.sessions.contains_key(session_id)
    }

    /// Get the current session ID
    #[must_use]
    pub fn current_session_id(&self) -> Option<&String> {
        self.current_session_id.as_ref()
    }

    /// Set the current session ID directly
    pub fn set_current_session_id(&mut self, session_id: impl Into<String>) {
        self.current_session_id = Some(session_id.into());
    }

    /// Set the current session (convenience method)
    pub fn set_current_session(&mut self, session_id: impl Into<String>) {
        self.set_current_session_id(session_id);
    }

    /// Clear the current active session (set it to None)
    pub fn clear_current_session(&mut self) {
        self.current_session_id = None;
    }

    /// Set sessions from gateway list response
    pub fn set_sessions(&mut self, session_list: Vec<SessionListItem>) {
        for item in session_list {
            // Check if session already exists
            if !self.sessions.contains_key(&item.id) {
                let mut session = Session::new(&item.id);
                session.set_name(&item.title);
                // Note: SessionListItem doesn't have all Session fields,
                // so we create a basic session and let resume fill in details
                self.sessions.insert(item.id.clone(), session);
            }
        }
    }

    /// Get session list for display (sorted by `updated_at`, newest first)
    #[must_use]
    pub fn session_list(&self) -> Vec<&Session> {
        let mut sessions: Vec<&Session> = self.sessions.values().collect();
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        sessions
    }

    /// Get the most recent session (excluding current)
    #[must_use]
    pub fn most_recent_session(&self) -> Option<&Session> {
        self.session_list()
            .into_iter()
            .find(|s| Some(&s.id) != self.current_session_id.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new("test-session");
        assert_eq!(session.id, "test-session");
        assert!(session.name.is_none());
        assert!(session.messages.is_empty());
        assert!(session.is_active == false);
        assert!(session.is_running == false);
    }

    #[test]
    fn test_session_with_name() {
        let session = Session::with_name("test-session", "Test Session");
        assert_eq!(session.id, "test-session");
        assert_eq!(session.name, Some("Test Session".to_string()));
    }

    #[test]
    fn test_session_add_message() {
        let mut session = Session::new("test-session");

        use crate::state::messages::Message;

        session.add_message(Message::user("Hello"));
        session.add_message(Message::assistant("World"));

        assert_eq!(session.message_count(), 2);
        assert_eq!(session.last_message().unwrap().content(), "World");
    }

    #[test]
    fn test_session_activate_deactivate() {
        let mut session = Session::new("test-session");

        assert!(!session.is_active);

        session.activate();
        assert!(session.is_active);

        session.deactivate();
        assert!(!session.is_active);
    }

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert!(manager.is_empty());
        assert!(manager.current_session_id().is_none());
    }

    #[test]
    fn test_session_manager_create_session() {
        let mut manager = SessionManager::new();

        let session = manager.create_session();
        assert_eq!(session.id, "session-1");
        assert!(manager.current_session_id().is_some());
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_session_manager_switch_session() {
        let mut manager = SessionManager::new();

        let _session1 = manager.create_session();
        let session2 = manager.create_session_with_name("Session 2");
        let session2_id = session2.id.clone();

        assert!(manager.switch_session(&session2_id).is_some());
        assert_eq!(manager.current_session_id(), Some(&session2_id));
    }

    #[test]
    fn test_session_manager_session_list() {
        let mut manager = SessionManager::new();

        manager.create_session();
        manager.create_session();
        manager.create_session();

        let list = manager.session_list();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_session_manager_most_recent() {
        let mut manager = SessionManager::new();

        manager.create_session_with_name("Session 1");
        manager.create_session_with_name("Session 2");

        let most_recent = manager.most_recent_session();
        assert!(most_recent.is_some());
    }
}
