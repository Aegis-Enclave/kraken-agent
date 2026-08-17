//! ANSI module - ANSI code utility functions
//!
//! This module provides utility functions for working with ANSI escape codes,
//! including parsing, stripping, and generating ANSI sequences.

use ratatui::style::Color;
use ratatui::text::{Line, Span, Text};

/// ANSI color codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Rgb(u8, u8, u8),
    Indexed(u8),
    Default,
}

impl AnsiColor {
    /// Parse an ANSI color code from a string
    #[must_use]
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            30 => Some(Self::Black),
            31 => Some(Self::Red),
            32 => Some(Self::Green),
            33 => Some(Self::Yellow),
            34 => Some(Self::Blue),
            35 => Some(Self::Magenta),
            36 => Some(Self::Cyan),
            37 => Some(Self::White),
            90 => Some(Self::BrightBlack),
            91 => Some(Self::BrightRed),
            92 => Some(Self::BrightGreen),
            93 => Some(Self::BrightYellow),
            94 => Some(Self::BrightBlue),
            95 => Some(Self::BrightMagenta),
            96 => Some(Self::BrightCyan),
            97 => Some(Self::BrightWhite),
            _ => None,
        }
    }

    /// Get the ANSI escape code for this color (foreground)
    #[must_use]
    pub fn to_fg_code(&self) -> String {
        match self {
            Self::Black => "30".to_string(),
            Self::Red => "31".to_string(),
            Self::Green => "32".to_string(),
            Self::Yellow => "33".to_string(),
            Self::Blue => "34".to_string(),
            Self::Magenta => "35".to_string(),
            Self::Cyan => "36".to_string(),
            Self::White => "37".to_string(),
            Self::BrightBlack => "90".to_string(),
            Self::BrightRed => "91".to_string(),
            Self::BrightGreen => "92".to_string(),
            Self::BrightYellow => "93".to_string(),
            Self::BrightBlue => "94".to_string(),
            Self::BrightMagenta => "95".to_string(),
            Self::BrightCyan => "96".to_string(),
            Self::BrightWhite => "97".to_string(),
            Self::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
            Self::Indexed(i) => format!("38;5;{i}"),
            Self::Default => "39".to_string(),
        }
    }

    /// Get the ANSI escape code for this color (background)
    #[must_use]
    pub fn to_bg_code(&self) -> String {
        match self {
            Self::Black => "40".to_string(),
            Self::Red => "41".to_string(),
            Self::Green => "42".to_string(),
            Self::Yellow => "44".to_string(),
            Self::Blue => "44".to_string(),
            Self::Magenta => "45".to_string(),
            Self::Cyan => "46".to_string(),
            Self::White => "47".to_string(),
            Self::BrightBlack => "100".to_string(),
            Self::BrightRed => "101".to_string(),
            Self::BrightGreen => "102".to_string(),
            Self::BrightYellow => "103".to_string(),
            Self::BrightBlue => "104".to_string(),
            Self::BrightMagenta => "105".to_string(),
            Self::BrightCyan => "106".to_string(),
            Self::BrightWhite => "107".to_string(),
            Self::Rgb(r, g, b) => format!("48;2;{r};{g};{b}"),
            Self::Indexed(i) => format!("48;5;{i}"),
            Self::Default => "49".to_string(),
        }
    }

    /// Convert to ratatui Color
    #[must_use]
    pub fn to_ratatui_color(&self) -> Color {
        match self {
            Self::Black => Color::Black,
            Self::Red => Color::Red,
            Self::Green => Color::Green,
            Self::Yellow => Color::Yellow,
            Self::Blue => Color::Blue,
            Self::Magenta => Color::Magenta,
            Self::Cyan => Color::Cyan,
            Self::White => Color::White,
            Self::BrightBlack => Color::Gray,
            Self::BrightRed => Color::LightRed,
            Self::BrightGreen => Color::LightGreen,
            Self::BrightYellow => Color::LightYellow,
            Self::BrightBlue => Color::LightBlue,
            Self::BrightMagenta => Color::LightMagenta,
            Self::BrightCyan => Color::LightCyan,
            Self::BrightWhite => Color::White,
            Self::Rgb(r, g, b) => Color::Rgb(*r, *g, *b),
            Self::Indexed(i) => Color::Indexed(*i),
            Self::Default => Color::Reset,
        }
    }
}

