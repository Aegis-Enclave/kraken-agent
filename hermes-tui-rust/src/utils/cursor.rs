//! Native Cursor Synchronization (Phase 3.3)
//!
//! Maps rope byte/char coordinates to terminal display columns using
//! `unicode-width` for correct multi-width Unicode alignment (CJK, emoji,
//! combining characters, Input Method Editors).
//!
//! This ensures the physical hardware cursor aligns with logical positions
//! in the editor, supporting screen readers and IME-based text entry.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Compute the terminal display column for a byte offset within a string.
///
/// Returns the column count (0-indexed) that the text up to `byte_idx`
/// occupies. Multi-width characters contribute their display width;
/// zero-width characters (combining marks, ZWJ, etc.) contribute nothing.
///
/// # Panics
///
/// If `byte_idx` is beyond the string's byte length.
#[must_use]
pub fn byte_to_col(text: &str, byte_idx: usize) -> usize {
    assert!(byte_idx <= text.len(), "byte_idx out of bounds");
    let prefix = &text[..byte_idx];
    prefix.width()
}

/// Compute the terminal display column for a char index within a string.
#[must_use]
pub fn char_to_col(text: &str, char_idx: usize) -> usize {
    let byte_idx = char_to_byte_idx(text, char_idx);
    byte_to_col(text, byte_idx)
}

/// Find the nearest byte offset for a terminal column within a string.
///
/// Scans graphemes / chars and accumulates display width until `col` is
/// reached or exceeded. Returns the byte index of the character at that
/// position. If `col` falls inside a multi-width character, the byte index
/// is rounded to the character's start (no splitting).
///
/// If `col` is beyond the total display width, returns `text.len()`.
#[must_use]
pub fn col_to_byte(text: &str, col: usize) -> usize {
    let mut accum = 0usize;
    for (byte_idx, ch) in text.char_indices() {
        let w = ch.width().unwrap_or(0);
        if accum + w > col {
            return byte_idx;
        }
        accum += w;
    }
    text.len()
}

/// Find the nearest char index for a terminal column within a string.
#[must_use]
pub fn col_to_char(text: &str, col: usize) -> usize {
    let byte_idx = col_to_byte(text, col);
    byte_to_char_idx(text, byte_idx)
}

/// Convert a char index to a byte index in a string.
fn char_to_byte_idx(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
}

/// Convert a byte index to a char index in a string.
fn byte_to_char_idx(text: &str, byte_idx: usize) -> usize {
    let clamped = byte_idx.min(text.len());
    // Walk char_indices — cheap because we only iterate up to `clamped`
    let mut count = 0usize;
    for (i, _) in text.char_indices() {
        if i >= clamped {
            break;
        }
        count += 1;
    }
    count
}

// ---------------------------------------------------------------------------
// CursorMap — high-level wrapper for RopeBuffer integration
// ---------------------------------------------------------------------------

/// Maps between rope coordinates and terminal display coordinates.
#[derive(Debug, Clone)]
pub struct CursorMap;

impl CursorMap {
    /// Compute the terminal column for a byte offset in a line.
    #[must_use]
    pub fn byte_to_col(line: &str, byte_idx: usize) -> usize {
        byte_to_col(line, byte_idx)
    }

    /// Compute the terminal column for a char index in a line.
    #[must_use]
    pub fn char_to_col(line: &str, char_idx: usize) -> usize {
        char_to_col(line, char_idx)
    }

    /// Find byte offset from a terminal column in a line.
    #[must_use]
    pub fn col_to_byte(line: &str, col: usize) -> usize {
        col_to_byte(line, col)
    }

    /// Find char index from a terminal column in a line.
    #[must_use]
    pub fn col_to_char(line: &str, col: usize) -> usize {
        col_to_char(line, col)
    }

