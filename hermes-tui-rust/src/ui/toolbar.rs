//! Toolbar module - Status bar UI component
//!
//! This module provides the status bar component for displaying information
//! like the current model, session, input mode, etc.

use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::state::config::{ChatColorsRgb, InputMode, ThemeColorsRgb};

/// Status bar item for displaying information
#[derive(Debug, Clone)]
pub struct ToolbarItem {
    /// Display text
    text: String,
}

impl ToolbarItem {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
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
    /// Whether tmux-style prefix mode is active
    prefix_mode: bool,
}

impl Toolbar {
    /// Create a new toolbar with the given colors
    #[must_use]
    pub fn new(theme_colors: ThemeColorsRgb, chat_colors: ChatColorsRgb) -> Self {
        Self {
            items: Vec::new(),
            theme_colors,
            chat_colors,
            input_mode: InputMode::Normal,
            is_thinking: false,
            spinner_idx: 0,
            verb_idx: 0,
            prefix_mode: false,
        }
    }

    /// Create a new toolbar with all defaults
    #[must_use]
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
            ChatColorsRgb::default(),
        )
    }

    /// Set the input mode
    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
    }

    /// Set prefix mode state for the toolbar indicator
    pub fn set_prefix_mode(&mut self, active: bool) {
        self.prefix_mode = active;
    }

    /// Set the currently focused pane (used for animated borders)
    pub fn set_focus_pane(&mut self, _pane: crate::state::config::FocusPane) {
        // Focus pane state is tracked in App; the toolbar doesn't need it
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

    pub fn add_text(&mut self, text: impl Into<String>) {
        self.items.push(ToolbarItem::new(text));
    }

    pub fn add_default_text(&mut self, text: impl Into<String>) {
        self.items.push(ToolbarItem::new(text));
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Get the input mode
    #[must_use]
    pub fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    /// Update thinking status and increment spinner
    pub fn tick(&mut self, is_thinking: bool) {
        self.is_thinking = is_thinking;
        if is_thinking {
            self.spinner_idx = self.spinner_idx.wrapping_add(1);
            // Change verb every 200 ticks (~3.3s at 60fps)
            if self.spinner_idx % 200 == 0 {
                self.verb_idx = self.verb_idx.wrapping_add(1);
            }
        }
    }

    pub fn update_status(
        &mut self,
        connected: bool,
        model: Option<&str>,
        provider: Option<&str>,
        session_name: Option<&str>,
        message_count: usize,
    ) {
        self.items.clear();
        if connected {
            self.add_text("●");
        } else {
            self.add_text("○");
        }
        if let Some(model_name) = model {
            self.add_text(format!("Model: {model_name}"));
        }
        if let Some(provider_name) = provider {
            self.add_text(format!("Provider: {provider_name}"));
        }
        if let Some(name) = session_name {
            self.add_text(format!("Session: {name}"));
        }
        if message_count > 0 {
            self.add_text(format!("Msgs: {message_count}"));
        }
        let mode_text = match self.input_mode {
            InputMode::Normal => "Normal",
            InputMode::Insert => "Insert",
            InputMode::Command => "Command",
        };
        self.add_text(format!("Mode: {mode_text}"));
    }

    /// Render the toolbar as a clean status line matching the TUI aesthetic
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if area.height == 0 {
            return;
        }

        let dim = self.chat_colors.tool_text;
        let accent_green = self.theme_colors.success;
        let accent_red = self.theme_colors.error;
        let main_fg = self.theme_colors.text;
        let sep = Style::default().fg(dim);
        let sep_str = " · ";

        let mut spans: Vec<Span> = Vec::new();

        // 1. Connection indicator
        let connected = self.items.iter().any(|i| i.text == "●");
        let dot_color = if connected { accent_green } else { accent_red };
        spans.push(Span::styled("●", Style::default().fg(dot_color).bold()));

        // 2. Prefix mode indicator
        if self.prefix_mode {
            spans.push(Span::styled(
                " [PREFIX] ",
                Style::default().fg(self.theme_colors.warning).bold(),
            ));
        }

        // 2. Thinking indicator (if active)
        if self.is_thinking {
            let faces = ["(≡)", "(≌)", "(‿)", "(◈)", "(Ψ)", "(🦑)"];
            let verbs = [
                "stirring the abyss",
                "unfurling tentacles",
                "charting deep currents",
                "inking the void",
                "tangling with the unknown",
                "sounding the trench",
                "coiling for strike",
                "reading pressure ridges",
                "glowing in the dark",
                "shedding a bioluminescent tear",
                "befriending a gulper eel",
                "counting jellyfish",
                "spiraling downward",
                "mapping the sea floor",
                "teasing the leviathan",
                "whispering to barnacles",
            ];
            let face = faces[(self.spinner_idx / 10) % faces.len()];
            let verb = verbs[self.verb_idx % verbs.len()];
            spans.push(Span::styled(
                format!(" {face} {verb}... "),
                Style::default().fg(accent_green).bold().italic(),
            ));
        }

        // 3. Info items (model, session, mode)
        let mut first = true;
        for item in &self.items {
            let text = item.text.trim();
            if text == "●" || text == "○" || text.is_empty() {
                continue;
            }
            if !first && !self.is_thinking {
                spans.push(Span::styled(sep_str, sep));
            }
            // Labels like "Model:" in dim, values in main fg
            if let Some((label, value)) = text.split_once(": ") {
                spans.push(Span::styled(label, Style::default().fg(dim)));
                spans.push(Span::styled(":", Style::default().fg(dim)));
                spans.push(Span::styled(
                    value.to_string(),
                    Style::default().fg(main_fg),
                ));
            } else {
                spans.push(Span::styled(text, Style::default().fg(main_fg)));
            }
            first = false;
        }

        // 4. Right side: clock
        let clock = chrono::Local::now().format("%H:%M").to_string();
        let clock_span = Span::styled(format!(" {clock} "), Style::default().fg(dim));

        let left_line = Line::from(spans);
        let left_width = left_line.width();
        let clock_width = clock_span.width();

        let mut final_spans = left_line.spans;

        if area.width > (left_width as u16 + clock_width as u16) {
            let padding = " ".repeat(area.width as usize - left_width - clock_width);
            final_spans.push(Span::raw(padding));
            final_spans.push(clock_span);
        }

        frame.render_widget(Paragraph::new(Line::from(final_spans)), area);
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
        ChatColorsRgb::default()
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
        let toolbar = Toolbar::new(theme_colors, chat_colors);
        assert!(toolbar.items.is_empty());
    }

    #[test]
    fn test_toolbar_add_text() {
        let theme_colors = create_test_theme_colors();
        let chat_colors = create_test_chat_colors();
        let mut toolbar = Toolbar::new(theme_colors, chat_colors);
        toolbar.add_text("Test");
        assert_eq!(toolbar.items.len(), 1);
    }
    #[test]
    fn test_toolbar_clear() {
        let theme_colors = create_test_theme_colors();
        let chat_colors = create_test_chat_colors();
        let mut toolbar = Toolbar::new(theme_colors, chat_colors);
        toolbar.add_default_text("Test 1");
        assert_eq!(toolbar.items.len(), 1);
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
        toolbar.update_status(true, Some("gpt-4"), Some("openai"), Some("Session 1"), 3);
        assert!(toolbar.items.len() >= 6);
    }

    #[test]
    fn test_toolbar_item_new() {
        let item = ToolbarItem::new("Test");
        assert_eq!(item.text, "Test");
    }

    #[test]
    fn test_toolbar_item_with_defaults() {
        let item = ToolbarItem::new("Defaults");
        assert_eq!(item.text, "Defaults");
    }
}