impl From<Color> for AnsiColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Reset => Self::Default,
            Color::Black => Self::Black,
            Color::Red => Self::Red,
            Color::Green => Self::Green,
            Color::Yellow => Self::Yellow,
            Color::Blue => Self::Blue,
            Color::Magenta => Self::Magenta,
            Color::Cyan => Self::Cyan,
            Color::Gray => Self::BrightBlack,
            Color::DarkGray => Self::Black,
            Color::LightRed => Self::BrightRed,
            Color::LightGreen => Self::BrightGreen,
            Color::LightYellow => Self::BrightYellow,
            Color::LightBlue => Self::BrightBlue,
            Color::LightMagenta => Self::BrightMagenta,
            Color::LightCyan => Self::BrightCyan,
            Color::White => Self::White,
            Color::Rgb(r, g, b) => Self::Rgb(r, g, b),
            Color::Indexed(i) => Self::Indexed(i),
        }
    }
}

/// ANSI text style
#[derive(Debug, Clone, Default)]
pub struct AnsiStyle {
    pub foreground: Option<AnsiColor>,
    pub background: Option<AnsiColor>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub strikethrough: bool,
}

impl AnsiStyle {
    /// Create a new empty style
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set foreground color
    #[must_use]
    pub fn fg(mut self, color: AnsiColor) -> Self {
        self.foreground = Some(color);
        self
    }

    /// Set background color
    #[must_use]
    pub fn bg(mut self, color: AnsiColor) -> Self {
        self.background = Some(color);
        self
    }

    /// Set bold
    #[must_use]
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Set dim
    #[must_use]
    pub fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    /// Set italic
    #[must_use]
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Set underline
    #[must_use]
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Set blink
    #[must_use]
    pub fn blink(mut self) -> Self {
        self.blink = true;
        self
    }

    /// Set reverse
    #[must_use]
    pub fn reverse(mut self) -> Self {
        self.reverse = true;
        self
    }

    /// Set hidden
    #[must_use]
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    /// Set strikethrough
    #[must_use]
    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    /// Get the ANSI escape code for this style
    #[must_use]
    pub fn to_ansi(&self) -> String {
        let mut codes: Vec<String> = Vec::new();

        // Text attributes
        if self.bold {
            codes.push("1".to_string());
        }
        if self.dim {
            codes.push("2".to_string());
        }
        if self.italic {
            codes.push("3".to_string());
        }
        if self.underline {
            codes.push("4".to_string());
        }
        if self.blink {
            codes.push("5".to_string());
        }
        if self.reverse {
            codes.push("7".to_string());
        }
        if self.hidden {
            codes.push("8".to_string());
        }
        if self.strikethrough {
            codes.push("9".to_string());
        }

        // Foreground color
        if let Some(fg) = &self.foreground {
            codes.push(fg.to_fg_code());
        }

        // Background color
        if let Some(bg) = &self.background {
            codes.push(bg.to_bg_code());
        }

        if codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", codes.join(";"))
        }
    }

    /// Get the ANSI reset code
    #[must_use]
    pub fn reset() -> String {
        "\x1b[0m".to_string()
    }

    /// Convert to ratatui Style
    #[must_use]
    pub fn to_ratatui_style(&self) -> ratatui::style::Style {
        let mut style = ratatui::style::Style::new();

        if self.bold {
            style = style.add_modifier(ratatui::style::Modifier::BOLD);
        }
        if self.dim {
            style = style.add_modifier(ratatui::style::Modifier::DIM);
        }
        if self.italic {
            style = style.add_modifier(ratatui::style::Modifier::ITALIC);
        }
        if self.underline {
            style = style.add_modifier(ratatui::style::Modifier::UNDERLINED);
        }
        if self.blink {
            style = style.add_modifier(ratatui::style::Modifier::SLOW_BLINK);
        }
        if self.reverse {
            style = style.add_modifier(ratatui::style::Modifier::REVERSED);
        }
        if self.hidden {
            style = style.add_modifier(ratatui::style::Modifier::HIDDEN);
        }
        if self.strikethrough {
            style = style.add_modifier(ratatui::style::Modifier::CROSSED_OUT);
        }

        if let Some(fg) = &self.foreground {
            style = style.fg(fg.to_ratatui_color());
        }

        if let Some(bg) = &self.background {
            style = style.bg(bg.to_ratatui_color());
        }

        style
    }
}