    /// Align a byte offset to the nearest valid character boundary.
    ///
    /// Useful when the cursor column falls inside a multi-width character —
    /// this returns the byte offset of the character's start.
    #[must_use]
    pub fn snap_byte_to_char_boundary(line: &str, byte_idx: usize) -> usize {
        let clamped = byte_idx.min(line.len());
        if line.is_char_boundary(clamped) {
            clamped
        } else {
            // Walk back to the previous char boundary
            let mut i = clamped;
            while i > 0 && !line.is_char_boundary(i) {
                i -= 1;
            }
            i
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_to_col_ascii() {
        assert_eq!(byte_to_col("hello", 0), 0);
        assert_eq!(byte_to_col("hello", 3), 3);
        assert_eq!(byte_to_col("hello", 5), 5);
    }

    #[test]
    fn test_byte_to_col_wide() {
        // "你好" = 6 bytes, 2 chars, each 2 columns wide
        assert_eq!(byte_to_col("你好", 0), 0);
        assert_eq!(byte_to_col("你好", 3), 2); // after first char
        assert_eq!(byte_to_col("你好", 6), 4); // both chars
    }

    #[test]
    fn test_char_to_col() {
        assert_eq!(char_to_col("hello", 0), 0);
        assert_eq!(char_to_col("hello", 3), 3);
        // "你好" — 2 wide chars
        assert_eq!(char_to_col("你好", 0), 0);
        assert_eq!(char_to_col("你好", 1), 2);
        assert_eq!(char_to_col("你好", 2), 4);
    }

    #[test]
    fn test_col_to_byte_ascii() {
        assert_eq!(col_to_byte("hello", 0), 0);
        assert_eq!(col_to_byte("hello", 3), 3);
        assert_eq!(col_to_byte("hello", 5), 5);
        assert_eq!(col_to_byte("hello", 99), 5); // beyond
    }

    #[test]
    fn test_col_to_byte_wide() {
        assert_eq!(col_to_byte("你好", 0), 0);
        assert_eq!(col_to_byte("你好", 2), 3); // end of first char (col 2)
        assert_eq!(col_to_byte("你好", 3), 3); // inside first char rounds down
        assert_eq!(col_to_byte("你好", 4), 6);
    }

    #[test]
    fn test_roundtrip() {
        let s = "a你b好c";
        for byte_idx in [0, 1, 4, 5, 8, 9] {
            let col = byte_to_col(s, byte_idx);
            let back = col_to_byte(s, col);
            // Should map back to the same char start
            assert!(
                back <= byte_idx,
                "byte_to_col({})→{}→col_to_byte→{} should be ≤{byte_idx}",
                byte_idx,
                col,
                back,
            );
        }
    }

    #[test]
    fn test_col_to_char() {
        assert_eq!(col_to_char("hello", 0), 0);
        assert_eq!(col_to_char("hello", 3), 3);
        assert_eq!(col_to_char("你好", 2), 1); // wide char takes 2 cols
    }

    #[test]
    fn test_snap_to_boundary() {
        let s = "héllo";
        // 'é' is 2 bytes at positions 1..3
        assert_eq!(CursorMap::snap_byte_to_char_boundary(s, 2), 1); // inside 'é' → start
        assert_eq!(CursorMap::snap_byte_to_char_boundary(s, 1), 1); // already at boundary
        assert_eq!(CursorMap::snap_byte_to_char_boundary(s, 0), 0);
        assert_eq!(CursorMap::snap_byte_to_char_boundary(s, 6), 6);
    }

    #[test]
    fn test_cursor_map_static_methods() {
        let s = "a😀b"; // 😀 is 4 bytes, 2 columns wide
        assert_eq!(CursorMap::byte_to_col(s, 0), 0);
        assert_eq!(CursorMap::byte_to_col(s, 1), 1); // 'a'
        assert_eq!(CursorMap::byte_to_col(s, 5), 3); // after 😀: 1 + 2 = 3
        assert_eq!(CursorMap::col_to_byte(s, 2), 1); // col 2 = inside 😀 → byte 1
    }
}
