//! Hashline UI - File edit visualization widget
//!
//! This module provides a widget for rendering hashline-style file edits
//! (visual diffs) in the chat transcript.

use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::state::hashline::{HashlineEditBlock, HashlineEditType};

/// Colors used in the hashline viewer
#[derive(Debug, Clone)]
pub struct HashlineColors {
    pub addition_fg: Color,
    pub addition_bg: Color,
    pub deletion_fg: Color,
    pub deletion_bg: Color,
    pub replacement_fg: Color,
    pub context_fg: Color,
    pub path_header_fg: Color,
    pub gutter_fg: Color,
    pub string_fg: Color,
}

impl Default for HashlineColors {
    fn default() -> Self {
        Self {
            addition_fg: Color::Rgb(0x4a, 0xcb, 0x62),    // green
            addition_bg: Color::Rgb(0x1b, 0x3a, 0x22),    // dark green
            deletion_fg: Color::Rgb(0xe5, 0x5c, 0x5c),    // red
            deletion_bg: Color::Rgb(0x3a, 0x1b, 0x1b),    // dark red
            replacement_fg: Color::Rgb(0xe5, 0xc0, 0x5c), // yellow
            context_fg: Color::Rgb(0xaa, 0xaa, 0xaa),     // gray
            path_header_fg: Color::Rgb(0x5c, 0xae, 0xe5), // blue
            gutter_fg: Color::Rgb(0x66, 0x66, 0x66),      // dim gray
            string_fg: Color::Rgb(0xce, 0x91, 0x78),      // orange (strings)
        }
    }
}

/// Widget for rendering hashline edit diffs
#[derive(Debug, Clone)]
pub struct HashlineViewer {
    /// Colors for the viewer
    colors: HashlineColors,
    /// Whether to show line numbers in the gutter
    show_gutter: bool,
}

impl Default for HashlineViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl HashlineViewer {
    /// Create a new hashline viewer with default colors
    #[must_use]
    pub fn new() -> Self {
        Self {
            colors: HashlineColors::default(),
            show_gutter: true,
        }
    }

    /// Set custom colors
    #[must_use]
    pub fn with_colors(colors: HashlineColors) -> Self {
        Self {
            colors,
            show_gutter: true,
        }
    }

    /// Enable or disable gutter
    #[must_use]
    pub fn show_gutter(mut self, show: bool) -> Self {
        self.show_gutter = show;
        self
    }

    /// Render a hashline edit block into the given area
    pub fn render(&self, edit_block: &HashlineEditBlock, area: Rect, frame: &mut Frame) {
        let lines = self.build_lines(edit_block);

        // Build file path header if available
        let title = if edit_block.path.is_empty() {
            " Edit ".to_string()
        } else {
            format!(" {} ", edit_block.path)
        };

        let block = Block::default()
            .title(title)
            .title_style(Style::default().fg(self.colors.path_header_fg).bold())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.colors.gutter_fg));

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Build ratatui Lines from an edit block
    fn build_lines(&self, edit_block: &HashlineEditBlock) -> Vec<Line<'static>> {
        let mut result = Vec::new();

        for (i, edit) in edit_block.edits.iter().enumerate() {
            let line_num = edit_block.start_line + i as u32;
            let (gutter_char, content_style) = self.style_for_type(edit.edit_type);

            // Build gutter part
            let mut spans: Vec<Span<'static>> = Vec::new();

            if self.show_gutter {
                let line_str = format!("{line_num:>4}");
                spans.push(Span::styled(
                    format!("{line_str}{gutter_char} "),
                    Style::default().fg(self.colors.gutter_fg),
                ));
            }

            // Style the content with basic syntax highlighting
            for token in tokenize_line(&edit.content) {
                let token_style = if token.is_string {
                    Style::default().fg(self.colors.string_fg)
                } else {
                    content_style
                };
                spans.push(Span::styled(token.text, token_style));
            }

            result.push(Line::from(spans));
        }

        result
    }

    /// Get the gutter character and content style for an edit type
    fn style_for_type(&self, edit_type: HashlineEditType) -> (&'static str, Style) {
        match edit_type {
            HashlineEditType::Addition => ("+", Style::default().fg(self.colors.addition_fg)),
            HashlineEditType::Deletion => ("-", Style::default().fg(self.colors.deletion_fg)),
            HashlineEditType::Replacement => ("~", Style::default().fg(self.colors.replacement_fg)),
            HashlineEditType::Insertion => (">", Style::default().fg(self.colors.addition_fg)),
            HashlineEditType::Context => (" ", Style::default().fg(self.colors.context_fg)),
        }
    }
}

/// A token from basic syntax highlighting
struct Token {
    text: String,
    is_string: bool,
}

/// Basic line-level syntax highlighting tokenizer
///
/// Splits a line into tokens, marking string literals (quoted text) differently.
fn tokenize_line(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().peekable();
    let mut current = String::new();

    while let Some(c) = chars.next() {
        if c == '"' {
            // Flush non-string buffer
            if !current.is_empty() {
                tokens.push(Token {
                    text: std::mem::take(&mut current),
                    is_string: false,
                });
            }
            // Start of string
            current.push('"');
            for c in chars.by_ref() {
                current.push(c);
                if c == '"' && !current.ends_with("\\\"") {
                    break;
                }
            }
            tokens.push(Token {
                text: std::mem::take(&mut current),
                is_string: true,
            });
        } else {
            current.push(c);
        }
    }

    if !current.is_empty() {
        tokens.push(Token {
            text: current,
            is_string: false,
        });
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::hashline::{HashlineEdit, HashlineEditType};

    #[test]
    fn test_tokenize_line_plain() {
        let tokens = tokenize_line("let x = 5;");
        assert_eq!(tokens.len(), 1);
        assert!(!tokens[0].is_string);
    }

    #[test]
    fn test_tokenize_line_with_string() {
        let tokens = tokenize_line(r#"println!("hello");"#);
        assert!(tokens.len() >= 2);
        assert!(tokens.iter().any(|t| t.is_string));
    }

    #[test]
    fn test_build_lines() {
        let viewer = HashlineViewer::new();
        let block = HashlineEditBlock {
            path: "test.rs".to_string(),
            edits: vec![
                HashlineEdit {
                    edit_type: HashlineEditType::Addition,
                    content: "fn main() {".to_string(),
                    line_number: None,
                    path: None,
                },
                HashlineEdit {
                    edit_type: HashlineEditType::Context,
                    content: "}".to_string(),
                    line_number: None,
                    path: None,
                },
            ],
            start_line: 1,
            end_line: 2,
        };
        let lines = viewer.build_lines(&block);
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_new_creates_default() {
        let viewer = HashlineViewer::new();
        assert!(viewer.show_gutter);
    }
}