/// ANSI escape code parser
#[derive(Debug, Default)]
pub struct AnsiParser {
    /// Current position in the input
    position: usize,
    /// Current style stack
    style_stack: Vec<AnsiStyle>,
}

impl AnsiParser {
    /// Create a new ANSI parser
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse ANSI escape codes from text and return stripped text with style info
    pub fn parse(&mut self, text: &str) -> Vec<(String, AnsiStyle)> {
        self.position = 0;
        self.style_stack = vec![AnsiStyle::new()];

        let mut result = Vec::new();
        let mut current_text = String::new();
        let current_style = self.style_stack.last().unwrap().clone();

        let bytes = text.as_bytes();
        while self.position < bytes.len() {
            if bytes[self.position] == b'\x1b' {
                // Found escape sequence
                if !current_text.is_empty() {
                    result.push((current_text.clone(), current_style.clone()));
                    current_text.clear();
                }

                // Parse the escape sequence
                self.parse_escape_sequence(bytes);
            } else {
                current_text.push(bytes[self.position] as char);
                self.position += 1;
            }
        }

        if !current_text.is_empty() {
            result.push((current_text, current_style));
        }

        result
    }

    /// Parse an ANSI escape sequence
    fn parse_escape_sequence(&mut self, bytes: &[u8]) {
        // Skip the escape character
        self.position += 1;

        // Check for CSI (Control Sequence Introducer)
        if self.position < bytes.len() && bytes[self.position] == b'[' {
            self.position += 1;
            self.parse_csi_sequence(bytes);
        } else {
            // Skip until we find a non-control character
            while self.position < bytes.len() {
                let c = bytes[self.position];
                self.position += 1;
                if (b'@'..=b'~').contains(&c) {
                    break;
                }
            }
        }
    }

    /// Parse a CSI (Control Sequence Introducer) sequence
    fn parse_csi_sequence(&mut self, bytes: &[u8]) {
        let mut params = Vec::new();
        let mut current_param = 0u32;
        let mut in_param = false;

        while self.position < bytes.len() {
            let c = bytes[self.position];
            self.position += 1;

            match c {
                b'0'..=b'9' => {
                    in_param = true;
                    current_param = current_param * 10 + u32::from(c - b'0');
                }
                b';' => {
                    if in_param {
                        params.push(current_param);
                        current_param = 0;
                        in_param = false;
                    }
                }
                b'@'..=b'~' => {
                    if in_param {
                        params.push(current_param);
                    }
                    self.handle_csi_command(c, &params);
                    return;
                }
                _ => {
                    // Invalid character, skip
                }
            }
        }
    }

