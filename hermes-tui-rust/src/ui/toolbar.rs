//! Toolbar module - Status bar UI component
//!
//! This module provides the status bar component for displaying information
//! like the current model, session, input mode, etc.

use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::state::config::{ChatColorsRgb, InputMode, ThemeColorsRgb};

/// Status bar item for displaying information
#[derive(Debug, Clone)]
pub struct ToolbarItem {
    /// Display text
    text: String,
    /// Style for this item
    style: Style,
}

impl ToolbarItem {
    /// Create a new toolbar item
    pub fn new(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    /// Create a new toolbar item with default style
    pub fn with_defaults(text: impl Into<String>) -> Self {
        Self::new(text, Style::new())
    }
}

/// Toolbar component for displaying status information
///
/// This component shows information like:
/// - Current model
/// - Current session
/// - Input mode
/// - Connection status
/// - Time
#[derive(Debug, Clone)]
pub struct Toolbar {
    /// List of items to display (left to right)
    items: Vec<ToolbarItem>,
    /// Theme colors from configuration
    theme_colors: ThemeColorsRgb,
    /// Chat colors from configuration (for border)
    chat_colors: ChatColorsRgb,
    /// Current input mode
    input_mode: InputMode,
    /// Whether the gateway is thinking
    is_thinking: bool,
    /// Current spinner index
    spinner_idx: usize,
    /// Current thinking verb index
    verb_idx: usize,
}

impl Toolbar {
    /// Create a new toolbar with the given colors
    pub fn new(theme_colors: ThemeColorsRgb, chat_colors: ChatColorsRgb) -> Self {
        Self {
            items: Vec::new(),
            theme_colors,
            chat_colors,
            input_mode: InputMode::Normal,
            is_thinking: false,
            spinner_idx: 0,
            verb_idx: 0,
        }
    }

    /// Create a new toolbar with all defaults
    pub fn with_defaults() -> Self {
        Self::new(
            ThemeColorsRgb {
                background: ratatui::style::Color::Reset,
                text: ratatui::style::Color::Reset,
                primary: ratatui::style::Color::Indexed(13),
                secondary: ratatui::style::Color::Indexed(14),
                accent: ratatui::style::Color::Indexed(11),
                error: ratatui::style::Color::Red,
                success: ratatui::style::Color::Green,
                warning: ratatui::style::Color::Yellow,
            },
            ChatColorsRgb {
                user_bg: ratatui::style::Color::Indexed(238),
                user_text: ratatui::style::Color::Indexed(252),
                assistant_bg: ratatui::style::Color::Indexed(236),
                assistant_text: ratatui::style::Color::Indexed(248),
                system_bg: ratatui::style::Color::Indexed(235),
                system_text: ratatui::style::Color::Indexed(245),
                tool_bg: ratatui::style::Color::Indexed(237),
                tool_text: ratatui::style::Color::Indexed(243),
                code_bg: ratatui::style::Color::Indexed(233),
                code_text: ratatui::style::Color::Indexed(252),
                border: ratatui::style::Color::Indexed(240),
                timestamp: ratatui::style::Color::Indexed(246),
            },
        )
    }

    /// Set the input mode
    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
    }

    /// Set the theme colors
    pub fn set_theme_colors(&mut self, colors: ThemeColorsRgb) {
        self.theme_colors = colors;
    }

    /// Set the chat colors
    pub fn set_chat_colors(&mut self, colors: ChatColorsRgb) {
        self.chat_colors = colors;
    }

    /// Add an item to the toolbar
    pub fn add_item(&mut self, item: ToolbarItem) {
        self.items.push(item);
    }

    /// Add a text item with the given style
    pub fn add_text(&mut self, text: impl Into<String>, style: Style) {
        self.items.push(ToolbarItem::new(text, style));
    }

