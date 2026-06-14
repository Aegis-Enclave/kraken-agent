//! Session picker component for TUI
//!
//! This module provides an overlay dialog centered on screen
//! for choosing, scrolling, and resuming historical chat sessions.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};
use crate::state::config::ChatColorsRgb;
use crate::protocol::types::SessionListItem;

/// Session picker popup component
#[derive(Debug, Clone)]
pub struct SessionPicker {
    /// Active sessions to select from
    sessions: Vec<SessionListItem>,
    /// Visibility state
    visible: bool,
    /// Currently selected index
    selected_index: usize,
    /// UI colors from configuration
    colors: ChatColorsRgb,
}

impl SessionPicker {
    /// Create a new session picker instance
    pub fn new(colors: ChatColorsRgb) -> Self {
        Self {
            sessions: Vec::new(),
            visible: false,
            selected_index: 0,
            colors,
        }
    }

    /// Show the session picker with list of sessions
    pub fn show(&mut self, sessions: Vec<SessionListItem>) {
        self.sessions = sessions;
        self.selected_index = 0;
        self.visible = !self.sessions.is_empty();
    }

    /// Hide the session picker and clear items
    pub fn hide(&mut self) {
        self.visible = false;
        self.sessions.clear();
    }

    /// Check if the session picker is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the currently selected session list item
    pub fn selected_session(&self) -> Option<&SessionListItem> {
        if self.visible && !self.sessions.is_empty() {
            self.sessions.get(self.selected_index)
        } else {
            None
        }
    }

    /// Select the next session in the list
    pub fn select_next(&mut self) {
        if !self.sessions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.sessions.len();
        }
    }

    /// Select the previous session in the list
    pub fn select_prev(&mut self) {
        if !self.sessions.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.sessions.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Render the centered session picker popup
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible || self.sessions.is_empty() {
            return;
        }

        // Center the popup on the screen
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - 60) / 2),
                Constraint::Percentage(60),
                Constraint::Percentage((100 - 60) / 2),
            ])
            .split(area);

        let popup_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - 80) / 2),
                Constraint::Percentage(80),
                Constraint::Percentage((100 - 80) / 2),
            ])
            .split(popup_layout[1])[1];

        // Clear the area to draw overlay cleanly
        frame.render_widget(Clear, popup_area);

        // Build list items
        let list_items: Vec<ListItem> = self
            .sessions
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let formatted_time = chrono::DateTime::from_timestamp(s.started_at, 0)
                    .map_or("Unknown".to_string(), |dt| dt.format("%Y-%m-%d %H:%M").to_string());
                let title = if s.title.is_empty() { "Unnamed" } else { &s.title };
                let source = s.source.as_deref().unwrap_or("cli");
                
                let line1 = format!(
                    "{} (msgs: {}, time: {}, source: {})",
                    title, s.message_count, formatted_time, source
                );
                
                // Crop preview to fit inside popup comfortably
                let max_preview_len = popup_area.width.saturating_sub(6) as usize;
                let preview = if s.preview.len() > max_preview_len {
                    let cropped = &s.preview[..max_preview_len.saturating_sub(3)];
                    format!("{}...", cropped)
                } else {
                    s.preview.clone()
                };

                let item_style = if i == self.selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(self.colors.user_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.colors.user_text)
                };

                let text = vec![
                    ratatui::text::Line::from(line1),
                    ratatui::text::Line::from(format!("  ↳ {}", preview)).style(Style::default().fg(self.colors.timestamp)),
                ];
                
                ListItem::new(text).style(item_style)
            })
            .collect();

        let block = Block::default()
            .title(" Resume Session ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.colors.border));

        let list = List::new(list_items)
            .block(block);

        frame.render_widget(list, popup_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_colors() -> ChatColorsRgb {
        ChatColorsRgb {
            user_bg: Color::Reset,
            user_text: Color::Reset,
            assistant_bg: Color::Reset,
            assistant_text: Color::Reset,
            system_bg: Color::Reset,
            system_text: Color::Reset,
            tool_bg: Color::Reset,
            tool_text: Color::Reset,
            code_bg: Color::Reset,
            code_text: Color::Reset,
            border: Color::Reset,
            timestamp: Color::Reset,
        }
    }

    #[test]
    fn test_session_picker_visibility() {
        let mut picker = SessionPicker::new(create_test_colors());
        assert!(!picker.is_visible());

        let sessions = vec![
            SessionListItem {
                id: "session_1".to_string(),
                title: "Rust coding".to_string(),
                message_count: 5,
                preview: "Implementing key routines...".to_string(),
                started_at: 1_718_000_000,
                source: Some("cli".to_string()),
            },
        ];

        picker.show(sessions);
        assert!(picker.is_visible());

        picker.hide();
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_session_picker_navigation() {
        let mut picker = SessionPicker::new(create_test_colors());
        let sessions = vec![
            SessionListItem {
                id: "session_1".to_string(),
                title: "Rust coding".to_string(),
                message_count: 5,
                preview: "Implementing key routines...".to_string(),
                started_at: 1_718_000_000,
                source: Some("cli".to_string()),
            },
            SessionListItem {
                id: "session_2".to_string(),
                title: "Python gateway".to_string(),
                message_count: 10,
                preview: "Refactoring server code...".to_string(),
                started_at: 1_718_000_100,
                source: Some("cli".to_string()),
            },
        ];

        picker.show(sessions);
        assert_eq!(picker.selected_index, 0);
        assert_eq!(picker.selected_session().unwrap().title, "Rust coding");

        picker.select_next();
        assert_eq!(picker.selected_index, 1);
        assert_eq!(picker.selected_session().unwrap().title, "Python gateway");

        picker.select_next();
        assert_eq!(picker.selected_index, 0);

        picker.select_prev();
        assert_eq!(picker.selected_index, 1);
    }
}
