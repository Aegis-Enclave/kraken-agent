//! Composer module - Input composer UI component
//!
//! This module provides the multi-line text input component for the TUI.

use ratatui::{
    layout::{Position, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::state::config::{ChatColorsRgb, InputMode};

/// Input composer component for multi-line text input
///
/// This component handles user text input with support for:
/// - Multi-line editing
/// - Command mode (for slash commands)
/// - Syntax highlighting
/// - History navigation
/// - Tab completion
#[derive(Debug, Clone)]
pub struct InputComposer {
    /// Input text
    input: String,
    /// Cursor position (character index)
    cursor_pos: usize,
    /// Current input mode
    input_mode: InputMode,
    /// Chat colors from configuration
    colors: ChatColorsRgb,
    /// Input prompt string
    prompt: String,
    /// Whether the composer is active
    active: bool,
    /// History of previous inputs
    history: Vec<String>,
    /// Current position in history
    history_index: isize,
    /// Saved input when entering history mode
    saved_input: String,
}

impl InputComposer {
    /// Create a new input composer with the given colors
    #[must_use]
    pub fn new(colors: ChatColorsRgb) -> Self {
        Self {
            input: String::new(),
            cursor_pos: 0,
            input_mode: InputMode::Normal,
            colors,
            prompt: " ≡ ".to_string(),
            active: true,
            history: Vec::new(),
            history_index: -1,
            saved_input: String::new(),
        }
    }

    /// Create a new input composer with all defaults
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(ChatColorsRgb {
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
        })
    }

    /// Set the input mode
    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
        self.update_prompt();
    }

    /// Update the prompt based on mode
    fn update_prompt(&mut self) {
        match self.input_mode {
            InputMode::Normal => self.prompt = " ≡ ".to_string(),
            InputMode::Insert => self.prompt = " ≡ ".to_string(),
            InputMode::Command => self.prompt = " / ".to_string(),
        }
    }

    /// Handle a key event
    pub fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> bool {
        if !self.active {
            return false;
        }

        match key.code {
            crossterm::event::KeyCode::Char(c) => {
                // Insert character
                let insert_pos = self.cursor_pos.min(self.input.len());
                self.input.insert(insert_pos, c);
                self.cursor_pos = insert_pos + 1;
                self.cancel_history();
                true
            }
            crossterm::event::KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    let remove_pos = self.cursor_pos - 1;
                    self.input.remove(remove_pos);
                    self.cursor_pos = remove_pos;
                    self.cancel_history();
                }
                true
            }
            crossterm::event::KeyCode::Delete => {
                if self.cursor_pos < self.input.len() {
                    self.input.remove(self.cursor_pos);
                    self.cancel_history();
                }
                true
            }
            crossterm::event::KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                true
            }
            crossterm::event::KeyCode::Right => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                }
                true
            }
            crossterm::event::KeyCode::Home => {
                self.cursor_pos = 0;
                true
            }
            crossterm::event::KeyCode::End => {
                self.cursor_pos = self.input.len();
                true
            }
            crossterm::event::KeyCode::Enter => {
                if !self.input.is_empty() {
                    self.add_to_history(self.input.clone());
                }
                // Clear input but don't clear history
                self.saved_input = self.input.clone();
                self.input.clear();
                self.cursor_pos = 0;
                true
            }
            crossterm::event::KeyCode::Esc => {
                self.clear();
                true
            }
            crossterm::event::KeyCode::Up => {
                self.history_previous();
                true
            }
            crossterm::event::KeyCode::Down => {
                self.history_next();
                true
            }
            _ => false,
        }
    }

    /// Get the current input text
    #[must_use]
    pub fn get_input(&self) -> &str {
        &self.input
    }

    /// Get the current cursor position (character index)
    #[must_use]
    pub fn cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    /// Get the current input and cursor position
    #[must_use]
    pub fn input_and_cursor(&self) -> (&str, usize) {
        (&self.input, self.cursor_pos)
    }

    /// Set the cursor position (character index)
    pub fn set_cursor_pos(&mut self, pos: usize) {
        self.cursor_pos = pos.min(self.input.len());
    }

    /// Set the input text
    pub fn set_input(&mut self, text: impl Into<String>) {
        self.input = text.into();
        // Cursor goes to the end of the text
        self.cursor_pos = self.input.len();
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_pos = 0;
        self.cancel_history();
    }

    /// Insert text at the current cursor position
    pub fn insert_text(&mut self, text: &str) {
        let pos = self.cursor_pos.min(self.input.len());
        self.input.insert_str(pos, text);
        self.cursor_pos = pos + text.len();
        self.cancel_history();
    }

    /// Add to history
    pub fn add_to_history(&mut self, text: String) {
        if text.is_empty() {
            return;
        }
        if let Some(last) = self.history.last() {
            if last == &text {
                return;
            }
        }
        self.history.push(text);
        if self.history.len() > 100 {
            self.history.remove(0);
        }
        self.history_index = -1;
        self.saved_input.clear();
    }

    /// Navigate to previous history entry
    pub fn history_previous(&mut self) {
        if self.history.is_empty() {
            return;
        }
        if self.history_index < 0 {
            self.saved_input = self.input.clone();
            self.history_index = self.history.len() as isize - 1;
        } else if self.history_index > 0 {
            self.history_index -= 1;
        }
        let index = self.history_index as usize;
        if index < self.history.len() {
            self.set_input(self.history[index].clone());
        }
    }

    /// Navigate to next history entry
    pub fn history_next(&mut self) {
        if self.history_index < 0 {
            return;
        }
        if self.history_index < (self.history.len() - 1) as isize {
            self.history_index += 1;
            let index = self.history_index as usize;
            if index < self.history.len() {
                self.set_input(self.history[index].clone());
            }
        } else {
            self.history_index = -1;
            self.set_input(self.saved_input.clone());
            self.saved_input.clear();
        }
    }

    /// Cancel history navigation
    pub fn cancel_history(&mut self) {
        self.history_index = -1;
        self.saved_input.clear();
    }

    /// Set active state
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Check if active
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get input mode
    #[must_use]
    pub fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    /// Set colors
    pub fn set_colors(&mut self, colors: ChatColorsRgb) {
        self.colors = colors;
    }

    /// Get colors
    #[must_use]
    pub fn colors(&self) -> &ChatColorsRgb {
        &self.colors
    }

    /// Get the history
    #[must_use]
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Clear the history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.history_index = -1;
        self.saved_input.clear();
    }

    /// Calculate the required height in terminal rows for the current input.
    /// Returns at least 1, and at most a configurable max height (default 8).
    #[must_use]
    pub fn required_height(&self, max_height: u16) -> u16 {
        if self.input.is_empty() {
            // Welcome message + input line + some padding
            return 3u16.min(max_height);
        }
        let line_count = self.input.lines().count().max(1) as u16;
        // Add 1 for the prompt + anything beyond max gets clamped
        (line_count + 1).min(max_height).max(1)
    }
    /// Render the composer
    /// Render the composer without a surrounding block (for the clean footer)
    pub fn render_clean(&self, frame: &mut Frame, area: Rect) {
        let mut lines = Vec::new();

        // Welcome message (if input is empty)
        if self.input.is_empty() {
            lines.push(Line::from(Span::styled(
                "Welcome to Hermes TUI! Type your message or /help for commands.",
                Style::new().fg(Color::Rgb(248, 248, 242)),
            )));
            lines.push(Line::from(vec![
                Span::styled("* ", Style::new().fg(Color::Rgb(166, 226, 46))),
                Span::styled("Tip: ", Style::new().fg(Color::Rgb(248, 248, 242)).bold()),
                Span::styled("hermes chat --max-turns 200 overrides the default 90-iteration limit per turn.", Style::new().fg(Color::Rgb(117, 113, 94)).italic()),
            ]));
        }

        // Prompt and input
        let prompt_span = Span::styled(
            &self.prompt,
            Style::new().fg(Color::Rgb(248, 248, 242)).bold(),
        );

        let input_line = if self.input.is_empty() {
            Line::from(vec![prompt_span])
        } else {
            let mut spans = vec![prompt_span];
            spans.push(Span::raw(&self.input));
            Line::from(spans)
        };
        lines.push(input_line);

        let line_count = lines.len();
        let paragraph =
            Paragraph::new(Text::from(lines)).wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(paragraph, area);

        // Set cursor position
        if self.active {
            let prompt_len = self.prompt.chars().count();
            let cursor_x = area.x + (prompt_len + self.cursor_pos) as u16;
            // The cursor should be on the last line (the input line)
            let cursor_y = area.y + (line_count as u16).saturating_sub(1);

            if cursor_x < area.x + area.width {
                frame.set_cursor_position(Position::new(cursor_x, cursor_y));
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let border_style = if self.active {
            Style::new().fg(self.colors.border)
        } else {
            Style::new().fg(self.colors.border).dim()
        };

        let block = Block::default()
            .title(" Input ".bold())
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(border_style);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // Build display text
        let mut lines = Vec::new();
        let prompt_span = Span::styled(
            &self.prompt,
            Style::new().fg(Color::Rgb(102, 217, 239)).bold(),
        ); // #66D9EF Cyan

        if self.input.is_empty() {
            // Show placeholder
            let placeholder = match self.input_mode {
                InputMode::Normal => "Type your message or /help...",
                InputMode::Insert => "Insert text...",
                InputMode::Command => "Enter command...",
            };
            lines.push(Line::from(vec![
                prompt_span,
                Span::styled(
                    placeholder,
                    Style::new().fg(Color::Rgb(117, 113, 94)).italic(),
                ), // #75715E Gray
            ]));
        } else {
            // Split input into lines and prepend prompt to first line
            let input_lines: Vec<&str> = self.input.lines().collect();
            if input_lines.is_empty() {
                lines.push(Line::from(vec![prompt_span]));
            } else {
                for (i, line) in input_lines.iter().enumerate() {
                    if i == 0 {
                        lines.push(Line::from(vec![prompt_span.clone(), Span::raw(*line)]));
                    } else {
                        // Indent subsequent lines by prompt length
                        let indent = " ".repeat(self.prompt.chars().count());
                        lines.push(Line::from(vec![Span::raw(indent), Span::raw(*line)]));
                    }
                }
            }
        }

        // Create paragraph
        let paragraph = Paragraph::new(Text::from(lines))
            .style(Style::new().fg(self.colors.code_text))
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(paragraph, inner_area);

        // Set cursor position with multi-line support
        if self.active {
            let prompt_len = self.prompt.chars().count() as u16;
            let inner_width = inner_area.width;

            // Find which logical line the cursor is in
            let input_before = &self.input[..self.cursor_pos.min(self.input.len())];
            let current_logical_line_idx = input_before.matches('\n').count();
            let input_lines: Vec<&str> = self.input.lines().collect();

            let mut cursor_y = inner_area.y;

            // Add visual lines for all logical lines before the current one
            for i in 0..current_logical_line_idx {
                let line_text = input_lines.get(i).copied().unwrap_or("");
                let line_len = line_text.chars().count() as u16;
                let first_line_width = inner_width.saturating_sub(prompt_len);

                if first_line_width > 0 && line_len > first_line_width {
                    let remaining = line_len - first_line_width;
                    cursor_y += 1
                        + (remaining + inner_width - 1)
                            .checked_div(inner_width)
                            .unwrap_or(0);
                } else {
                    cursor_y += 1;
                }
            }

            // Now calculate position within the current logical line
            let line_start = input_before.rfind('\n').map_or(0, |i| i + 1);
            let col_in_line = (self.cursor_pos - line_start) as u16;
            let first_line_width = inner_width.saturating_sub(prompt_len);

            let (wrapped_extra, final_col) =
                if first_line_width > 0 && col_in_line >= first_line_width {
                    let remaining = col_in_line - first_line_width;
                    if inner_width > 0 {
                        (1 + remaining / inner_width, remaining % inner_width)
                    } else {
                        (1, 0)
                    }
                } else {
                    (0, prompt_len + col_in_line)
                };

            cursor_y += wrapped_extra;
            let cursor_x = inner_area.x + final_col;

            if cursor_x < inner_area.x + inner_area.width
                && cursor_y < inner_area.y + inner_area.height
            {
                frame.set_cursor_position(Position::new(cursor_x, cursor_y));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_colors() -> ChatColorsRgb {
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
    fn test_composer_new() {
        let colors = create_test_colors();
        let composer = InputComposer::new(colors);
        assert!(composer.get_input().is_empty());
        assert_eq!(composer.cursor_pos, 0);
        assert_eq!(composer.input_mode(), InputMode::Normal);
    }

    #[test]
    fn test_composer_set_input() {
        let colors = create_test_colors();
        let mut composer = InputComposer::new(colors);

        composer.set_input("Hello, world!");
        assert_eq!(composer.get_input(), "Hello, world!");
        assert_eq!(composer.cursor_pos, 13);
    }

    #[test]
    fn test_composer_clear() {
        let colors = create_test_colors();
        let mut composer = InputComposer::new(colors);

        composer.set_input("Test");
        assert_eq!(composer.get_input(), "Test");

        composer.clear();
        assert!(composer.get_input().is_empty());
        assert_eq!(composer.cursor_pos, 0);
    }

    #[test]
    fn test_composer_handle_char() {
        let colors = create_test_colors();
        let mut composer = InputComposer::new(colors);

        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(composer.handle_key_event(key));
        assert_eq!(composer.get_input(), "a");
        assert_eq!(composer.cursor_pos, 1);
    }

    #[test]
    fn test_composer_handle_backspace() {
        let colors = create_test_colors();
        let mut composer = InputComposer::new(colors);

        composer.set_input("Hello");
        // After set_input, cursor is at position 5 (after 'o')
        assert_eq!(composer.cursor_pos, 5);

        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(composer.handle_key_event(key));
        assert_eq!(composer.get_input(), "Hell");
        // After backspace, cursor is at position 4 (after 'l')
        assert_eq!(composer.cursor_pos, 4);
    }

    #[test]
    fn test_composer_handle_enter() {
        let colors = create_test_colors();
        let mut composer = InputComposer::new(colors);

        composer.set_input("Hello");
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(composer.handle_key_event(key));
        assert!(composer.get_input().is_empty());
        assert!(composer.history().contains(&"Hello".to_string()));
    }

    #[test]
    fn test_composer_cursor_movement() {
        let colors = create_test_colors();
        let mut composer = InputComposer::new(colors);

        composer.set_input("Hello");

        // Move to start
        let key = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        assert!(composer.handle_key_event(key));
        assert_eq!(composer.cursor_pos, 0);

        // Move to end
        let key = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);
        assert!(composer.handle_key_event(key));
        assert_eq!(composer.cursor_pos, 5);

        // Move left
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        assert!(composer.handle_key_event(key));
        assert_eq!(composer.cursor_pos, 4);

        // Move right
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        assert!(composer.handle_key_event(key));
        assert_eq!(composer.cursor_pos, 5);
    }

    #[test]
    fn test_composer_history() {
        let colors = create_test_colors();
        let mut composer = InputComposer::new(colors);

        composer.add_to_history("first".to_string());
        composer.add_to_history("second".to_string());

        assert_eq!(composer.history().len(), 2);

        // Navigate to previous - loads "second" (most recent)
        composer.history_previous();
        assert_eq!(composer.get_input(), "second");

        // Navigate to previous again - loads "first" (older)
        composer.history_previous();
        assert_eq!(composer.get_input(), "first");

        // Navigate to next - goes back to "second"
        composer.history_next();
        assert_eq!(composer.get_input(), "second");

        // Navigate to next again - restores saved input (empty)
        composer.history_next();
        assert!(composer.get_input().is_empty());
    }

    #[test]
    fn test_composer_input_mode() {
        let colors = create_test_colors();
        let mut composer = InputComposer::new(colors);

        assert_eq!(composer.input_mode(), InputMode::Normal);

        composer.set_input_mode(InputMode::Insert);
        assert_eq!(composer.input_mode(), InputMode::Insert);

        composer.set_input_mode(InputMode::Command);
        assert_eq!(composer.input_mode(), InputMode::Command);
    }

    #[test]
    fn test_composer_with_defaults() {
        let composer = InputComposer::with_defaults();
        assert!(composer.get_input().is_empty());
        assert_eq!(composer.input_mode(), InputMode::Normal);
    }

    #[test]
    fn test_composer_active_state() {
        let colors = create_test_colors();
        let mut composer = InputComposer::new(colors);

        assert!(composer.is_active());

        composer.set_active(false);
        assert!(!composer.is_active());

        composer.set_active(true);
        assert!(composer.is_active());
    }
}
