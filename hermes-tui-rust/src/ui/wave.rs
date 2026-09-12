//! GlyphWave — Phase-shifted sine wave footer
//!
//! Implements the "Aetheric Shaders" concept from Phase 4 of the animation
//! plan.  Instead of static spinners or dots, a fluid, oceanic wave of block
//! characters (`' ', '▂', '▃', '▄', '▅'`) provides constant visual feedback
//! during "Thinking / Working" states.
//!
//! ## How it works
//! - `wave_glyph(x, tick)` evaluates two summed sine waves at different
//!   frequencies and maps the result onto 5 block height levels.
//! - Rendering counts backwards (`iter().rev()`) and breaks early once the
//!   visible `Rect` width is filled — exactly as the `oha` pattern prescribes.
//!
//! ## Integration
//! Call `render_wave_footer(frame, area, tick, usage)` in your draw path whenever
//! `thinking == true`.  The component respects `ACTIVE_ANIMATIONS` so the
//! event loop stays demand-driven.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::engine;

const WAVE_STR: [char; 9] = [' ', ' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

const COLOR_PROMPT: Color = Color::Rgb(102, 217, 239); // Cyan
const COLOR_TOOL: Color = Color::Rgb(250, 189, 47); // Yellow
const COLOR_REASON: Color = Color::Rgb(174, 129, 255); // Purple
const COLOR_OUTPUT: Color = Color::Rgb(166, 226, 46); // Green
const COLOR_FAILED: Color = Color::Rgb(249, 38, 114); // Red

const COLORS: [Color; 5] = [
    COLOR_PROMPT,
    COLOR_TOOL,
    COLOR_REASON,
    COLOR_OUTPUT,
    COLOR_FAILED,
];

const LABELS: [&str; 5] = ["Prompt", "Tool", "Reason", "Output", "Fail"];

/// Internal wave height evaluator for an "Equalizer" style motion.
/// Uses multiple frequencies to simulate a frequency analyzer.
fn get_bar_height(x: usize, tick: usize, count: u32) -> f64 {
    let speed_mult = 1.0 + f64::from(count).ln_1p() * 0.5;
    let t = (tick as f64 / 15.0) * speed_mult;
    let x_f = x as f64;

    // Sum of different frequencies for "frequency analyzer" look
    let f1 = (x_f * 0.5 + t * 0.8).sin() * 0.4;
    let f2 = (x_f * 1.2 - t * 1.5).sin() * 0.3;
    let f3 = (x_f * 2.5 + t * 2.2).sin() * 0.2;
    let f4 = (x_f * 0.1 - t * 0.5).cos() * 0.5; // slow baseline

    let val = (f1 + f2 + f3 + f4).abs(); // Use absolute to keep it above baseline

    // Scale and add some randomness based on X to make bars jumpy
    let noise = ((x_f * 7.7 + t * 3.3).sin() * 0.1).abs();

    (val + noise).clamp(0.0, 1.0)
}

/// Map a normalized wave height to one of the renderable block glyphs.
#[cfg(test)]
fn wave_glyph(x: usize, tick: usize, count: u32) -> char {
    let h_norm = get_bar_height(x, tick, count);
    let idx = (h_norm * (WAVE_STR.len() - 1) as f64).round() as usize;
    WAVE_STR[idx.clamp(0, WAVE_STR.len() - 1)]
}

/// Render an animated equalizer wave footer across the full width of `area`.
pub fn render_wave_footer(
    frame: &mut Frame,
    area: Rect,
    tick: usize,
    usage: (u32, u32, u32, u32, u32),
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let text_height = 1.min(area.height);
    let wave_height = area.height.saturating_sub(text_height);

    let wave_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: wave_height,
    };

    let text_area = Rect {
        x: area.x,
        y: area.y + wave_height,
        width: area.width,
        height: text_height,
    };

    let width = area.width as usize;
    let counts = [usage.0, usage.1, usage.2, usage.3, usage.4];

    // Calculate pillar widths
    let mut col_widths = [0_u16; 5];
    for x in 0..width {
        let segment_idx = (x * 5) / width;
        col_widths[segment_idx.min(4)] += 1;
    }

    let bg_color = Color::Rgb(27, 32, 33);

    // Render Equalizer Bars
    if wave_height > 0 {
        let mut lines: Vec<Line> = Vec::with_capacity(wave_height as usize);

        let styles = [
            Style::default().fg(COLORS[0]).bg(bg_color),
            Style::default().fg(COLORS[1]).bg(bg_color),
            Style::default().fg(COLORS[2]).bg(bg_color),
            Style::default().fg(COLORS[3]).bg(bg_color),
            Style::default().fg(COLORS[4]).bg(bg_color),
        ];

        let wave_h_f = f64::from(wave_height);

        for row in 0..wave_height {
            let row_idx = f64::from(wave_height - 1 - row);
            let mut spans: Vec<Span> = Vec::with_capacity(width);

            for x in 0..width {
                let segment_idx = (x * 5) / width;
                let segment_idx = segment_idx.min(4);
                let count = counts[segment_idx];
                let style = styles[segment_idx];

                let h_norm = get_bar_height(x, tick, count);
                let total_h = h_norm * wave_h_f;

                let ch = if total_h >= row_idx + 1.0 {
                    WAVE_STR[8] // Full block
                } else if total_h <= row_idx {
                    WAVE_STR[0] // Space
                } else {
                    let frac = total_h - row_idx;
                    let idx = (frac * 8.0).round() as usize;
                    WAVE_STR[idx.clamp(0, 8)]
                };

                spans.push(Span::styled(ch.to_string(), style));
            }

            lines.push(Line::from(spans));
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, wave_area);
    }

    // Render text footer
    if text_height > 0 {
        let mut current_x = text_area.x;
        for (i, &w) in col_widths.iter().enumerate() {
            if w == 0 {
                continue;
            }

            let chunk = Rect {
                x: current_x,
                y: text_area.y,
                width: w,
                height: 1,
            };
            current_x += w;

            let is_active = counts[i] > 0;
            let mut text_style = Style::default().fg(COLORS[i]).bg(bg_color);
            if is_active {
                text_style = text_style.add_modifier(Modifier::BOLD);
            } else {
                text_style = text_style.add_modifier(Modifier::DIM);
            }

            let label_str = format!("{} {}", LABELS[i], counts[i]);
            let paragraph = Paragraph::new(label_str)
                .style(text_style)
                .alignment(ratatui::layout::Alignment::Center);

            frame.render_widget(paragraph, chunk);
        }
    }
}

