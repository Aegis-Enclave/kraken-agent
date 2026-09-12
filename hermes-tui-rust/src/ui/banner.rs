//! Banner module - ASCII logo and branding header
//!
//! This module provides the banner component for the TUI,
//! featuring the ASCII logo and deep-sea styling with a 3D
//! drop-shadow effect and bold filled characters.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

/// Banner component
#[derive(Debug, Default)]
pub struct Banner;

impl Banner {
    /// Render the banner with Kraken logo and branding
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        const SHADOW_DEPTH: u16 = 1;

        let colors = [
            Color::Rgb(166, 226, 46), // #A6E22E (Neon Green)
            Color::Rgb(166, 226, 46),
            Color::Rgb(230, 219, 116), // #E6DB74 (Yellow)
            Color::Rgb(230, 219, 116),
            Color::Rgb(174, 129, 255), // #AE81FF (Purple)
            Color::Rgb(174, 129, 255),
        ];

        // Background fill colors — darker tone of each foreground for solid "filled" look
        let fill_colors = [
            Color::Rgb(20, 40, 5),
            Color::Rgb(20, 40, 5),
            Color::Rgb(40, 35, 10),
            Color::Rgb(40, 35, 10),
            Color::Rgb(25, 15, 45),
            Color::Rgb(25, 15, 45),
        ];

        // Shadow-mask colors — very dark, desaturated version of each foreground
        let shadow_colors = [
            Color::Rgb(25, 35, 8),
            Color::Rgb(25, 35, 8),
            Color::Rgb(35, 32, 12),
            Color::Rgb(35, 32, 12),
            Color::Rgb(20, 15, 35),
            Color::Rgb(20, 15, 35),
        ];

        let logo_lines = [
            " _  _______            _  ________ _   _            _____ ______ _   _ _______ ",
            "| |/ /  __ \\     /\\\\   | |/ /  ____| \\\\ | |     /\\\\   / ____|  ____| \\\\ | |__   __|",
            "| ' /| |__) |   /  \\\\  | ' /| |__  |  \\\\| |    /  \\\\ | |  __| |__  |  \\\\| |  | |   ",
            "|  < |  _  /   / /\\\\ \\\\ |  < |  __| | . ` |   / /\\\\ \\\\| | |_ |  __| | . ` |  | |   ",
            "| . \\\\| | \\\\ \\\\  / ____ \\\\| . \\\\| |____| |\\\\  |  / ____ \\\\ |__| | |____| |\\\\  |  | |   ",
            "|_|\\\\_\\\\_|  \\\\_\\\\/_/    \\\\_\\\\_|\\\\_\\\\______|_| \\\\_| /_/    \\\\_\\\\_____|______|_| \\\\_|  |_|   ",
        ];

        // ── 3D drop-shadow layer ──────────────────────────────────
        // Render a darkened copy of the logo offset right by SHADOW_DEPTH
        // so it peeks out from behind the foreground.
        let shadow_area = Rect {
            x: area.x.saturating_add(SHADOW_DEPTH),
            y: area.y,
            width: area.width.saturating_sub(SHADOW_DEPTH),
            height: area.height,
        };

        let mut shadow_lines: Vec<Line> = Vec::new();
        for (i, text) in logo_lines.iter().enumerate() {
            let color = shadow_colors[i % shadow_colors.len()];
            shadow_lines.push(Line::from(Span::styled(*text, Style::default().fg(color))));
        }

        let shadow_para = Paragraph::new(shadow_lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        frame.render_widget(shadow_para, shadow_area);

        // ── Bold filled foreground layer ──────────────────────────
        let mut lines: Vec<Line> = Vec::new();
        for (i, text) in logo_lines.iter().enumerate() {
            let fg = colors[i % colors.len()];
            let bg = fill_colors[i % fill_colors.len()];
            lines.push(Line::from(Span::styled(
                *text,
                Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
            )));
        }

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Render the smaller version of the banner (just tagline or logo)
    ///
    /// Matches the default TUI's CompactBanner style: a rule-line with the
    /// brand name centered and surrounded by fill dashes filling the width.
    pub fn render_mini(&self, frame: &mut Frame, area: Rect) {
        let width = area.width as usize;
        let brand = "KRAKEN AGENT";
        let label = format!(" {} ", brand);
        let total_dashes = width.saturating_sub(label.len());
        let left_dashes = total_dashes / 2;
        let right_dashes = total_dashes - left_dashes;
        let green = Color::Rgb(166, 226, 46);

        let line = Line::from(vec![
            Span::styled(
                "─".repeat(left_dashes),
                Style::default().fg(green).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                brand,
                Style::default().fg(green).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "─".repeat(right_dashes),
                Style::default().fg(green).add_modifier(Modifier::BOLD),
            ),
        ]);

        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, area);
    }
}
