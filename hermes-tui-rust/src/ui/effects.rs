//! Aetheric Shaders — Protocol-Linked DSL Effects (Phase 4.2)
//!
//! Manages a `tachyonfx::EffectManager` for lifecycle-tracking effects
//! (coalesce on LLM stream, HSL shift on tool execution). All post-processing
//! operates on a shim `Buffer` via the public `apply` method.
//!
//! ## Effect Pipeline
//!
//! 1. **LLM Delta Streaming** — `fx::coalesce` over newly appended text
//!    blocks, smoothing the appearance of streaming tokens.
//! 2. **Tool Execution** — `fx::hsl_shift` sweeping a bright cyan/green line
//!    across the tool card area.
//!
//! All effects are gated behind [`set_low_motion`] so users on
//! resource-constrained terminals can opt out with `--low-motion`.

use ratatui::{style::Color};
use tachyonfx::{self as txfx, EffectManager, EffectTimer, Interpolation};

/// Global gate for low-motion mode.
static mut LOW_MOTION: bool = false;

/// Enable or disable low-motion mode.
pub fn set_low_motion(enabled: bool) {
    unsafe { LOW_MOTION = enabled; }
}

/// Whether effects are currently allowed.
#[inline]
fn effects_enabled() -> bool {
    !unsafe { LOW_MOTION }
}

/// Shader state tied to the TUI event loop.
///
/// Uses a shim buffer from `ratatui_core` (the same crate tachyonfx depends on)
/// to avoid type mismatches with ratatui 0.28's own `Buffer`.
pub struct ShaderState {
    pub manager: EffectManager<()>,
}

impl Default for ShaderState {
    fn default() -> Self {
        Self {
            manager: EffectManager::default(),
        }
    }
}

impl ShaderState {
    /// Create a new ShaderState with an empty effect queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Trigger the LLM-delta coalesce effect (300ms).
    pub fn trigger_stream_effect(&mut self) {
        if !effects_enabled() {
            return;
        }
        self.manager
            .add_effect(txfx::fx::coalesce(EffectTimer::from_ms(300, Interpolation::Linear)));
    }

    /// Trigger the tool-execution HSL shift effect (400ms).
    pub fn trigger_tool_effect(&mut self) {
        if !effects_enabled() {
            return;
        }
        self.manager.add_effect(txfx::fx::hsl_shift(
            Some([60.0, 10.0, 15.0]),
            None,
            EffectTimer::from_ms(400, Interpolation::SineInOut),
        ));
    }

    /// Advance effect timers (called from the demand-driven ticker).
    pub fn advance(&mut self) {
        if !effects_enabled() || !self.manager.is_running() {
            return;
        }
        // Use ratatui_core types directly (tachyonfx's dependency)
        let mut buf = ratatui_core::buffer::Buffer::empty(
            ratatui_core::layout::Rect::new(0, 0, 0, 0),
        );
        self.manager.process_effects(
            txfx::Duration::from_millis(16),
            &mut buf,
            ratatui_core::layout::Rect::default(),
        );
    }

    /// Whether any effects are currently running.
    #[inline]
    pub fn is_running(&self) -> bool {
        self.manager.is_running()
    }

    /// Apply the current effects to the frame buffer.
    pub fn apply(&mut self, buf: &mut ratatui_core::buffer::Buffer, area: ratatui_core::layout::Rect) {
        if !effects_enabled() || !self.manager.is_running() {
            return;
        }
        self.manager
            .process_effects(txfx::Duration::from_millis(16), buf, area);
    }
}

/// Sinusoidal colour offsets for the wave footer.
pub fn wave_palette_shift(phase: f64) -> (Color, Color) {
    let r = (128.0 + (phase * 0.7).sin() * 40.0) as u8;
    let g = (180.0 + (phase * 1.3).cos() * 30.0) as u8;
    let b = (220.0 + (phase * 0.5).sin() * 35.0) as u8;

    let p2 = (phase * 2.0 + 1.2).sin();
    let r2 = (160.0 + p2 * 45.0) as u8;
    let g2 = (200.0 - p2 * 25.0) as u8;
    let b2 = (120.0 + p2 * 40.0) as u8;

    (Color::Rgb(r, g, b), Color::Rgb(r2, g2, b2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effects_enabled_default() {
        assert!(effects_enabled());
    }

    #[test]
    fn test_low_motion_gate() {
        set_low_motion(true);
        assert!(!effects_enabled());
        set_low_motion(false);
        assert!(effects_enabled());
    }

    #[test]
    fn test_wave_palette_shift_is_smooth() {
        let (a, _) = wave_palette_shift(0.0);
        let (c, _) = wave_palette_shift(1.0);
        assert_ne!(format!("{a:?}"), format!("{c:?}"));
    }

    #[test]
    fn test_shader_state_default() {
        let state = ShaderState::new();
        assert!(!state.is_running());
    }

    #[test]
    fn test_shader_state_trigger_stream() {
        let mut state = ShaderState::new();
        state.trigger_stream_effect();
        assert!(state.is_running());
    }

    #[test]
    fn test_shader_state_in_low_motion() {
        set_low_motion(true);
        let mut state = ShaderState::new();
        state.trigger_stream_effect();
        state.trigger_tool_effect();
        assert!(!state.is_running());
        set_low_motion(false);
    }
}