    /// Add a text item with default style
    pub fn add_default_text(&mut self, text: impl Into<String>) {
        self.items.push(ToolbarItem::with_defaults(text));
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Get the input mode
    pub fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    /// Update thinking status and increment spinner
    pub fn tick(&mut self, is_thinking: bool) {
        self.is_thinking = is_thinking;
        if is_thinking {
            self.spinner_idx = self.spinner_idx.wrapping_add(1);
            // Change verb every 20 ticks
            if self.spinner_idx % 20 == 0 {
                self.verb_idx = self.verb_idx.wrapping_add(1);
            }
        }
    }

    /// Update the status based on connection state and current mode
    pub fn update_status(&mut self, connected: bool, model: Option<&str>, session: Option<&str>) {
        self.items.clear();

        // Add connection status
        if connected {
            self.add_text(
                "● ".to_string(),
                Style::new().fg(self.theme_colors.success),
            );
        } else {
            self.add_text(
                "○ ".to_string(),
                Style::new().fg(self.theme_colors.error),
            );
        }
        
        // Add model name
        if let Some(model_name) = model {
            self.add_text(
                format!("Model: {} ", model_name),
                Style::new().fg(self.theme_colors.text),
            );
        }
        
        // Add session name
        if let Some(session_name) = session {
            self.add_text(
                format!("Session: {} ", session_name),
                Style::new().fg(self.theme_colors.text),
            );
        }
        
        // Add input mode
        let mode_text = match self.input_mode {
            InputMode::Normal => "Normal",
            InputMode::Insert => "Insert",
            InputMode::Command => "Command",
        };
        self.add_text(
            format!("Mode: {} ", mode_text),
            Style::new().fg(self.theme_colors.accent),
        );
    }

    /// Render the toolbar with Powerline-style segments
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if area.height == 0 {
            return;
        }

        // Colors
        let bg_main = Color::Rgb(27, 29, 30);   // #1B1D1E
        let bg_seg1 = Color::Rgb(117, 113, 94); // #75715E (Dev)
        let bg_seg2 = Color::Rgb(73, 72, 62);   // #49483E (Status)
        let fg_text = Color::Rgb(248, 248, 242); // #F8F8F2

        // Background
        frame.render_widget(Block::default().style(Style::default().bg(bg_main)), area);

        let mut left_spans = Vec::new();

        // 1. Env Segment
        left_spans.push(Span::styled(" Dev ", Style::default().bg(bg_seg1).fg(bg_main).bold()));
        left_spans.push(Span::styled("", Style::default().fg(bg_seg1).bg(bg_seg2)));

        // 2. Connection Status
        let (status_icon, status_color) = if self.items.iter().any(|i| i.text.contains("●")) {
            (" ● ", Color::Rgb(166, 226, 46))
        } else {
            (" ○ ", Color::Rgb(249, 38, 114))
        };
        left_spans.push(Span::styled(status_icon, Style::default().bg(bg_seg2).fg(status_color).bold()));
        left_spans.push(Span::styled("", Style::default().fg(bg_seg2).bg(bg_main)));

        // 3. Thinking Indicator (if active)
        if self.is_thinking {
            let faces = ["(≡)", "(≌)", "(‿)", "(◈)", "(Ψ)", "(🦑)"];
            let verbs = [
                "stirring the abyss", "unfurling tentacles", "charting deep currents",
                "inking the void", "tangling with the unknown", "sounding the trench",
                "coiling for strike", "reading pressure ridges", "glowing in the dark",
                "shedding a bioluminescent tear", "befriending a gulper eel",
                "counting jellyfish", "spiraling downward", "mapping the sea floor",
                "teasing the leviathan", "whispering to barnacles",
            ];
            let face = faces[self.spinner_idx % faces.len()];
            let verb = verbs[self.verb_idx % verbs.len()];
            
            left_spans.push(Span::styled(format!(" {} {}... ", face, verb), Style::default().fg(Color::Rgb(166, 226, 46)).bg(bg_main).bold().italic()));
            left_spans.push(Span::styled("", Style::default().fg(bg_seg2)));
        }

        // 4. Model/Session Info
        for item in &self.items {
            let text = item.text.trim();
            if text.contains("●") || text.contains("○") || text.is_empty() {
                continue;
            }
            left_spans.push(Span::styled(format!(" {} ", text), Style::default().fg(fg_text).bg(bg_main)));
            left_spans.push(Span::styled(" ", Style::default().fg(bg_seg2)));
        }

        // Right Info
        let clock = chrono::Local::now().format("%H:%M").to_string();
        let right_text = format!("  100%  {} ", clock);
        let right_span = Span::styled(right_text, Style::default().fg(bg_seg1).bg(bg_main));

        let left_line = Line::from(left_spans);
        let left_width = left_line.width();
        let right_width = right_span.width();

        let mut final_spans = left_line.spans;
        if area.width > (left_width + right_width) as u16 {
            let padding = " ".repeat(area.width as usize - left_width - right_width);
            final_spans.push(Span::raw(padding));
            final_spans.push(right_span);
        } else if area.width > left_width as u16 {
            // Squeeze padding
            let padding = " ".repeat(area.width.saturating_sub(left_width as u16) as usize);
            final_spans.push(Span::raw(padding));
        }

        frame.render_widget(Paragraph::new(Line::from(final_spans)).style(Style::default().bg(bg_main)), area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_theme_colors() -> ThemeColorsRgb {
        ThemeColorsRgb {
            background: ratatui::style::Color::Reset,
            text: ratatui::style::Color::Indexed(252),
            primary: ratatui::style::Color::Indexed(13),
            secondary: ratatui::style::Color::Indexed(14),
            accent: ratatui::style::Color::Indexed(11),
            error: ratatui::style::Color::Red,
            success: ratatui::style::Color::Green,
            warning: ratatui::style::Color::Yellow,
        }
    }

    fn create_test_chat_colors() -> ChatColorsRgb {
        ChatColorsRgb {
            user_bg: ratatui::style::Color::Indexed(238),
            user_text: ratatui::style::Color::Indexed(252),
            assistant_bg: ratatui::style::Color::Indexed(236),
            assistant_text: ratatui::style::Color::Indexed(248),
            system_bg: ratatui::style::Color::Indexed(235),
            system_text: ratatui::style::Color::Indexed(245),
            tool_bg: ratatui::style::Color::Indexed(237),
            tool_text: ratatui::style::Color::Indexed(243),
            code_bg: ratatui::style::Color::Indexed(233),
            code_text: ratatui::style::Color::Indexed(252),
            border: ratatui::style::Color::Indexed(240),
            timestamp: ratatui::style::Color::Indexed(246),
        }
    }

    #[test]
    fn test_toolbar_new() {
        let theme_colors = create_test_theme_colors();
        let chat_colors = create_test_chat_colors();
        let toolbar = Toolbar::new(theme_colors, chat_colors);
        assert!(toolbar.items.is_empty());
    }

    #[test]
    fn test_toolbar_with_defaults() {
        let toolbar = Toolbar::with_defaults();
        assert!(toolbar.items.is_empty());
    }

    #[test]
    fn test_toolbar_add_item() {
        let theme_colors = create_test_theme_colors();
        let chat_colors = create_test_chat_colors();
        let mut toolbar = Toolbar::new(theme_colors, chat_colors);
        
        let item = ToolbarItem::new("Test", Style::new());
        toolbar.add_item(item);
        
        assert_eq!(toolbar.items.len(), 1);
    }

    #[test]
    fn test_toolbar_add_text() {
        let theme_colors = create_test_theme_colors();
        let chat_colors = create_test_chat_colors();
        let mut toolbar = Toolbar::new(theme_colors, chat_colors);
        
        toolbar.add_text("Test", Style::new().fg(ratatui::style::Color::Red));
        
        assert_eq!(toolbar.items.len(), 1);
    }

    #[test]
    fn test_toolbar_clear() {
        let theme_colors = create_test_theme_colors();
        let chat_colors = create_test_chat_colors();
        let mut toolbar = Toolbar::new(theme_colors, chat_colors);
        
        toolbar.add_default_text("Test 1");
        toolbar.add_default_text("Test 2");
        assert_eq!(toolbar.items.len(), 2);
        
        toolbar.clear();
        assert!(toolbar.items.is_empty());
    }

    #[test]
    fn test_toolbar_input_mode() {
        let theme_colors = create_test_theme_colors();
        let chat_colors = create_test_chat_colors();
        let mut toolbar = Toolbar::new(theme_colors, chat_colors);
        
        assert_eq!(toolbar.input_mode(), InputMode::Normal);
        
        toolbar.set_input_mode(InputMode::Insert);
        assert_eq!(toolbar.input_mode(), InputMode::Insert);
        
        toolbar.set_input_mode(InputMode::Command);
        assert_eq!(toolbar.input_mode(), InputMode::Command);
    }

    #[test]
    fn test_toolbar_update_status() {
        let theme_colors = create_test_theme_colors();
        let chat_colors = create_test_chat_colors();
        let mut toolbar = Toolbar::new(theme_colors, chat_colors);
        
        toolbar.update_status(true, Some("gpt-4"), Some("session-1"));
        
        // Should have connection indicator + model + session + mode
        assert!(toolbar.items.len() >= 4);
    }

    #[test]
    fn test_toolbar_item_new() {
        let item = ToolbarItem::new("Test", Style::new());
        assert_eq!(item.text, "Test");
    }

    #[test]
    fn test_toolbar_item_with_defaults() {
        let item = ToolbarItem::with_defaults("Test");
        assert_eq!(item.text, "Test");
    }
}
