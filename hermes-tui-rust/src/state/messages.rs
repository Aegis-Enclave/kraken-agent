//! Messages module - Message history and management
//!
//! This module provides the data structures and operations for managing
//! the conversation message history.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::protocol::types::MessageRole;

/// Maximum number of messages to keep in history (to prevent unbounded memory usage)
pub const MAX_MESSAGE_HISTORY: usize = 1000;

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// Message role (user, assistant, system, tool)
    pub role: MessageRole,
    /// Message content
    pub content: String,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    /// Optional message ID (assigned by gateway)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// Optional context (e.g., tool name for tool messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Optional name (e.g., tool name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether this message is part of a streaming response
    #[serde(default)]
    pub streaming: bool,
    /// Whether this message is complete
    #[serde(default)]
    pub complete: bool,
}

impl Message {
    /// Create a new message with the given role and content
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            timestamp: Utc::now(),
            message_id: None,
            context: None,
            name: None,
            streaming: false,
            complete: true,
        }
    }

    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(MessageRole::User, content)
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content)
    }

    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(MessageRole::System, content)
    }

    /// Create a new tool message
    pub fn tool(content: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            timestamp: Utc::now(),
            message_id: None,
            context: None,
            name: Some(name.into()),
            streaming: false,
            complete: true,
        }
    }

    /// Create a streaming message (delta)
    pub fn streaming_delta(role: MessageRole, content: impl Into<String>, message_id: Option<String>) -> Self {
        Self {
            role,
            content: content.into(),
            timestamp: Utc::now(),
            message_id,
            context: None,
            name: None,
            streaming: true,
            complete: false,
        }
    }

    /// Mark this message as complete
    pub fn mark_complete(&mut self) {
        self.complete = true;
        self.streaming = false;
    }

    /// Check if this is a user message
    pub fn is_user(&self) -> bool {
        self.role == MessageRole::User
    }

    /// Check if this is an assistant message
    pub fn is_assistant(&self) -> bool {
        self.role == MessageRole::Assistant
    }

    /// Check if this is a system message
    pub fn is_system(&self) -> bool {
        self.role == MessageRole::System
    }

    /// Check if this is a tool message
    pub fn is_tool(&self) -> bool {
        self.role == MessageRole::Tool
    }

    /// Check if this is a tool message containing file edits
    /// (name contains "edit", "write", "patch")
    pub fn is_edit_tool_message(&self) -> bool {
        if !self.is_tool() {
            return false;
        }
        self.name.as_deref().map_or(false, |n| {
            let n_lower = n.to_lowercase();
            n_lower.contains("edit") || n_lower.contains("write") || n_lower.contains("patch")
        })
    }

    /// Check if this message is still streaming
    pub fn is_streaming(&self) -> bool {
        self.streaming
    }

    /// Get the message content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Set the message ID
    pub fn set_message_id(&mut self, id: impl Into<String>) {
        self.message_id = Some(id.into());
    }

    /// Append content to this message (for streaming)
    pub fn append_content(&mut self, delta: &str) {
        self.content.push_str(delta);
    }
}

/// Message history manager
///
/// This struct manages the conversation message history with a maximum size
/// to prevent unbounded memory usage.
#[derive(Debug, Clone)]
pub struct MessageHistory {
    /// Messages stored in order (oldest first, newest last)
    messages: VecDeque<Message>,
    /// Maximum number of messages to keep
    max_messages: usize,
    /// Current session ID
    session_id: Option<String>,
}

impl MessageHistory {
    /// Create a new message history with the default maximum size
    pub fn new() -> Self {
        Self::with_capacity(MAX_MESSAGE_HISTORY)
    }

    /// Create a new message history with a custom maximum size
    pub fn with_capacity(max_messages: usize) -> Self {
        Self {
            messages: VecDeque::with_capacity(max_messages),
            max_messages,
            session_id: None,
        }
    }

    /// Set the current session ID
    pub fn set_session_id(&mut self, session_id: impl Into<String>) {
        self.session_id = Some(session_id.into());
    }

    /// Get the current session ID
    pub fn session_id(&self) -> Option<&String> {
        self.session_id.as_ref()
    }

    /// Add a message to the history
    pub fn push(&mut self, message: Message) {
        // If we're at capacity, remove the oldest message
        if self.messages.len() >= self.max_messages {
            self.messages.pop_front();
        }
        
        self.messages.push_back(message);
    }
    
    /// Add a message to the history (alias for push)
    pub fn add_message(&mut self, message: Message) {
        self.push(message);
    }
    
    /// Get all messages as a vector (owned)
    pub fn all_messages(&self) -> Vec<Message> {
        self.messages.iter().cloned().collect()
    }

    /// Add content to the last message (for streaming deltas)
    pub fn append_to_last(&mut self, delta: &str) -> bool {
        if let Some(last) = self.messages.back_mut() {
            last.append_content(delta);
            true
        } else {
            false
        }
    }

