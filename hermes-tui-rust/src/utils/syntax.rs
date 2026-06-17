//! Syntax Highlighting Module
//!
//! This module provides syntax highlighting for code blocks in chat messages
//! using the syntect library, plus a thread-local O(1) scope-color cache
//! (Phase 2.2) for fast inline token styling during LLM streaming.

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Style as SyntectStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

// ============================================================================
// Phase 2.2: Thread-Local O(1) Syntax Highlighting Cache
// ============================================================================

/// A pre-computed 11-color palette for inline scope → color resolution.
///
/// Each index maps to a common syntax-highlighting scope category:
///
/// | Index | Scope           | Typical Colour     |
/// |-------|-----------------|--------------------|
/// | 0     | Plain text      | Default foreground |
/// | 1     | Keyword         | Cyan / Blue        |
/// | 2     | String          | Green              |
/// | 3     | Comment         | Dim gray           |
/// | 4     | Type            | Yellow             |
/// | 5     | Function/Ident  | Bright blue        |
/// | 6     | Number/Constant | Magenta            |
/// | 7     | Operator        | White              |
/// | 8     | Preprocessor    | Purple             |
/// | 9     | Markdown header | Bright yellow      |
/// | 10    | Markdown link   | Cyan underline     |
#[derive(Debug, Clone)]
pub struct ScopePalette {
    pub colors: [Color; 11],
}

impl ScopePalette {
    /// Catppuccin Macchiato-inspired palette — warm, high-contrast.
    #[must_use]
    pub const fn catppuccin() -> Self {
        Self {
            colors: [
                Color::Rgb(202, 211, 245),  // 0 text
                Color::Rgb(137, 180, 250),  // 1 keyword
                Color::Rgb(166, 218, 149),  // 2 string
                Color::Rgb(128, 135, 154),  // 3 comment
                Color::Rgb(250, 189, 47),   // 4 type
                Color::Rgb(138, 173, 244),  // 5 function
                Color::Rgb(245, 169, 127),  // 6 number
                Color::Rgb(198, 212, 240),  // 7 operator
                Color::Rgb(198, 160, 246),  // 8 preproc
                Color::Rgb(250, 200, 90),   // 9 header
                Color::Rgb(137, 220, 235),  // 10 link
            ],
        }
    }

    /// Resolve a scope index to a colour (clamped to palette length).
    #[must_use]
    pub fn resolve(&self, scope: usize) -> Color {
        self.colors.get(scope).copied().unwrap_or(self.colors[0])
    }
}

thread_local! {
    /// Thread-local O(1) scope-color cache.
    ///
    /// Initialised once per thread (typically the main / UI thread). Avoids
    /// repeated syntect theme resolution during markdown streaming.
    static SCOPE_COLOR_CACHE: ScopePalette = ScopePalette::catppuccin();
}

/// Apply a single scope-index colour to a text fragment in O(1).
///
/// # Panics
///
/// Never — out-of-range scopes gracefully fall back to the default colour.
#[must_use]
pub fn fast_scope_color(scope: usize) -> Color {
    SCOPE_COLOR_CACHE.with(|palette| palette.resolve(scope))
}

/// Style a `Span` with a scope-indexed colour.
///
/// # Example
///
/// ```ignore
/// let span = fast_style_span("fn", 1); // keyword colour
/// ```
#[must_use]
pub fn fast_style_span(text: &str, scope: usize) -> Span<'static> {
    let color = fast_scope_color(scope);
    Span::styled(text.to_string(), Style::new().fg(color))
}

/// Build a `Line` from (text, scope) pairs in O(1) colour time per token.
#[must_use]
pub fn fast_highlight_line(tokens: &[(&str, usize)]) -> Line<'static> {
    let spans: Vec<Span<'static>> = tokens
        .iter()
        .map(|(text, scope)| fast_style_span(text, *scope))
        .collect();
    Line::from(spans)
}

/// High-level wrapper for the O(1) highlight path.
pub struct FastScopeHighlighter;

impl FastScopeHighlighter {
    /// Create a new highlighter (zero-cost).
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Highlight a string token by scope index.
    #[must_use]
    pub fn highlight_token(&self, text: &str, scope: usize) -> Span<'static> {
        fast_style_span(text, scope)
    }

    /// Highlight a line from (text, scope) pairs.
    #[must_use]
    pub fn highlight_line(&self, tokens: &[(&str, usize)]) -> Line<'static> {
        fast_highlight_line(tokens)
    }
}

impl Default for FastScopeHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Original syntect-based highlighter (unchanged)
// ============================================================================

