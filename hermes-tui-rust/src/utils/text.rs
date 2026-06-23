//! Text module - Text utility functions
//!
//! This module provides utility functions for text manipulation,
use ratatui::text::Line;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// ============================================================================
// Cell-aware helpers (used by chat height math and truncation)
// ============================================================================

/// Total *display cells* of a string, honoring East-Asian width, emoji, ZWJ.
#[must_use]
pub fn display_width(s: &str) -> usize {
    s.width()
}

/// Total *display cells* of a `ratatui::text::Line`, summing span widths.
#[must_use]
pub fn line_display_width(line: &Line<'_>) -> usize {
    line.spans.iter().map(|s| s.content.width()).sum()
}

/// Wrap a string to ≤ `max_cells` display cells, breaking only at cell
/// boundaries (never mid-grapheme).
///
/// The strategy:
/// 1. Walk cells, accumulating into `current`.
/// 2. If a hard `\n` is hit, push `current` and reset.
/// 3. If `current`'s cell width would exceed `max_cells`, prefer to break
///    at the last whitespace cell; otherwise hard-break at the boundary.
#[must_use]
pub fn wrap_display_cells(s: &str, max_cells: usize) -> Vec<String> {
    if max_cells == 0 {
        return vec![s.to_string()];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_cells = 0usize;
    let mut last_break_byte: Option<usize> = None;
    let mut last_break_cells = 0usize;

    for c in s.chars() {
        if c == '\n' {
            lines.push(std::mem::take(&mut current));
            current_cells = 0;
            last_break_byte = None;
            last_break_cells = 0;
            continue;
        }
        let cw = c.width().unwrap_or(1);
        if cw == 0 {
            current.push(c);
            continue;
        }

        if current_cells + cw > max_cells {
            if let Some(byte_idx) = last_break_byte {
                let head = current[..byte_idx].trim_end().to_string();
                let tail_start = if byte_idx < current.len() {
                    let after_ws = current[byte_idx..]
                        .chars()
                        .next()
                        .map(|ch| ch.len_utf8())
                        .unwrap_or(0);
                    current[byte_idx + after_ws..].to_string()
                } else {
                    String::new()
                };
                lines.push(head);
                current = tail_start;
                current_cells = current_cells.saturating_sub(last_break_cells + 1);
                last_break_byte = None;
                last_break_cells = 0;
            } else {
                // Hard break at cell boundary. Push current and reset.
                lines.push(std::mem::take(&mut current));
                current_cells = 0;
            }
            // Whether soft or hard break, skip any whitespace that would
            // become leading whitespace on the new line.
            if c.is_whitespace() {
                continue;
            }
        }

        if c.is_whitespace() && !current.is_empty() {
            last_break_byte = Some(current.len());
            last_break_cells = current_cells;
        }
        current.push(c);
        current_cells += cw;
    }

    if !current.is_empty() || lines.is_empty() {
        lines.push(current);
    }
    lines
}

/// Truncate a string to ≤ `max_cells` display cells, appending `ellipsis`
/// if the string was cut. Respects cell boundaries.
#[must_use]
pub fn truncate_to_cells(s: &str, max_cells: usize, ellipsis: &str) -> String {
    if max_cells == 0 {
        return String::new();
    }
    let s_width = s.width();
    if s_width <= max_cells {
        return s.to_string();
    }
    let ellipsis_w = ellipsis.width();
    if max_cells <= ellipsis_w {
        return ellipsis.chars().take(max_cells).collect();
    }
    let budget = max_cells - ellipsis_w;
    let mut out = String::new();
    let mut used = 0usize;
    for c in s.chars() {
        let cw = c.width().unwrap_or(1);
        if used + cw > budget {
            break;
        }
        out.push(c);
        used += cw;
    }
    out.push_str(ellipsis);
    out
}

/// Truncate text to a specified character count, respecting UTF-8 boundaries
/// and skipping ANSI escape codes for the count.
#[must_use]
pub fn truncate_safe(text: &str, max_chars: usize) -> String {
    let mut result = String::new();
    let mut char_count = 0;
    let mut i = 0;
    let bytes = text.as_bytes();

    while i < bytes.len() && char_count < max_chars {
        if bytes[i] == b'\x1b' {
            // Found escape sequence, copy it without counting as a character
            result.push(bytes[i] as char);
            i += 1;
            if i < bytes.len() && bytes[i] == b'[' {
                result.push(bytes[i] as char);
                i += 1;
                // Skip until final byte
                while i < bytes.len() {
                    let c = bytes[i];
                    result.push(c as char);
                    i += 1;
                    if (b'@'..=b'~').contains(&c) {
                        break;
                    }
                }
            }
        } else {
            // Regular character
            if let Some(c) = text[i..].chars().next() {
                result.push(c);
                i += c.len_utf8();
                char_count += 1;
            } else {
                break;
            }
        }
    }

    result
}

/// Determine if input looks like a slash command.
#[must_use]
pub fn looks_like_slash_command(input: &str) -> bool {
    input.starts_with('/') && !input.contains('\n')
}

/// A completion request derived from user input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionRequest {
    /// Method name to send to the gateway.
    pub method: &'static str,
    /// Query text (`text` for slash, `word` for path).
    pub query: String,
    /// Character index in the input where replacement should start.
    pub replace_from: usize,
}

