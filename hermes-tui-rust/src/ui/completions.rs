//! Completions popup component for TUI
//!
//! This module provides a popup dialog for autocomplete suggestions
//! (such as slash commands, paths, etc.) displayed above the input composer.

use crate::protocol::types::CompletionItem;
use crate::state::config::ChatColorsRgb;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

#[derive(Debug, Clone)]
pub struct CompletionPopup {
    /// Autocomplete items
    items: Vec<CompletionItem>,
    /// Visibility state
    visible: bool,
    /// Currently selected index
    selected_index: usize,
    /// List state for scrolling
    state: ListState,
    /// Start position to replace in input
    replace_from: Option<usize>,
    /// UI colors from configuration
    colors: ChatColorsRgb,
}

impl CompletionPopup {
    /// Create a new completion popup instance
    #[must_use]
    pub fn new(colors: ChatColorsRgb) -> Self {
        Self {
            items: Vec::new(),
            visible: false,
            selected_index: 0,
            state: ListState::default(),
            replace_from: None,
            colors,
        }
    }

    /// Show the completion popup with items
    pub fn show(&mut self, items: Vec<CompletionItem>, replace_from: Option<usize>) {
        self.items = items;
        self.replace_from = replace_from;
        self.selected_index = 0;
        self.state.select(Some(0));
        self.visible = !self.items.is_empty();
    }

    /// Hide the completion popup and clear items
    pub fn hide(&mut self) {
        self.visible = false;
        self.items.clear();
        self.replace_from = None;
        self.state.select(None);
    }

    /// Check if the popup is visible
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the replacement start index
    #[must_use]
    pub fn replace_from(&self) -> Option<usize> {
        self.replace_from
    }

    /// Get the currently selected completion item
    #[must_use]
    pub fn selected_item(&self) -> Option<&CompletionItem> {
        if self.visible && !self.items.is_empty() {
            self.items.get(self.selected_index)
        } else {
            None
        }
    }

    /// Select the next item in the completions list
    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.items.len();
            self.state.select(Some(self.selected_index));
        }
    }

    /// Select the previous item in the completions list
    pub fn select_prev(&mut self) {
        if !self.items.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.items.len() - 1;
            } else {
                self.selected_index -= 1;
            }
            self.state.select(Some(self.selected_index));
        }
    }

    /// Render the completion popup centered on the screen
    pub fn render(&mut self, frame: &mut Frame, area: Rect, animation_frame: u64) {
        if !self.visible || self.items.is_empty() {
            return;
        }

        // Center and scale popup: 60% width, 60% height
        let width = (area.width * 60 / 100).max(40).min(area.width);
        let height = (area.height * 60 / 100).max(10).min(area.height);
        let x = area.x + (area.width - width) / 2;
        let y = area.y + (area.height - height) / 2;
        let popup_area = Rect::new(x, y, width, height);

        // Clear the area to overlay the popup cleanly
        frame.render_widget(Clear, popup_area);

        // Build list items
        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let prefix = if i == self.selected_index { "> " } else { "  " };
                let text = if let Some(desc) = &item.meta {
                    format!("{}{}- {}", prefix, item.display, desc)
                } else {
                    format!("{}{}", prefix, item.display)
                };
                let style = if i == self.selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::Rgb(40, 40, 50))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.colors.user_text)
                };
                ListItem::new(text).style(style)
            })
            .collect();

        let block = Block::default()
            .title(" COMPLETIONS ".bold())
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.colors.border));

        let list = List::new(list_items).block(block).highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_stateful_widget(list, popup_area, &mut self.state);

        // Render animated gradient border over the block
        crate::ui::borders::render_gradient_border(
            frame.buffer_mut(),
            popup_area,
            animation_frame,
            true,
            false,
        );

        // Render helper hints
        let hints = Line::from(vec![
            " ↑↓ ".bold().yellow(),
            "navigate".into(),
            " │ ".into(),
            " Enter ".bold().yellow(),
            "apply".into(),
            " │ ".into(),
            " Esc ".bold().yellow(),
            "cancel".into(),
        ])
        .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(
            ratatui::widgets::Paragraph::new(hints),
            Rect::new(
                popup_area.x,
                popup_area.y + popup_area.height - 2,
                popup_area.width,
                1,
            ),
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
    fn test_completion_popup_visibility() {
        let mut popup = CompletionPopup::new(create_test_colors());
        assert!(!popup.is_visible());

        let items = vec![CompletionItem {
            display: "help".to_string(),
            text: "/help".to_string(),
            meta: Some("Show help".to_string()),
        }];

        popup.show(items, None);
        assert!(popup.is_visible());

        popup.hide();
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_completion_popup_navigation() {
        let mut popup = CompletionPopup::new(create_test_colors());
        let items = vec![
            CompletionItem {
                display: "help".to_string(),
                text: "/help".to_string(),
                meta: None,
            },
            CompletionItem {
                display: "quit".to_string(),
                text: "/quit".to_string(),
                meta: None,
            },
        ];

        popup.show(items, None);
        assert_eq!(popup.selected_index, 0);
        assert_eq!(popup.selected_item().unwrap().display, "help");

        popup.select_next();
        assert_eq!(popup.selected_index, 1);
        assert_eq!(popup.selected_item().unwrap().display, "quit");

        popup.select_next();
        assert_eq!(popup.selected_index, 0);

        popup.select_prev();
        assert_eq!(popup.selected_index, 1);
    }
}