/// A ticker that manages the wave animation lifecycle.
#[derive(Debug)]
pub struct WaveTicker {
    tick: usize,
    active: bool,
}

impl WaveTicker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tick: 0,
            active: false,
        }
    }

    pub fn advance(&mut self) -> usize {
        self.tick = self.tick.wrapping_add(1);
        if !self.active {
            engine::animation_start();
            self.active = true;
        }
        self.tick
    }

    pub fn stop(&mut self) {
        if self.active {
            engine::animation_end();
            self.active = false;
        }
        self.tick = 0;
    }

    #[must_use]
    pub fn current_tick(&self) -> usize {
        self.tick
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for WaveTicker {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WaveTicker {
    fn drop(&mut self) {
        if self.active {
            engine::animation_end();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_glyph_returns_valid_char() {
        for x in 0..50 {
            for tick in [0, 16, 100, 500] {
                for count in [0, 10, 100, 1000] {
                    let ch = wave_glyph(x, tick, count);
                    assert!(WAVE_STR.contains(&ch), "unexpected glyph {ch:?}");
                }
            }
        }
    }

    #[test]
    fn test_wave_ticker_lifecycle() {
        {
            let mut ticker = WaveTicker::new();
            assert!(!ticker.is_active());

            let t1 = ticker.advance();
            assert!(ticker.is_active());
            assert_eq!(t1, 1);

            let t2 = ticker.advance();
            assert_eq!(t2, 2);

            ticker.stop();
            assert!(!ticker.is_active());
            assert_eq!(ticker.current_tick(), 0);
        }
    }
}