/// Compute the completion request (if any) for the current input.
///
/// Mirrors the behavior of the TypeScript TUI completion hook:
/// - `/...` triggers slash completion (except `/model`).
/// - Path-like tokens (`./`, `../`, `~/`, `/`, `@`, or `foo/`) trigger path completion.
#[must_use]
pub fn completion_request_for_input(input: &str) -> Option<CompletionRequest> {
    let is_slash = looks_like_slash_command(input);

    // `/model` uses the dedicated ModelPicker; skip slash completion there.
    if is_slash
        && input.starts_with("/model")
        && (input.len() == 6 || input.chars().nth(6).is_some_and(char::is_whitespace))
    {
        return None;
    }

    if is_slash {
        return Some(CompletionRequest {
            method: "complete.slash",
            query: input.to_string(),
            replace_from: 1,
        });
    }

    // Path completion: match the last path-like token before the end of input.
    // Mirrors ui-tui: tokens starting with ./, ../, ~/, /, @, or containing a slash.
    let word = last_path_token(input)?;
    let replace_from = input.len() - word.len();

    Some(CompletionRequest {
        method: "complete.path",
        query: word.to_string(),
        replace_from,
    })
}

/// Extract the last path-like token from `input`, or None if there isn't one.
fn last_path_token(input: &str) -> Option<&str> {
    // Find the start of the last whitespace-delimited token.
    let start = input
        .rfind(|c: char| c.is_whitespace() || c == '"' || c == '\'')
        .map_or(0, |i| i + 1);
    let word = &input[start..];
    if word.is_empty() {
        return None;
    }

    let is_path_prefix = word.starts_with("./")
        || word.starts_with("../")
        || word.starts_with("~/")
        || word.starts_with('/')
        || word.starts_with('@')
        || word.starts_with("./")
        || word.starts_with("../");
    let contains_slash = word.contains('/');
    let is_windows_abs = word.len() >= 3
        && word.as_bytes()[1] == b':'
        && (word.as_bytes()[2] == b'/' || word.as_bytes()[2] == b'\\');

    if is_path_prefix || contains_slash || is_windows_abs {
        Some(word)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_width_cjk() {
        assert_eq!(display_width("你好世界"), 8);
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width(""), 0);
    }

    #[test]
    fn test_line_display_width_sums_spans() {
        let line = Line::from(vec![
            ratatui::text::Span::raw("你"),
            ratatui::text::Span::raw("好"),
        ]);
        assert_eq!(line_display_width(&line), 4);
    }

    #[test]
    fn test_display_width_empty() {
        assert_eq!(display_width(""), 0);
    }

    #[test]
    fn test_line_display_width_handles_empty_line() {
        let empty = Line::default();
        assert_eq!(line_display_width(&empty), 0);
    }

    #[test]
    fn test_wrap_display_cells_basic() {
        let input = "Hello world this is a test";
        let lines = wrap_display_cells(input, 10);
        assert!(!lines.is_empty());
        for line in &lines {
            assert!(line.width() <= 10);
        }
    }

    #[test]
    fn test_wrap_display_cells_respects_hard_newline() {
        let input = "Hello\nworld";
        let lines = wrap_display_cells(input, 100);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Hello");
        assert_eq!(lines[1], "world");
    }

    #[test]
    fn test_truncate_to_cells_short() {
        assert_eq!(truncate_to_cells("hi", 10, "…"), "hi");
    }

    #[test]
    fn test_truncate_to_cells_truncates() {
        let result = truncate_to_cells("hello world", 8, "…");
        assert!(result.width() <= 8);
        assert!(result.ends_with('…'));
    }

    #[test]
    fn test_truncate_to_cells_zero_max() {
        assert_eq!(truncate_to_cells("hello", 0, "…"), "");
    }

    #[test]
    fn test_truncate_safe_ansi() {
        let input = "\x1b[31mHello\x1b[0m";
        // 5 visible chars; trailing ANSI reset is not reached
        let result = truncate_safe(input, 5);
        assert_eq!(result, "\x1b[31mHello");
    }

    #[test]
    fn test_truncate_safe_limits_chars() {
        let result = truncate_safe("Hello World", 5);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_looks_like_slash_command() {
        assert!(looks_like_slash_command("/help"));
        assert!(!looks_like_slash_command("/help\nmore"));
        assert!(!looks_like_slash_command("help"));
    }

    #[test]
    fn test_completion_request_slash() {
        let req = completion_request_for_input("/he").unwrap();
        assert_eq!(req.method, "complete.slash");
        assert_eq!(req.query, "/he");
        assert_eq!(req.replace_from, 1);
    }

    #[test]
    fn test_completion_request_skips_model() {
        assert!(completion_request_for_input("/model").is_none());
        assert!(completion_request_for_input("/model ").is_none());
    }

    #[test]
    fn test_completion_request_path() {
        let req = completion_request_for_input("read ./src").unwrap();
        assert_eq!(req.method, "complete.path");
        assert_eq!(req.query, "./src");
        assert_eq!(req.replace_from, 5);
    }

    #[test]
    fn test_completion_request_no_trigger() {
        assert!(completion_request_for_input("hello").is_none());
        assert!(completion_request_for_input("").is_none());
    }

    // ----- cell-aware helpers -----
    #[test]
    fn test_wrap_display_cells_long_word_hard_break() {
        let input = "superlongword";
        let lines = wrap_display_cells(input, 5);
        assert!(!lines.is_empty());
        for line in &lines {
            assert!(line.width() <= 5, "{line:?} width {} > 5", line.width());
        }
    }

    #[test]
    fn test_wrap_display_cells_empty_input() {
        let lines = wrap_display_cells("", 10);
        assert!(!lines.is_empty());
        assert_eq!(lines[0], "");
    }

    #[test]
    fn test_wrap_display_cells_zero_max() {
        let lines = wrap_display_cells("test", 0);
        assert_eq!(lines, vec!["test"]);
    }
}