/// Syntax highlighter for code blocks
pub struct SyntaxHighlighter {
    /// Syntax set for parsing code
    syntax_set: SyntaxSet,
    /// Theme set for coloring
    theme_set: ThemeSet,
    /// Current theme name
    theme_name: String,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter with default theme
    #[must_use]
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        Self {
            syntax_set,
            theme_set,
            theme_name: "base16-ocean.dark".to_string(),
        }
    }

    /// Create a new syntax highlighter with a specific theme
    pub fn with_theme(theme_name: impl Into<String>) -> Self {
        let mut highlighter = Self::new();
        highlighter.set_theme(theme_name);
        highlighter
    }

    /// Set the syntax highlighting theme
    pub fn set_theme(&mut self, theme_name: impl Into<String>) {
        self.theme_name = theme_name.into();
    }

    /// Get available themes
    pub fn available_themes(&self) -> Vec<&str> {
        self.theme_set
            .themes
            .keys()
            .map(std::string::String::as_str)
            .collect()
    }

    /// Highlight a code block and return styled lines
    pub fn highlight(&self, code: &str, language: Option<&str>) -> Vec<Line<'static>> {
        // Get the theme
        let theme = self
            .theme_set
            .themes
            .get(&self.theme_name)
            .unwrap_or_else(|| self.theme_set.themes.get("base16-ocean.dark").unwrap());

        // Determine syntax based on language hint
        let syntax = if let Some(lang) = language {
            self.syntax_set
                .find_syntax_for_file(lang)
                .ok()
                .or_else(|| Some(self.syntax_set.find_syntax_by_token(lang)))
                .unwrap_or(Some(self.syntax_set.find_syntax_plain_text()))
        } else {
            Some(self.syntax_set.find_syntax_plain_text())
        };

        let syntax = syntax.unwrap();
        let mut h = HighlightLines::new(syntax, theme);

        // Process each line
        let lines = LinesWithEndings::from(code);
        let mut styled_lines = Vec::new();

        for line in lines {
            let ranges: Vec<(SyntectStyle, &str)> =
                h.highlight_line(line, &self.syntax_set).unwrap_or_default();
            let mut spans = Vec::new();

            for (style, text) in ranges {
                let fg_color = style.foreground;
                let color = convert_syntect_color(fg_color);

                spans.push(Span::styled(text.to_string(), Style::new().fg(color)));
            }

            styled_lines.push(Line::from(spans));
        }

        styled_lines
    }
}

/// Convert syntect highlighting Color to ratatui Color
fn convert_syntect_color(color: syntect::highlighting::Color) -> Color {
    Color::Rgb(color.r, color.g, color.b)
}

/// Extract code blocks from markdown text
/// Returns a vector of (language, code) tuples
#[must_use]
pub fn extract_code_blocks(text: &str) -> Vec<(Option<String>, String)> {
    let mut code_blocks = Vec::new();
    let mut lines = text.lines();

    while let Some(line) = lines.next() {
        if line.trim().starts_with("```") {
            let lang = if line.trim().len() > 3 {
                Some(line.trim()[3..].trim().to_string())
            } else {
                None
            };

            let mut code_lines = Vec::new();
            for next_line in lines.by_ref() {
                if next_line.trim().starts_with("```") {
                    break;
                }
                code_lines.push(next_line);
            }

            code_blocks.push((lang, code_lines.join("\n")));
        }
    }

    code_blocks
}

/// Check if a line is a fenced code block start
#[must_use]
pub fn is_code_block_start(line: &str) -> bool {
    line.trim().starts_with("```")
}

/// Check if a line is a fenced code block end
#[must_use]
pub fn is_code_block_end(line: &str) -> bool {
    line.trim().starts_with("```")
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Existing tests unchanged ---
    #[test]
    fn test_syntax_highlighter_new() {
        let highlighter = SyntaxHighlighter::new();
        assert!(!highlighter.available_themes().is_empty());
    }

    #[test]
    fn test_extract_code_blocks() {
        let text = r#"
Some text before

```python
def hello():
    print("world")
```

More text

```rust
fn main() {
    println!("hello");
}
```
"#;

        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 2);

        assert_eq!(blocks[0].0, Some("python".to_string()));
        assert!(blocks[0].1.contains("def hello():"));

        assert_eq!(blocks[1].0, Some("rust".to_string()));
        assert!(blocks[1].1.contains("fn main()"));
    }

    #[test]
    fn test_highlight_basic() {
        let highlighter = SyntaxHighlighter::new();
        let code = "fn main() {\n    println!(\"hello\");\n}";

        let lines = highlighter.highlight(code, Some("rust"));
        assert!(!lines.is_empty());
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_is_code_block_start() {
        assert!(is_code_block_start("```python"));
        assert!(is_code_block_start("```"));
        assert!(!is_code_block_start("not a code block"));
    }

    // --- Phase 2.2 tests ---

    #[test]
    fn test_scope_palette_catppuccin() {
        let palette = ScopePalette::catppuccin();
        assert_eq!(palette.colors.len(), 11);
        // Ensure default colour is the first entry
        assert_eq!(palette.resolve(99), palette.colors[0]);
    }

    #[test]
    fn test_fast_scope_color_in_range() {
        let color = fast_scope_color(1); // keyword
        assert_ne!(color, Color::Reset);
    }

    #[test]
    fn test_fast_scope_color_out_of_range_falls_back() {
        let default = fast_scope_color(0);
        let fallback = fast_scope_color(42);
        assert_eq!(default, fallback);
    }

    #[test]
    fn test_fast_style_span() {
        let span = fast_style_span("fn", 1);
        assert!(span.content.to_string().contains("fn"));
    }

    #[test]
    fn test_fast_highlight_line() {
        let line = fast_highlight_line(&[("let", 1), (" ", 0), ("x", 5), (" = ", 7), ("42", 6)]);
        let spans: Vec<String> = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert_eq!(spans.join(""), "let x = 42");
    }

    #[test]
    fn test_fast_scope_highlighter() {
        let hl = FastScopeHighlighter::new();
        let span = hl.highlight_token("const", 1);
        assert!(span.content.to_string().contains("const"));

        let line = hl.highlight_line(&[("// comment", 3)]);
        assert_eq!(line.spans.len(), 1);
    }
}
