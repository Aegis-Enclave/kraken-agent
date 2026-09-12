//! Session picker component for TUI
//!
//! This module provides an overlay dialog centered on screen
//! for choosing, scrolling, and resuming historical chat sessions.

use crate::protocol::types::SessionListItem;
use crate::state::config::ChatColorsRgb;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

#[derive(Debug, Clone)]
pub struct SessionPicker {
    /// Active sessions to select from
    sessions: Vec<SessionListItem>,
    /// Filtered indices into `sessions`
    filtered_indices: Vec<usize>,
    /// Visibility state
    visible: bool,
    /// Currently selected index (into `filtered_indices`)
    selected_index: usize,
    /// UI colors from configuration
    colors: ChatColorsRgb,
    /// Filter query for searching sessions
    filter: String,
}

impl SessionPicker {
    /// Create a new session picker instance
    #[must_use]
    pub fn new(colors: ChatColorsRgb) -> Self {
        Self {
            sessions: Vec::new(),
            filtered_indices: Vec::new(),
            visible: false,
            selected_index: 0,
            colors,
            filter: String::new(),
        }
    }

    /// Show the session picker with list of sessions
    pub fn show(&mut self, sessions: Vec<SessionListItem>) {
        self.sessions = sessions;
        self.filter.clear();
        self.apply_filter();
        self.selected_index = 0;
        self.visible = !self.filtered_indices.is_empty();
    }

    /// Hide the session picker and clear items
    pub fn hide(&mut self) {
        self.visible = false;
        self.sessions.clear();
        self.filtered_indices.clear();
        self.filter.clear();
    }

    /// Append a character to the filter query
    pub fn append_filter(&mut self, c: char) {
        self.filter.push(c);
        self.apply_filter();
    }

    /// Remove the last character from the filter query
    pub fn pop_filter(&mut self) {
        self.filter.pop();
        self.apply_filter();
    }

    /// Clear the filter query
    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.apply_filter();
    }

    fn apply_filter(&mut self) {
        let query = self.filter.to_lowercase();
        self.filtered_indices = self
            .sessions
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                if query.is_empty() {
                    return true;
                }
                s.title.to_lowercase().contains(&query)
                    || s.preview.to_lowercase().contains(&query)
                    || s.id.to_lowercase().contains(&query)
            })
            .map(|(i, _)| i)
            .collect();
        if self.selected_index >= self.filtered_indices.len() {
            self.selected_index = 0;
        }
    }
    /// Check if the session picker is visible
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the currently selected session list item
    #[must_use]
    pub fn selected_session(&self) -> Option<&SessionListItem> {
        if self.visible && !self.filtered_indices.is_empty() {
            self.filtered_indices
                .get(self.selected_index)
                .and_then(|&idx| self.sessions.get(idx))
        } else {
            None
        }
    }

    /// Select the next session in the list
    pub fn select_next(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_indices.len();
        }
    }

    /// Select the previous session in the list
    pub fn select_prev(&mut self) {
        if !self.filtered_indices.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.filtered_indices.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Current filter query
    #[must_use]
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Render the centered session picker popup
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        animation_frame: u64,
        current_session_id: Option<&str>,
        is_running: bool,
    ) {
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
            .filtered_indices
            .iter()
            .enumerate()
            .map(|(list_i, &session_i)| {
                let s = &self.sessions[session_i];
                let is_active = current_session_id.map_or(false, |id| id == s.id);
                let is_selected = list_i == self.selected_index;

                let active_indicator = if is_active { "◈" } else { "◇" };
                let select_indicator = if is_selected { ">" } else { " " };

                let formatted_time = chrono::DateTime::from_timestamp(s.started_at, 0)
                    .map_or("Unknown".to_string(), |dt| {
                        dt.format("%Y-%m-%d %H:%M").to_string()
                    });
                let title = if s.title.is_empty() {
                    "Unnamed"
                } else {
                    &s.title
                };
                let source = s.source.as_deref().unwrap_or("cli");

                let line1 = format!(
                    "{} {} {} (msgs: {}, time: {}, source: {})",
                    select_indicator,
                    active_indicator,
                    title,
                    s.message_count,
                    formatted_time,
                    source
                );

                // Crop preview to fit inside popup comfortably
                let max_preview_len = popup_area.width.saturating_sub(6) as usize;
                let preview = if s.preview.len() > max_preview_len {
                    let cropped = &s.preview[..max_preview_len.saturating_sub(3)];
                    format!("{cropped}...")
                } else {
                    s.preview.clone()
                };

                let item_style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(self.colors.user_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.colors.user_text)
                };

                let text = vec![
                    ratatui::text::Line::from(line1),
                    ratatui::text::Line::from(format!("    ↳ {preview}"))
                        .style(Style::default().fg(self.colors.timestamp)),
                ];

                ListItem::new(text).style(item_style)
            })
            .collect();

        let title = if self.filter.is_empty() {
            " Resume Session ".to_string()
        } else {
            format!(" Resume Session | filter: {} ", self.filter)
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.colors.border));

        let list = List::new(list_items).block(block);

        frame.render_widget(list, popup_area);

        // Render animated gradient border over the block
        crate::ui::borders::render_gradient_border(
            frame.buffer_mut(),
            popup_area,
            animation_frame,
            true,
            is_running,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_colors() -> ChatColorsRgb {
        ChatColorsRgb::default()
    }

    #[test]
    fn test_session_picker_visibility() {
        let mut picker = SessionPicker::new(create_test_colors());
        assert!(!picker.is_visible());

        let sessions = vec![SessionListItem {
            id: "session_1".to_string(),
            title: "Rust coding".to_string(),
            message_count: 5,
            preview: "Implementing key routines...".to_string(),
            started_at: 1_718_000_000,
            source: Some("cli".to_string()),
        }];

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
