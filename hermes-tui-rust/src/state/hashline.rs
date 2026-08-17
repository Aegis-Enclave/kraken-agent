//! Hashline module - File edit visualization types and parser
//!
//! This module provides types for parsing and representing file edit operations
//! (the hashline format) used by the agent when modifying files.

use serde::{Deserialize, Serialize};

/// Type of line edit in a hashline block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashlineEditType {
    /// Line was added (+)
    Addition,
    /// Line was deleted (-)
    Deletion,
    /// Line was replaced
    Replacement,
    /// Context line shown for reference (no change)
    Context,
    /// Content was inserted
    Insertion,
}

/// A single edit line in a hashline block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashlineEdit {
    /// Type of edit
    pub edit_type: HashlineEditType,
    /// Line content
    pub content: String,
    /// Optional line number
    pub line_number: Option<u32>,
    /// Optional file path (for multi-file blocks)
    pub path: Option<String>,
}

/// A block of hashline edits for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashlineEditBlock {
    /// File path being edited
    pub path: String,
    /// Edits in this block
    pub edits: Vec<HashlineEdit>,
    /// Starting line number (1-indexed)
    pub start_line: u32,
    /// Ending line number (1-indexed)
    pub end_line: u32,
}

/// Parser for hashline-format edit descriptions
///
/// The hashline format looks like:
/// ```text
/// [src/main.rs#A1B2]
/// replace 1..3:
/// +new line one
/// +new line two
/// ```
#[derive(Debug, Default)]
pub struct HashlineParser;

impl HashlineParser {
    /// Create a new parser
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Parse a hashline-format string into an edit block
    ///
    /// Returns `None` if the content doesn't match the hashline format.
    /// The format is:
    /// - First line: `[path/to/file.rs#TAG]` (optional header)
    /// - Followed by edit commands like: `replace N..M:`, `delete N`, `insert after N:`
    /// - Followed by body lines prefixed with `+` for additions
    #[must_use]
    pub fn parse(content: &str) -> Option<HashlineEditBlock> {
        let mut lines = content.lines().peekable();

        // Parse optional file path header: [path.rs#TAG]
        let mut path = String::new();
        if let Some(first) = lines.peek() {
            if first.starts_with('[') && first.contains(']') {
                let inner = first.trim_start_matches('[').split(']').next()?;
                let path_part = inner.split('#').next()?.trim().to_string();
                if !path_part.is_empty() {
                    path = path_part;
                }
                lines.next(); // consume header
            }
        }

        let mut edits = Vec::new();
        let mut has_edit_markers = false;

        for line in lines {
            let trimmed = line.trim();

            // Check for edit commands
            if trimmed.starts_with("replace")
                || trimmed.starts_with("delete")
                || trimmed.starts_with("insert")
            {
                has_edit_markers = true;
                continue;
            }

            if let Some(content) = trimmed.strip_prefix('+') {
                has_edit_markers = true;
                edits.push(HashlineEdit {
                    edit_type: HashlineEditType::Addition,
                    content: content.to_string(),
                    line_number: None,
                    path: None,
                });
            } else if let Some(content) = trimmed.strip_prefix('-') {
                has_edit_markers = true;
                edits.push(HashlineEdit {
                    edit_type: HashlineEditType::Deletion,
                    content: content.to_string(),
                    line_number: None,
                    path: None,
                });
            }
        }

        // Only return a block if we actually found edit markers
        if !has_edit_markers {
            return None;
        }

        Some(HashlineEditBlock {
            path,
            edits,
            start_line: 1,
            end_line: 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hashline_simple() {
        let content =
            "[src/main.rs#A1B2]\nreplace 1..3:\n+fn main() {\n+    println!(\"hello\");\n}";
        let block = HashlineParser::parse(content);
        assert!(block.is_some());
        let block = block.unwrap();
        assert_eq!(block.path, "src/main.rs");
        assert_eq!(block.edits.len(), 2);
        assert_eq!(block.edits[0].edit_type, HashlineEditType::Addition);
        assert_eq!(block.edits[0].content, "fn main() {");
        assert_eq!(block.edits[1].edit_type, HashlineEditType::Addition);
        assert_eq!(block.edits[1].content, "    println!(\"hello\");");
    }

    #[test]
    fn test_parse_hashline_no_header() {
        let content = "+    println!(\"hello\");";
        let block = HashlineParser::parse(content);
        assert!(block.is_some());
        let block = block.unwrap();
        assert!(block.path.is_empty());
        assert_eq!(block.edits.len(), 1);
    }

    #[test]
    fn test_parse_no_edits() {
        assert!(HashlineParser::parse("just a plain message").is_none());
        assert!(HashlineParser::parse("").is_none());
    }

    #[test]
    fn test_parse_delete_lines() {
        let content = "[src/lib.rs#C3D4]\ndelete 10\n-old line\n-older line";
        let block = HashlineParser::parse(content);
        assert!(block.is_some());
        let block = block.unwrap();
        assert_eq!(block.path, "src/lib.rs");
        assert_eq!(block.edits.len(), 2);
        assert_eq!(block.edits[0].edit_type, HashlineEditType::Deletion);
        assert_eq!(block.edits[1].edit_type, HashlineEditType::Deletion);
    }
}
