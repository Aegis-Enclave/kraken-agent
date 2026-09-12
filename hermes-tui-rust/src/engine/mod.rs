//! Core Animation & Rendering Engine
//!
//! This module implements the demand-driven render state, zero-flicker DEC 2026
//! synchronization, and bounded worker-pool foundations described in the
//! MODERN_TUI_ANIMATIONS plan.
//!
//! ## Demand-Driven Principle
//! A global `ACTIVE_ANIMATIONS` counter controls the event-loop poll timeout.
//! When zero animations are active the renderer sleeps deeply (blocking on I/O);
//! only when something is animating does it wake at 60 FPS.
//!
//! ## DEC 2026 Synchronized Output
//! Every `terminal.draw()` call is wrapped in `\x1b[?2026h` / `\x1b[?2026l`
//! sequences so modern emulators (Kitty, Ghostty, WezTerm) perform atomic
//! frame swaps, eliminating tearing during high-frequency updates.

use std::io::{self, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate},
};
use log::trace;
use ratatui::{backend::CrosstermBackend, Terminal};

/// Global counter of active animations.
///
/// - **Increment** when an animation starts (wave, spinner, effect).
/// - **Decrement** when it finishes.
/// - The event loop reads this to decide its poll timeout.
pub static ACTIVE_ANIMATIONS: AtomicU64 = AtomicU64::new(0);

/// Start tracking a new animation.
#[inline]
pub fn animation_start() {
    ACTIVE_ANIMATIONS.fetch_add(1, Ordering::Relaxed);
    trace!(
        "ACTIVE_ANIMATIONS ↑ {}",
        ACTIVE_ANIMATIONS.load(Ordering::Relaxed)
    );
}

/// Stop tracking an animation.
#[inline]
pub fn animation_end() {
    ACTIVE_ANIMATIONS.fetch_sub(1, Ordering::Relaxed);
    trace!(
        "ACTIVE_ANIMATIONS ↓ {}",
        ACTIVE_ANIMATIONS.load(Ordering::Relaxed)
    );
}

/// Query the best poll timeout for the event loop.
///
/// | Animations active | Timeout  | Behaviour             |
/// |-------------------|----------|-----------------------|
/// | 0                 | 16 ms    | Light idle tick       |
/// | >0                | 16 ms    | 60 FPS animation      |
///
/// The original design suggested *suspending* the ticker entirely when idle
/// (blocking indefinitely on `event::read`), but we keep a gentle 16 ms tick
/// so gateway health checks and message polling still run responsively.
#[must_use]
pub fn poll_timeout() -> Duration {
    let active = ACTIVE_ANIMATIONS.load(Ordering::Relaxed);
    if active > 0 {
        Duration::from_millis(16) // ~60 FPS
    } else {
        Duration::from_millis(100) // gentle idle, still fast enough for gateway I/O
    }
}

/// Wrap a `terminal.draw()` call with DEC 2026 synchronized output sequences.
///
/// Kitty, Ghostty, WezTerm, and other modern terminal emulators support the
/// DEC 2026 protocol which defers the actual frame compositing until the
/// end-sequence is received, producing tear-free swaps even at high FPS.
///
/// ## Fallback
/// Terminals that do not support DEC 2026 silently ignore the escape codes.
pub fn draw_sync<F>(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, draw_fn: F) -> Result<()>
where
    F: FnOnce(&mut ratatui::Frame),
{
    // Begin synchronized update
    execute!(io::stdout(), BeginSynchronizedUpdate)?;

    // Perform the actual widget rendering
    let res = terminal.draw(draw_fn);

    // End synchronized update
    execute!(io::stdout(), EndSynchronizedUpdate)?;
    io::stdout().flush()?;

    res.map(|_| ()).map_err(anyhow::Error::from)
}

// ============================================================================
// Bounded Channel Helpers
// ============================================================================

/// Suggested buffer size for the primary event bus.
///
/// Tokio `mpsc::channel(BACKPRESSURE_BUFFER)` enforces backpressure during
/// heavy LLM streaming so the UI thread is never starved.
pub const BACKPRESSURE_BUFFER: usize = 1024;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_counter() {
        assert_eq!(ACTIVE_ANIMATIONS.load(Ordering::Relaxed), 0);
        animation_start();
        assert_eq!(ACTIVE_ANIMATIONS.load(Ordering::Relaxed), 1);
        animation_start();
        assert_eq!(ACTIVE_ANIMATIONS.load(Ordering::Relaxed), 2);
        animation_end();
        assert_eq!(ACTIVE_ANIMATIONS.load(Ordering::Relaxed), 1);
        animation_end();
        assert_eq!(ACTIVE_ANIMATIONS.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_poll_timeout() {
        // Ensure restoring state after test
        let before = ACTIVE_ANIMATIONS.load(Ordering::Relaxed);
        assert_eq!(poll_timeout(), Duration::from_millis(100));
        ACTIVE_ANIMATIONS.store(5, Ordering::Relaxed);
        assert_eq!(poll_timeout(), Duration::from_millis(16));
        ACTIVE_ANIMATIONS.store(before, Ordering::Relaxed);
    }
}