    /// Mark the last message as complete
    pub fn mark_last_complete(&mut self) -> bool {
        if let Some(last) = self.messages.back_mut() {
            last.mark_complete();
            true
        } else {
            false
        }
    }

    /// Get a message by index
    pub fn get(&self, index: usize) -> Option<&Message> {
        self.messages.get(index)
    }

    /// Get the last message
    pub fn last(&self) -> Option<&Message> {
        self.messages.back()
    }

    /// Check if the last message is currently streaming
    pub fn has_streaming_message(&self) -> bool {
        self.last().map_or(false, |m| m.is_streaming())
    }

    /// Get the last message (mutable)
    pub fn last_mut(&mut self) -> Option<&mut Message> {
        self.messages.back_mut()
    }

    /// Get all messages
    pub fn messages(&self) -> &VecDeque<Message> {
        &self.messages
    }

    /// Get messages as a vector (for iteration)
    pub fn to_vec(&self) -> Vec<&Message> {
        self.messages.iter().collect()
    }

    /// Number of messages in history
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Remove the last message
    pub fn pop_last(&mut self) -> Option<Message> {
        self.messages.pop_back()
    }

    /// Get messages in a range
    pub fn range(&self, start: usize, end: usize) -> Vec<&Message> {
        self.messages.range(start..end.min(self.messages.len())).collect()
    }

    /// Get the most recent N messages
    pub fn last_n(&self, n: usize) -> Vec<&Message> {
        let start = self.messages.len().saturating_sub(n);
        self.range(start, self.messages.len())
    }

    /// Find a message by message ID
    pub fn find_by_id(&self, message_id: &str) -> Option<&Message> {
        self.messages.iter().find(|m| m.message_id.as_deref() == Some(message_id))
    }

    /// Update or create a message by message ID
    pub fn upsert_by_id(&mut self, message: Message) {
        // If message has an ID, try to update existing message
        if let Some(id) = &message.message_id {
            if let Some(existing) = self.messages.iter_mut().find(|m| m.message_id.as_deref() == Some(id)) {
                *existing = message;
                return;
            }
        }
        
        // Otherwise, just push the message
        self.push(message);
    }

    /// Check if a message with the given ID exists
    pub fn contains_id(&self, message_id: &str) -> bool {
        self.messages.iter().any(|m| m.message_id.as_deref() == Some(message_id))
    }
    /// Update a message's content by message ID
    pub fn update_message_by_id(&mut self, message_id: &str, content: String) -> Option<Message> {
        if let Some(msg) = self.messages.iter_mut().find(|m| m.message_id.as_deref() == Some(message_id)) {
            msg.content = content;
            Some(msg.clone())
        } else {
            None
        }
    }
}

impl Default for MessageHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl std::iter::IntoIterator for MessageHistory {
    type Item = Message;
    type IntoIter = std::collections::vec_deque::IntoIter<Message>;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.into_iter()
    }
}

impl<'a> IntoIterator for &'a MessageHistory {
    type Item = &'a Message;
    type IntoIter = std::collections::vec_deque::Iter<'a, Message>;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello, world!");
        assert!(msg.is_user());
        assert_eq!(msg.content(), "Hello, world!");
        assert!(!msg.is_streaming());
        assert!(msg.complete);
    }

    #[test]
    fn test_assistant_message() {
        let msg = Message::assistant("Hello from assistant!");
        assert!(msg.is_assistant());
        assert_eq!(msg.content(), "Hello from assistant!");
    }

    #[test]
    fn test_tool_message() {
        let msg = Message::tool("Tool output", "search_files");
        assert!(msg.is_tool());
        assert_eq!(msg.content(), "Tool output");
        assert_eq!(msg.name, Some("search_files".to_string()));
    }

    #[test]
    fn test_message_history_new() {
        let history = MessageHistory::new();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_message_history_push() {
        let mut history = MessageHistory::new();
        
        history.push(Message::user("First message"));
        history.push(Message::assistant("Second message"));
        
        assert_eq!(history.len(), 2);
        assert!(!history.is_empty());
    }

    #[test]
    fn test_message_history_capacity() {
        let mut history = MessageHistory::with_capacity(3);
        
        history.push(Message::user("1"));
        history.push(Message::user("2"));
        history.push(Message::user("3"));
        history.push(Message::user("4"));
        
        // Should only have 3 messages (oldest removed)
        assert_eq!(history.len(), 3);
        assert_eq!(history.get(0).unwrap().content(), "2");
        assert_eq!(history.get(1).unwrap().content(), "3");
        assert_eq!(history.get(2).unwrap().content(), "4");
    }

    #[test]
    fn test_message_history_contains_id() {
        let mut history = MessageHistory::new();
        
        let msg = Message {
            role: MessageRole::Assistant,
            content: "Test".to_string(),
            timestamp: Utc::now(),
            message_id: Some("msg-123".to_string()),
            context: None,
            name: None,
            streaming: false,
            complete: true,
        };
        
        history.push(msg);
        
        assert!(history.contains_id("msg-123"));
        assert!(!history.contains_id("msg-456"));
    }
}

