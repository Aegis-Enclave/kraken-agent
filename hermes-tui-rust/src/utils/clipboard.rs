//! Clipboard abstraction.
//!
//! Tries `arboard` first; falls back to writing an OSC 52 escape sequence to
//! stdout on failure (works in many modern terminals). Tests use a recorder
//! that captures the last set text.

use std::io::Write;

/// Errors a clipboard implementation can return.
#[derive(Debug)]
pub enum ClipboardError {
    /// The backend could not complete the operation.
    Backend(String),
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::Backend(s) => write!(f, "clipboard backend error: {s}"),
        }
    }
}

impl std::error::Error for ClipboardError {}

/// Trait implemented by every clipboard backend.
pub trait Clipboard {
    /// Copy `text` to the clipboard.
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError>;
}

/// Real clipboard that wraps `arboard` if the `clipboard-support` feature is enabled.
#[cfg(feature = "clipboard-support")]
pub struct ArboardClipboard {
    inner: arboard::Clipboard,
}

#[cfg(feature = "clipboard-support")]
impl ArboardClipboard {
    /// Construct a new `arboard` clipboard.
    pub fn new() -> Result<Self, ClipboardError> {
        arboard::Clipboard::new()
            .map(|inner| Self { inner })
            .map_err(|e| ClipboardError::Backend(e.to_string()))
    }
}

#[cfg(feature = "clipboard-support")]
impl Clipboard for ArboardClipboard {
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.inner
            .set_text(text.to_string())
            .map_err(|e| ClipboardError::Backend(e.to_string()))
    }
}

/// OSC 52 fallback. Writes a `DCS 52 ; c ; <base64> ST` sequence to the given
/// writer. Useful for headless setups where `arboard` cannot connect to a
/// display server.
pub struct Osc52Clipboard<W: Write> {
    sink: W,
}

impl<W: Write> Osc52Clipboard<W> {
    /// Construct a new OSC 52 sink.
    pub fn new(sink: W) -> Self {
        Self { sink }
    }
}

impl<W: Write> Clipboard for Osc52Clipboard<W> {
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        // OSC 52 ; c ; <base64> ESC \
        // We use the canonical C1 form (DCS + ST) for max terminal compatibility.
        let mut buf = String::with_capacity(text.len() * 2);
        buf.push_str("\x1b]52;c;");
        // Simple base64 encode; we use a tiny impl to avoid pulling in base64
        // just for the OSC 52 path.
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let bytes = text.as_bytes();
        let mut i = 0;
        while i + 3 <= bytes.len() {
            let n =
                ((bytes[i] as u32) << 16) | ((bytes[i + 1] as u32) << 8) | (bytes[i + 2] as u32);
            buf.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
            buf.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
            buf.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
            buf.push(ALPHABET[(n & 0x3F) as usize] as char);
            i += 3;
        }
        match bytes.len() - i {
            1 => {
                let n = (bytes[i] as u32) << 16;
                buf.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
                buf.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
                buf.push('=');
                buf.push('=');
            }
            2 => {
                let n = ((bytes[i] as u32) << 16) | ((bytes[i + 1] as u32) << 8);
                buf.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
                buf.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
                buf.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
                buf.push('=');
            }
            _ => {}
        }
        buf.push('\x07');
        self.sink
            .write_all(buf.as_bytes())
            .map_err(|e| ClipboardError::Backend(e.to_string()))
    }
}

/// Test-only clipboard that records the last `set_text` call.
#[cfg(test)]
#[derive(Debug, Default)]
pub struct MockClipboard {
    pub last: std::cell::RefCell<Option<String>>,
}

#[cfg(test)]
impl MockClipboard {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
impl Clipboard for MockClipboard {
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        *self.last.borrow_mut() = Some(text.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_clipboard_records() {
        let mut cb = MockClipboard::new();
        cb.set_text("hello").unwrap();
        assert_eq!(cb.last.borrow().as_deref(), Some("hello"));
    }

    #[test]
    fn test_osc52_emits_valid_sequence() {
        let mut buf = Vec::new();
        {
            let mut cb = Osc52Clipboard::new(&mut buf);
            cb.set_text("abc").unwrap();
        }
        let out = String::from_utf8(buf).unwrap();
        assert!(out.starts_with("\x1b]52;c;"));
        assert!(out.ends_with('\x07'));
        // "abc" base64 = "YWJj"
        assert!(out.contains("YWJj"));
    }

    #[test]
    fn test_osc52_handles_one_byte_padding() {
        let mut buf = Vec::new();
        {
            let mut cb = Osc52Clipboard::new(&mut buf);
            cb.set_text("a").unwrap();
        }
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("YQ=="));
    }
}