    /// Handle a CSI command
    fn handle_csi_command(&mut self, final_byte: u8, params: &[u32]) {
        let mut style = self.style_stack.last().unwrap().clone();

        match final_byte {
            // SGR (Select Graphic Rendition)
            b'm' => {
                if params.is_empty() {
                    // Reset
                    *self.style_stack.last_mut().unwrap() = AnsiStyle::new();
                } else {
                    let mut i = 0;
                    while i < params.len() {
                        match params[i] {
                            0 => {
                                // Reset
                                *self.style_stack.last_mut().unwrap() = AnsiStyle::new();
                            }
                            1 => style.bold = true,
                            2 => style.dim = true,
                            3 => style.italic = true,
                            4 => style.underline = true,
                            5 => style.blink = true,
                            7 => style.reverse = true,
                            8 => style.hidden = true,
                            9 => style.strikethrough = true,
                            21 | 22 => style.bold = false,
                            23 => style.italic = false,
                            24 => style.underline = false,
                            25 => style.blink = false,
                            27 => style.reverse = false,
                            28 => style.hidden = false,
                            29 => style.strikethrough = false,
                            30..=37 => {
                                // Foreground color (standard)
                                if let Some(color) = AnsiColor::from_code(params[i] as u8) {
                                    style.foreground = Some(color);
                                }
                            }
                            38 => {
                                // Foreground color (extended)
                                if i + 2 < params.len() && params[i + 1] == 2 {
                                    // RGB
                                    let r = params[i + 2] as u8;
                                    let g = params[i + 3] as u8;
                                    let b = params[i + 4] as u8;
                                    style.foreground = Some(AnsiColor::Rgb(r, g, b));
                                    i += 4; // Skip the next 4 params
                                } else if i + 1 < params.len() && params[i + 1] == 5 {
                                    // Indexed
                                    let index = params[i + 2] as u8;
                                    style.foreground = Some(AnsiColor::Indexed(index));
                                    i += 2; // Skip the next 2 params
                                }
                            }
                            39 => style.foreground = None, // Default foreground
                            40..=47 => {
                                // Background color (standard)
                                let bg_code = params[i] - 10; // Convert to foreground code
                                if let Some(color) = AnsiColor::from_code(bg_code as u8) {
                                    // Need to map to actual background colors
                                    style.background = Some(color);
                                }
                            }
                            48 => {
                                // Background color (extended)
                                if i + 2 < params.len() && params[i + 1] == 2 {
                                    // RGB
                                    let r = params[i + 2] as u8;
                                    let g = params[i + 3] as u8;
                                    let b = params[i + 4] as u8;
                                    style.background = Some(AnsiColor::Rgb(r, g, b));
                                    i += 4;
                                } else if i + 1 < params.len() && params[i + 1] == 5 {
                                    // Indexed
                                    let index = params[i + 2] as u8;
                                    style.background = Some(AnsiColor::Indexed(index));
                                    i += 2;
                                }
                            }
                            49 => style.background = None, // Default background
                            90..=97 => {
                                // Bright foreground color
                                let bright_code = params[i] - 60; // Convert to standard code
                                if let Some(color) = AnsiColor::from_code(bright_code as u8) {
                                    // Map to bright variant
                                    style.foreground = Some(color);
                                }
                            }
                            100..=107 => {
                                // Bright background color
                                let bright_code = params[i] - 70;
                                if let Some(color) = AnsiColor::from_code(bright_code as u8) {
                                    style.background = Some(color);
                                }
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                    *self.style_stack.last_mut().unwrap() = style;
                }
            }
            // Cursor Up
            b'A' => {
                // Handle cursor movement if needed
            }
            // Cursor Down
            b'B' => {
                // Handle cursor movement if needed
            }
            // Cursor Forward
            b'C' => {
                // Handle cursor movement if needed
            }
            // Cursor Backward
            b'D' => {
                // Handle cursor movement if needed
            }
            // Cursor Next Line
            b'E' => {
                // Handle cursor movement if needed
            }
            // Cursor Previous Line
            b'F' => {
                // Handle cursor movement if needed
            }
            // Cursor Horizontal Absolute
            b'G' => {
                // Handle cursor movement if needed
            }
            // Cursor Position
            b'H' | b'f' => {
                // Handle cursor movement if needed
            }
            // Erase in Display
            b'J' => {
                // Handle erase if needed
            }
            // Erase in Line
            b'K' => {
                // Handle erase if needed
            }
            // Scroll Up
            b'S' => {
                // Handle scroll if needed
            }
            // Scroll Down
            b'T' => {
                // Handle scroll if needed
            }
            // Set Top and Bottom Margins
            b'r' => {
                // Handle margins if needed
            }
            // Save Cursor
            b's' => {
                // Handle save if needed
            }
            // Restore Cursor
            b'u' => {
                // Handle restore if needed
            }
            _ => {
                // Unknown command
            }
        }
    }

    /// Strip all ANSI escape codes from text
    #[must_use]
    pub fn strip_ansi(text: &str) -> String {
        let mut result = String::new();
        let bytes = text.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            if bytes[i] == b'\x1b' {
                // Skip escape sequence
                i += 1;
                if i < bytes.len() && bytes[i] == b'[' {
                    i += 1;
                    // Skip until final byte
                    while i < bytes.len() {
                        let c = bytes[i];
                        i += 1;
                        if (b'@'..=b'~').contains(&c) {
                            break;
                        }
                    }
                } else {
                    // Skip single escape
                    continue;
                }
            } else {
                result.push(bytes[i] as char);
                i += 1;
            }
        }

        result
    }

    /// Check if text contains ANSI escape codes
    #[must_use]
    pub fn has_ansi(text: &str) -> bool {
        text.contains('\x1b')
    }

    /// Get the visible width of text (excluding ANSI codes)
    #[must_use]
    pub fn visible_width(text: &str) -> usize {
        use unicode_width::UnicodeWidthStr;
        Self::strip_ansi(text).width()
    }

    /// Get the visible length of text (excluding ANSI codes)
    #[must_use]
    pub fn visible_length(text: &str) -> usize {
        Self::strip_ansi(text).chars().count()
    }
}

/// Convert ratatui Text to plain text with ANSI codes
#[must_use]
pub fn text_to_ansi(text: &Text) -> String {
    let mut result = String::new();

    for line in &text.lines {
        for span in &line.spans {
            // Convert style to ANSI
            let mut style = AnsiStyle::new();

            if span.style.fg.is_some() {
                if let Some(color) = span.style.fg {
                    style.foreground = Some(AnsiColor::from(color));
                }
            }

            if span.style.bg.is_some() {
                if let Some(color) = span.style.bg {
                    style.background = Some(AnsiColor::from(color));
                }
            }

            if span
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::BOLD)
            {
                style.bold = true;
            }
            if span
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::DIM)
            {
                style.dim = true;
            }
            if span
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::ITALIC)
            {
                style.italic = true;
            }
            if span
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::UNDERLINED)
            {
                style.underline = true;
            }
            if span
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::REVERSED)
            {
                style.reverse = true;
            }
            if span
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::HIDDEN)
            {
                style.hidden = true;
            }
            if span
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::CROSSED_OUT)
            {
                style.strikethrough = true;
            }

            result.push_str(&style.to_ansi());
            result.push_str(&span.content);
            result.push_str(&AnsiStyle::reset());
        }
        result.push('\n');
    }

    result
}

