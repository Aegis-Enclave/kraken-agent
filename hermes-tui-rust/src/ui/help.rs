//! Help module - Global keybindings help overlay
//!
//! This module provides a modal overlay that displays all available
//! keyboard shortcuts across the application.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Row, Table},
    Frame,
};

pub struct HelpView;

impl HelpView {
    pub fn render(frame: &mut Frame, area: Rect) {
        let help_area = Self::centered_rect(70, 70, area);

        // Clear the background
        frame.render_widget(Clear, help_area);

        let block = Block::default()
            .title(Span::styled(
                " ⌨️  KEYBINDINGS HELP ",
                Style::default()
                    .fg(Color::Rgb(250, 189, 47))
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Color::Rgb(250, 189, 47)))
            .padding(Padding::new(2, 2, 1, 1));

        let inner = block.inner(help_area);
        frame.render_widget(block, help_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(1),    // Table
                Constraint::Length(1), // Footer
            ])
            .split(inner);

        // Header
        let header = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "Hermes TUI v0.1.0",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "ESC",
                    Style::default()
                        .fg(Color::Rgb(250, 189, 47))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" or ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "?",
                    Style::default()
                        .fg(Color::Rgb(250, 189, 47))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to close this help.", Style::default().fg(Color::Gray)),
            ]),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(header, chunks[0]);

        // Keybindings Table
        let rows = vec![
            Row::new(vec!["Global", "", ""]),
            Row::new(vec![
                "  Alt+A",
                "Prefix Mode",
                "Enter tmux-style command mode",
            ]),
            Row::new(vec!["  ?", "Help", "Toggle this help overlay"]),
            Row::new(vec!["  Ctrl+C", "Quit", "Exit the application"]),
            Row::new(vec!["", "", ""]),
            Row::new(vec!["Views", "", ""]),
            Row::new(vec!["  1", "Dashboard", "System metrics and tools"]),
            Row::new(vec!["  2", "IDE", "File explorer and editor"]),
            Row::new(vec!["  3", "Kanban", "Task orchestration board"]),
            Row::new(vec!["  4", "Chat", "Main conversation interface"]),
            Row::new(vec!["", "", ""]),
            Row::new(vec!["Chat & Composer", "", ""]),
            Row::new(vec!["  i", "Insert Mode", "Focus input field"]),
            Row::new(vec!["  Esc", "Normal Mode", "Focus chat history"]),
            Row::new(vec!["  Enter", "Submit", "Send message or execute tool"]),
            Row::new(vec!["  Tab", "Next Pane", "Cycle focus between panes"]),
            Row::new(vec!["  /", "Command", "Enter slash command mode"]),
            Row::new(vec!["", "", ""]),
            Row::new(vec!["IDE View", "", ""]),
            Row::new(vec![
                "  Tab",
                "Switch Focus",
                "Toggle between Tree and Editor",
            ]),
            Row::new(vec![
                "  Enter",
                "Open File",
                "Load selected file into editor",
            ]),
            Row::new(vec!["  j/k", "Navigation", "Move cursor up/down"]),
        ];

        let table = Table::new(
            rows,
            [
                Constraint::Length(15),
                Constraint::Length(20),
                Constraint::Min(10),
            ],
        )
        .header(
            Row::new(vec!["KEY", "ACTION", "DESCRIPTION"]).style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .column_spacing(2)
        .highlight_style(Style::default().fg(Color::Rgb(250, 189, 47)));

        frame.render_widget(table, chunks[1]);

        // Footer
        let footer = Paragraph::new(Span::styled(
            "Inspired by oh-my-pi • Built with Ratatui",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[2]);
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
