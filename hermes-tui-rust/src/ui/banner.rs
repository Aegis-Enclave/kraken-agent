//! Banner module - ASCII logo and branding header
//!
//! This module provides the banner component for the TUI,
//! featuring the Kraken ASCII logo and deep-sea styling.

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
        let colors = [
            Color::Rgb(166, 226, 46),   // #A6E22E (Neon Green)
            Color::Rgb(166, 226, 46),
            Color::Rgb(230, 219, 116),  // #E6DB74 (Yellow)
            Color::Rgb(230, 219, 116),
            Color::Rgb(174, 129, 255),  // #AE81FF (Purple)
            Color::Rgb(174, 129, 255),
        ];

        let logo_lines = [
            " _  _______            _  ________ _   _            _____ ______ _   _ _______ ",
            "| |/ /  __ \\     /\\   | |/ /  ____| \\ | |     /\\   / ____|  ____| \\ | |__   __|",
            "| ' /| |__) |   /  \\  | ' /| |__  |  \\| |    /  \\ | |  __| |__  |  \\| |  | |   ",
            "|  < |  _  /   / /\\ \\ |  < |  __| | . ` |   / /\\ \\| | |_ |  __| | . ` |  | |   ",
            "| . \\| | \\ \\  / ____ \\| . \\| |____| |\\  |  / ____ \\ |__| | |____| |\\  |  | |   ",
            "|_|\\_\\_|  \\_\\/_/    \\_\\_|\\_\\______|_| \\_| /_/    \\_\\_____|______|_| \\_|  |_|   ",
        ];

        let mut lines = Vec::new();
        for (i, text) in logo_lines.iter().enumerate() {
            let color = colors[i % colors.len()];
            lines.push(Line::from(Span::styled(
                *text,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )));
        }

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Render the smaller version of the banner (just tagline or logo)
    pub fn render_mini(&self, frame: &mut Frame, area: Rect) {
        let line = Line::from(vec![
            Span::styled(" ≡ ", Style::default().fg(Color::Rgb(174, 129, 255)).add_modifier(Modifier::BOLD)),
            Span::styled("KRAKEN AGENT", Style::default().fg(Color::Rgb(166, 226, 46)).add_modifier(Modifier::BOLD)),
            Span::styled(" from the abyss ", Style::default().fg(Color::Rgb(117, 113, 94)).add_modifier(Modifier::ITALIC)),
            Span::styled(" ≡ ", Style::default().fg(Color::Rgb(174, 129, 255)).add_modifier(Modifier::BOLD)),
        ]);

        let paragraph = Paragraph::new(line).alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }
}
