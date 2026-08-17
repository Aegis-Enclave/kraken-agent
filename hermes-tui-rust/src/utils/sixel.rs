//! Sixel terminal graphics support for image and video attachments.
//!
//! Encodes image bytes into SIXEL control strings that compatible terminals
//! (`XTerm`, `WezTerm`, foot, iTerm2, etc.) can render inline.

use anyhow::{Context, Result};
use icy_sixel::{EncodeOptions, SixelImage};

/// Detect whether the current terminal appears to support SIXEL.
///
/// Checks common environment variables first; if they are not set, assumes
/// support is absent to avoid garbled output on non-SIXEL terminals.
#[must_use]
pub fn terminal_supports_sixel() -> bool {
    if std::env::var("TERM").unwrap_or_default().contains("sixel") {
        return true;
    }
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        let lower = term_program.to_lowercase();
        if lower.contains("wezterm") || lower.contains("iterm") || lower.contains("mintty") {
            return true;
        }
    }
    if let Ok(true) = std::env::var("WEZTERM_EXECUTABLE").map(|v| !v.is_empty()) {
        return true;
    }
    false
}

/// Encode raw image bytes (any format supported by the `image` crate) into a
/// SIXEL string with a fixed maximum dimension.
///
/// Returns `Ok(None)` if SIXEL is not supported by the terminal.
pub fn encode_image_bytes(data: &[u8], max_dimension: u32) -> Result<Option<String>> {
    if !terminal_supports_sixel() {
        return Ok(None);
    }

    let img = image::load_from_memory(data).context("failed to load image")?;
    encode_dynamic_image(&resize_to_fit(img, max_dimension))
}

/// Encode a single frame of a video (or the first frame) into a SIXEL string.
///
/// The `data` argument is the raw video bytes. This is a best-effort helper
/// that decodes the input as an animated image/webp/gif first; otherwise
/// falls back to treating the bytes as a still image.
pub fn encode_video_frame(data: &[u8], max_dimension: u32) -> Result<Option<String>> {
    if !terminal_supports_sixel() {
        return Ok(None);
    }

    // Try to decode as an animated image/webp/gif first frame.
    if let Ok(img) = image::load_from_memory(data) {
        return encode_dynamic_image(&resize_to_fit(img, max_dimension));
    }

    // Fallback: treat as still image.
    encode_image_bytes(data, max_dimension)
}

pub fn encode_dynamic_image(img: &image::DynamicImage) -> Result<Option<String>> {
    if !terminal_supports_sixel() {
        return Ok(None);
    }

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let image = SixelImage::from_rgba(rgba.into_raw(), width as usize, height as usize);

    let opts = EncodeOptions::default();
    let sixel = image
        .encode_with(&opts)
        .map_err(|e| anyhow::anyhow!("sixel encode failed: {e}"))?;

    Ok(Some(sixel))
}
#[must_use]
pub fn resize_to_fit(img: image::DynamicImage, max_dimension: u32) -> image::DynamicImage {
    let (w, h) = (img.width(), img.height());
    if w <= max_dimension && h <= max_dimension {
        return img;
    }

    let ratio = (max_dimension as f32 / w as f32).min(max_dimension as f32 / h as f32);
    let new_w = (w as f32 * ratio).max(1.0) as u32;
    let new_h = (h as f32 * ratio).max(1.0) as u32;

    img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3)
}

/// Decode a base64-encoded image and encode it as SIXEL.
pub fn encode_base64_image(b64: &str, max_dimension: u32) -> Result<Option<String>> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .context("invalid base64 image data")?;
    encode_image_bytes(&bytes, max_dimension)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_supports_sixel_default() {
        // Default environment in tests should report false.
        assert!(!terminal_supports_sixel());
    }

    #[test]
    fn test_encode_image_bytes_without_sixel_support() {
        let data = [0u8; 16];
        // Since tests don't claim SIXEL support, this returns Ok(None).
        let result = encode_image_bytes(&data, 100);
        assert!(matches!(result, Ok(None)));
    }
}