/// Convert plain text with ANSI codes to ratatui Text
#[must_use]
pub fn ansi_to_text(text: &str) -> Text<'_> {
    let mut parser = AnsiParser::new();
    let parsed = parser.parse(text);

    let mut lines: Vec<Line> = Vec::new();
    let mut current_line = Vec::new();

    for (text, style) in parsed {
        if text.contains('\n') {
            // Split by newlines
            for line in text.split('\n') {
                if !line.is_empty() {
                    current_line.push(Span::styled(line.to_string(), style.to_ratatui_style()));
                }
                if !current_line.is_empty() {
                    lines.push(Line::from(std::mem::take(&mut current_line)));
                }
            }
        } else {
            current_line.push(Span::styled(text, style.to_ratatui_style()));
        }
    }

    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }

    Text::from(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_color_from_code() {
        assert_eq!(AnsiColor::from_code(30), Some(AnsiColor::Black));
        assert_eq!(AnsiColor::from_code(31), Some(AnsiColor::Red));
        assert_eq!(AnsiColor::from_code(32), Some(AnsiColor::Green));
        assert_eq!(AnsiColor::from_code(90), Some(AnsiColor::BrightBlack));
        assert_eq!(AnsiColor::from_code(99), None);
    }

    #[test]
    fn test_ansi_color_to_fg_code() {
        assert_eq!(AnsiColor::Red.to_fg_code(), "31");
        assert_eq!(AnsiColor::Green.to_fg_code(), "32");
        assert_eq!(AnsiColor::BrightRed.to_fg_code(), "91");
        assert_eq!(AnsiColor::Rgb(255, 0, 0).to_fg_code(), "38;2;255;0;0");
        assert_eq!(AnsiColor::Indexed(123).to_fg_code(), "38;5;123");
    }

    #[test]
    fn test_ansi_style_to_ansi() {
        let style = AnsiStyle::new().fg(AnsiColor::Red).bold().underline();

        let ansi = style.to_ansi();
        assert!(ansi.contains("31"));
        assert!(ansi.contains("1"));
        assert!(ansi.contains("4"));
    }

    #[test]
    fn test_ansi_parser_strip() {
        let text = "\x1b[31mRed text\x1b[0m and \x1b[32mGreen\x1b[0m";
        let stripped = AnsiParser::strip_ansi(text);
        assert_eq!(stripped, "Red text and Green");
    }

    #[test]
    fn test_ansi_parser_has_ansi() {
        assert!(AnsiParser::has_ansi("\x1b[31mRed\x1b[0m"));
        assert!(!AnsiParser::has_ansi("Plain text"));
    }

    #[test]
    fn test_ansi_parser_visible_width() {
        let text = "\x1b[31mHello\x1b[0m";
        let width = AnsiParser::visible_width(text);
        assert_eq!(width, 5); // "Hello" is 5 chars
    }

    #[test]
    fn test_color_conversion() {
        let color = Color::Red;
        let ansi_color = AnsiColor::from(color);
        assert_eq!(ansi_color, AnsiColor::Red);

        let ansi_color = AnsiColor::Red;
        let color = ansi_color.to_ratatui_color();
        assert_eq!(color, Color::Red);
    }

    #[test]
    fn test_text_to_ansi() {
        let text = Text::from(Line::from(vec![
            Span::styled("Red", ratatui::style::Style::new().fg(Color::Red)),
            Span::styled(
                " Bold",
                ratatui::style::Style::new()
                    .fg(Color::Red)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
        ]));

        let ansi = text_to_ansi(&text);
        assert!(ansi.contains("\x1b[31m"), "should contain red code");
        assert!(
            ansi.contains("\x1b[1;31m") || ansi.contains(";1m"),
            "should contain bold+red combined code"
        );
        assert!(ansi.contains("Red"));
        assert!(ansi.contains("Bold"));
    }

    #[test]
    fn test_ansi_to_text() {
        let ansi = "\x1b[31mRed\x1b[0m \x1b[32mGreen\x1b[0m";
        let text = ansi_to_text(ansi);

        assert_eq!(text.lines.len(), 1);
        assert_eq!(text.lines[0].spans.len(), 3); // Red, space, Green
    }

    #[test]
    fn test_style_builder() {
        let style = AnsiStyle::new()
            .fg(AnsiColor::Blue)
            .bg(AnsiColor::White)
            .bold()
            .italic();

        assert_eq!(style.foreground, Some(AnsiColor::Blue));
        assert_eq!(style.background, Some(AnsiColor::White));
        assert!(style.bold);
        assert!(style.italic);
    }

    #[test]
    fn test_ansi_color_to_bg_code() {
        assert_eq!(AnsiColor::Red.to_bg_code(), "41");
        assert_eq!(AnsiColor::Green.to_bg_code(), "42");
        assert_eq!(AnsiColor::BrightRed.to_bg_code(), "101");
    }
}
