//! Chat module - Chat transcript UI component
//!
//! This module provides the chat display component for rendering conversation messages.

use chrono::{DateTime, Utc};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    symbols::scrollbar,
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};

use crate::protocol::types::MessageRole;
use crate::state::{config::ChatColorsRgb, messages::Message};

/// Chat component for displaying conversation messages
///
/// This component renders a scrollable chat transcript with proper
/// formatting, colors, and timestamps.
#[derive(Debug, Clone)]
pub struct ChatComponent {
    /// Messages to display
    messages: Vec<Message>,
    /// Current scroll position (in message height units)
    scroll_position: u16,
    /// Visible height of the chat area in lines
    visible_height: u16,
    /// Chat colors from configuration
    colors: ChatColorsRgb,
    /// Whether to show timestamps
    show_timestamps: bool,
}

impl ChatComponent {
    /// Create a new chat component with the given colors and timestamp setting
    pub fn new(colors: ChatColorsRgb, show_timestamps: bool) -> Self {
        Self {
            messages: Vec::new(),
            scroll_position: 0,
            visible_height: 0,
            colors,
            show_timestamps,
        }
    }

    /// Create a new chat component with all defaults
    pub fn with_defaults() -> Self {
        Self::new(
            ChatColorsRgb {
                user_bg: ratatui::style::Color::Indexed(238),
                user_text: ratatui::style::Color::Indexed(252),
                assistant_bg: ratatui::style::Color::Indexed(236),
                assistant_text: ratatui::style::Color::Indexed(248),
                system_bg: ratatui::style::Color::Indexed(235),
                system_text: ratatui::style::Color::Indexed(245),
                tool_bg: ratatui::style::Color::Indexed(237),
                tool_text: ratatui::style::Color::Indexed(243),
                code_bg: ratatui::style::Color::Indexed(233),
                code_text: ratatui::style::Color::Indexed(252),
                border: ratatui::style::Color::Indexed(240),
                timestamp: ratatui::style::Color::Indexed(246),
            },
            true,
        )
    }

    /// Set the messages to display
    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    /// Set the visible height of the chat area
    pub fn set_visible_height(&mut self, height: u16) {
        self.visible_height = height;
    }

    /// Scroll down by the given amount (in lines)
    pub fn scroll_down(&mut self, amount: u16) {
        let max = self.max_scroll_position();
        self.scroll_position = self.scroll_position.saturating_add(amount).min(max);
    }

    /// Scroll up by the given amount (in lines)
    pub fn scroll_up(&mut self, amount: u16) {
        self.scroll_position = self.scroll_position.saturating_sub(amount);
    }

    /// Scroll to the bottom
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_position = self.max_scroll_position();
    }

    /// Scroll to the top
    pub fn scroll_to_top(&mut self) {
        self.scroll_position = 0;
    }

    /// Get the maximum scroll position
    pub fn max_scroll_position(&self) -> u16 {
        let total_height: u16 = self
            .messages
            .iter()
            .map(|m| self.message_height(m))
            .sum();
        total_height.saturating_sub(self.visible_height)
    }

    /// Calculate the height of a message in lines
    pub fn message_height(&self, message: &Message) -> u16 {
        // Count lines in content
        let mut content_lines = message.content.lines().count() as u16;
        if content_lines == 0 && message.is_streaming() {
            content_lines = 1;
        }

        // Add 2 lines for bubble borders (top/bottom) + 1 line for spacing
        content_lines + 3
    }

    /// Get the style for a message role
    pub fn get_role_style(&self, role: MessageRole) -> Style {
        match role {
            MessageRole::User => Style::new()
                .bg(self.colors.user_bg)
                .fg(self.colors.user_text),
            MessageRole::Assistant => Style::new()
                .bg(self.colors.assistant_bg)
                .fg(self.colors.assistant_text),
            MessageRole::System => Style::new()
                .bg(self.colors.system_bg)
                .fg(self.colors.system_text),
            MessageRole::Tool => Style::new()
                .bg(self.colors.tool_bg)
                .fg(self.colors.tool_text),
        }
    }

    /// Get the display string for a message role
    pub fn get_role_display(&self, role: MessageRole) -> &'static str {
        match role {
            MessageRole::User => " User ",
            MessageRole::Assistant => " ≡ Kraken ",
            MessageRole::System => " System ",
            MessageRole::Tool => " Tool ",
        }
    }

    /// Format a timestamp as a relative time string
    pub fn format_timestamp(&self, timestamp: DateTime<Utc>) -> String {
        let now = Utc::now();
        let duration = now - timestamp;

        if duration.num_seconds() < 60 {
            format!("{}s ago", duration.num_seconds())
        } else if duration.num_minutes() < 60 {
            format!("{}m ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{}h ago", duration.num_hours())
        } else {
            format!("{}d ago", duration.num_days())
        }
    }

    /// Get all messages
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Add a single message
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        // Auto-scroll to bottom when new message is added
        self.scroll_to_bottom();
    }

    /// Update an existing message (for streaming deltas)
    pub fn update_message(&mut self, updated_message: Message) {
        if let Some(index) = self
            .messages
            .iter()
            .position(|m| m.message_id == updated_message.message_id)
        {
            self.messages[index] = updated_message;
        }
    }

    /// Clear all messages
    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.scroll_position = 0;
    }

    /// Set all messages at once
    pub fn set_messages(&mut self, messages: Vec<Message>) {
        self.messages = messages;
        self.scroll_to_bottom();
    }

    /// Render the chat component
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if self.messages.is_empty() {
            self.render_empty(frame, area);
            return;
        }

        // Create a block for the chat area
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(self.colors.border));

        // Inner area for messages
        let inner_area = block.inner(area);
        if inner_area.height == 0 || inner_area.width == 0 {
            frame.render_widget(block, area);
            return;
        }

        // Calculate scroll state using actual inner area height
        let total_height: u16 = self
            .messages
            .iter()
            .map(|m| self.message_height(m))
            .sum();
        
        let max_scroll = total_height.saturating_sub(inner_area.height);
        let current_scroll = self.scroll_position.min(max_scroll);

        // Render the main block
        frame.render_widget(block, area);

        // Find starting message based on scroll position
        let mut current_y = 0u16;
        let mut start_idx = 0usize;
        let mut start_offset = 0u16;
        
        for (i, msg) in self.messages.iter().enumerate() {
            let msg_height = self.message_height(msg);
            if current_y + msg_height > current_scroll {
                start_idx = i;
                start_offset = current_scroll.saturating_sub(current_y);
                break;
            }
            current_y += msg_height;
        }

        // Render visible messages
        let mut y_offset = 0u16;
        
        for message in self.messages.iter().skip(start_idx) {
            if y_offset >= inner_area.height {
                break;
            }
            
            let msg_height = self.message_height(message);
            let available_height = inner_area.height.saturating_sub(y_offset);
            
            // Calculate slice of the message to show
            let render_height = msg_height.saturating_sub(start_offset).min(available_height);
            
            if render_height > 0 {
                // For simplicity in this complex TUI, we render the full message but clip it with a sub-area
                // Ratatui handles clipping automatically
                let msg_area = Rect {
                    x: inner_area.x,
                    y: inner_area.y + y_offset,
                    width: inner_area.width,
                    height: render_height,
                };
                
                self.render_message(frame, msg_area, message);
                y_offset += render_height;
            }
            
            // Reset start_offset after the first message
            start_offset = 0;
        }

        // Render scrollbar
        if total_height > inner_area.height {
            let mut scroll_state = ScrollbarState::new(total_height as usize)
                .position(current_scroll as usize);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .style(Style::new().fg(self.colors.border))
                .track_symbol(Some("│"))
                .thumb_symbol("█");
            
            frame.render_stateful_widget(
                scrollbar,
                area,
                &mut scroll_state,
            );
        }
    }

    /// Render a single message in a bubble
    fn render_message(&self, frame: &mut Frame, area: Rect, message: &Message) {
        let role_style = self.get_role_style(message.role.clone());
        let role_display = self.get_role_display(message.role.clone());

        // Determine border type and style based on role
        let (border_type, border_style) = match message.role {
            MessageRole::Assistant => (BorderType::Thick, Style::new().fg(self.colors.border)),
            MessageRole::User => (BorderType::Rounded, Style::new().fg(self.colors.user_bg)),
            MessageRole::Tool => (BorderType::Rounded, Style::new().fg(self.colors.tool_text)),
            MessageRole::System => (BorderType::Plain, Style::new().fg(self.colors.system_text)),
        };

        // Create the bubble block
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style)
            .title(Span::styled(
                role_display,
                role_style.add_modifier(Modifier::BOLD),
            ));

        // Add tool emoji to title if it's a tool message
        if message.role == MessageRole::Tool {
            let emoji = if message.content.contains("run_shell_command") {
                "🐚"
            } else if message.content.contains("read_file") || message.content.contains("write_file") {
                "📜"
            } else if message.content.contains("grep_search") || message.content.contains("glob") {
                "🔍"
            } else {
                "🛠️"
            };
            block = block.title_bottom(Line::from(vec![Span::raw(" "), Span::raw(emoji), Span::raw(" ")]));
        }

        // Add timestamp to bottom right
        if self.show_timestamps {
            let ts = message.timestamp.format("%H:%M:%S").to_string();
            block = block.title_bottom(
                Line::from(vec![Span::styled(
                    format!(" {} ", ts),
                    Style::new()
                        .fg(self.colors.timestamp)
                        .add_modifier(Modifier::ITALIC),
                )])
                .alignment(ratatui::layout::Alignment::Right),
            );
        }

        let mut lines = Vec::new();
        for line in message.content.lines() {
            lines.push(Line::from(Span::raw(line)));
        }

        // If streaming, add a cursor
        if message.is_streaming() {
            if lines.is_empty() {
                lines.push(Line::from(Span::styled("▊", Style::new().fg(Color::Yellow))));
            } else if let Some(last_line) = lines.last_mut() {
                last_line
                    .spans
                    .push(Span::styled("▊", Style::new().fg(Color::Yellow)));
            }
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Render empty state (Landing Page)
    fn render_empty(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(self.colors.border));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // Layout for landing page: Left (ASCII), Right (Info)
        let layout = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Length(50), // Fixed width for ASCII art
                ratatui::layout::Constraint::Min(1),    // Remaining space for info
            ])
            .split(inner_area);

        // Left: Kraken ASCII Art
        let hero_lines = [
            "⣴⣶⣤⡤⠦⣤⣀⣤⠆     ⣈⣭⣿⣶⣿⣦⣼⣆",
            " ⠉⠻⢿⣿⠿⣿⣿⣶⣦⠤⠄⡠⢾⣿⣿⡿⠋⠉⠉⠻⣿⣿⡛⣦",
            "      ⠈⢿⣿⣟⠦ ⣾⣿⣿⣷    ⠻⠿⢿⣿⣧⣄",
            "       ⣸⣿⣿⢧ ⢻⠻⣿⣿⣷⣄⣀⠄⠢⣀⡀⠈⠙⠿⠄",
            "      ⢠⣿⣿⣿⠈    ⣻⣿⣿⣿⣿⣿⣿⣿⣛⣳⣤⣀⣀",
            " ⢠⣧⣶⣥⡤⢄ ⣸⣿⣿⠘  ⢀⣴⣿⣿⡿⠛⣿⣿⣧⠈⢿⠿⠟⠛⠻⠿⠄",
            "⣰⣿⣿⠛⠻⣿⣿⡦⢹⣿⣷   ⢊⣿⣿⡏  ⢸⣿⣿⡇ ⢀⣠⣄⣾⠄",
            " ⣠⣿⠿⠛ ⢀⣿⣿⣷⠘⢿⣿⣦⡀ ⢸⢿⣿⣿⣄ ⣸⣿⣿⡇⣪⣿⡿⠿⣿⣷⡄",
            " ⠙⠃   ⣼⣿⡟  ⠈⠻⣿⣿⣦⣌⡇⠻⣿⣿⣷⣿⣿⣿ ⣿⣿⡇ ⠛⠻⢷⣄",
            "    ⢻⣿⣿⣄   ⠈⠻⣿⣿⣿⣷⣿⣿⣿⣿⣿⡟ ⠫⢿⣿⡆",
            "     ⠻⣿⣿⣿⣿⣶⣶⣾⣿⣿⣿⣿⣿⣿⣿⣿⡟⢀⣀⣤⣾⡿⠃",
            "                  from the abyss",
        ];

        let mut hero_spans = Vec::new();
        for (i, line) in hero_lines.iter().enumerate() {
            // Kraken-style color gradient
            let color = match i {
                0..=1 => Color::Rgb(166, 226, 46),  // Neon Green
                2..=3 => Color::Rgb(102, 217, 239), // Cyan
                4..=5 => Color::Rgb(174, 129, 255), // Purple
                6..=7 => Color::Rgb(249, 38, 114),  // Pink
                8..=9 => Color::Rgb(253, 151, 31),   // Orange
                _ => Color::Rgb(117, 113, 94),      // Gray
            };
            hero_spans.push(Line::from(Span::styled(*line, Style::default().fg(color))));
        }

        let hero_para = Paragraph::new(hero_spans)
            .alignment(ratatui::layout::Alignment::Left)
            .block(Block::default().padding(Padding::new(2, 0, 4, 0)));
        frame.render_widget(hero_para, layout[0]);

        // Right: Info blocks
        let mut info_text = Vec::new();
        
        // Version header
        info_text.push(Line::from(vec![
            Span::styled("Hermes Agent v0.16.0 (2026.6.5) · upstream bd16e524", Style::default().fg(Color::Rgb(230, 219, 116)).bold()),
        ]));

        // Connection Warning (if disconnected)
        info_text.push(Line::from(""));
        info_text.push(Line::from(Span::styled("  STATUS: DISCONNECTED (Check hermes-tui.log)", Style::default().fg(Color::Rgb(249, 38, 114)).bold())));
        info_text.push(Line::from(""));

        // Available Tools
        info_text.push(Line::from(Span::styled("Available Tools", Style::default().fg(Color::Rgb(230, 219, 116)).bold())));
        let tools = [
            ("browser", "browser_back, browser_click, ..."),
            ("browser-cdp", "browser_cdp, browser_dialog"),
            ("clarify", "clarify"),
            ("code_execution", "execute_code"),
            ("computer_use", "computer_use"),
            ("cronjob", "cronjob"),
            ("delegation", "delegate_task"),
            ("discord", "discord"),
        ];
        for (name, desc) in tools {
            info_text.push(Line::from(vec![
                Span::styled(format!("  {}: ", name), Style::default().fg(Color::Rgb(117, 113, 94))),
                Span::styled(desc, Style::default().fg(Color::Rgb(248, 248, 242))),
            ]));
        }
        info_text.push(Line::from(Span::styled("  (and 22 more toolsets...)", Style::default().fg(Color::Rgb(117, 113, 94)).italic())));
        info_text.push(Line::from(""));

        // MCP Servers
        info_text.push(Line::from(Span::styled("MCP Servers", Style::default().fg(Color::Rgb(230, 219, 116)).bold())));
        info_text.push(Line::from(vec![
            Span::styled("  playwright (stdio) ", Style::default().fg(Color::Rgb(248, 248, 242))),
            Span::styled("– connecting", Style::default().fg(Color::Rgb(230, 219, 116)).italic()),
        ]));
        info_text.push(Line::from(""));

        // Available Skills
        info_text.push(Line::from(Span::styled("Available Skills", Style::default().fg(Color::Rgb(230, 219, 116)).bold())));
        let skills = [
            ("autonomous-ai-agents", "coding-agents, hermes-agent, ..."),
            ("creative", "architecture-diagram, ascii-art, ..."),
            ("data-science", "jupyter-live-kernel"),
            ("devops", "kanban-orchestrator, kanban-worker, ..."),
            ("email", "himalaya"),
        ];
        for (name, desc) in skills {
            info_text.push(Line::from(vec![
                Span::styled(format!("  {}: ", name), Style::default().fg(Color::Rgb(117, 113, 94))),
                Span::styled(desc, Style::default().fg(Color::Rgb(248, 248, 242))),
            ]));
        }
        info_text.push(Line::from(""));

        // Footer counts
        info_text.push(Line::from(vec![
            Span::styled("  41 tools · 1321 skills · /help for commands", Style::default().fg(Color::Rgb(117, 113, 94)).italic()),
        ]));

        let info_para = Paragraph::new(info_text)
            .block(Block::default().padding(Padding::new(4, 0, 0, 0)));
        frame.render_widget(info_para, layout[1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_colors() -> ChatColorsRgb {
        ChatColorsRgb {
            user_bg: ratatui::style::Color::Indexed(238),
            user_text: ratatui::style::Color::Indexed(252),
            assistant_bg: ratatui::style::Color::Indexed(236),
            assistant_text: ratatui::style::Color::Indexed(248),
            system_bg: ratatui::style::Color::Indexed(235),
            system_text: ratatui::style::Color::Indexed(245),
            tool_bg: ratatui::style::Color::Indexed(237),
            tool_text: ratatui::style::Color::Indexed(243),
            code_bg: ratatui::style::Color::Indexed(233),
            code_text: ratatui::style::Color::Indexed(252),
            border: ratatui::style::Color::Indexed(240),
            timestamp: ratatui::style::Color::Indexed(246),
        }
    }

    fn create_test_message(role: MessageRole, content: &str) -> Message {
        Message::new(role, content)
    }

    #[test]
    fn test_new() {
        let colors = create_test_colors();
        let chat = ChatComponent::new(colors, true);
        assert!(chat.messages().is_empty());
        assert_eq!(chat.scroll_position, 0);
    }

    #[test]
    fn test_role_style() {
        let colors = create_test_colors();
        let chat = ChatComponent::new(colors, true);

        let user_style = chat.get_role_style(MessageRole::User);
        assert!(user_style.bg == Some(ratatui::style::Color::Indexed(238)));
        assert!(user_style.fg == Some(ratatui::style::Color::Indexed(252)));
    }

    #[test]
    fn test_role_display() {
        let colors = create_test_colors();
        let chat = ChatComponent::new(colors, false);

        assert_eq!(chat.get_role_display(MessageRole::User), " User ");
        assert_eq!(chat.get_role_display(MessageRole::Assistant), " ≡ Kraken ");
        assert_eq!(chat.get_role_display(MessageRole::System), " System ");
        assert_eq!(chat.get_role_display(MessageRole::Tool), " Tool ");
    }

    #[test]
    fn test_message_height() {
        let colors = create_test_colors();
        let mut chat = ChatComponent::new(colors, true);
        chat.set_visible_height(80);

        let msg = create_test_message(MessageRole::User, "Hello");
        assert!(chat.message_height(&msg) >= 3);
    }

    #[test]
    fn test_add_message() {
        let colors = create_test_colors();
        let mut chat = ChatComponent::new(colors, true);

        let msg = create_test_message(MessageRole::User, "Hello");
        chat.add_message(msg);

        assert_eq!(chat.messages().len(), 1);
    }

    #[test]
    fn test_scroll() {
        let colors = create_test_colors();
        let mut chat = ChatComponent::new(colors, false);
        chat.set_visible_height(10);

        for i in 0..20 {
            chat.add_message(create_test_message(
                MessageRole::User,
                &format!("Message {}", i),
            ));
        }

        assert!(chat.max_scroll_position() > 0);

        chat.scroll_to_bottom();
        assert_eq!(chat.scroll_position, chat.max_scroll_position());

        chat.scroll_to_top();
        assert_eq!(chat.scroll_position, 0);

        chat.scroll_down(5);
        assert!(chat.scroll_position >= 5);

        chat.scroll_up(3);
        assert!(chat.scroll_position >= 2);
    }

    #[test]
    fn test_clear_messages() {
        let colors = create_test_colors();
        let mut chat = ChatComponent::new(colors, true);

        chat.add_message(create_test_message(MessageRole::User, "Hello"));
        chat.add_message(create_test_message(MessageRole::Assistant, "World"));

        assert_eq!(chat.messages().len(), 2);

        chat.clear_messages();

        assert!(chat.messages().is_empty());
        assert_eq!(chat.scroll_position, 0);
    }

    #[test]
    fn test_set_messages() {
        let colors = create_test_colors();
        let mut chat = ChatComponent::new(colors, true);

        let messages = vec![
            create_test_message(MessageRole::User, "Hello"),
            create_test_message(MessageRole::Assistant, "World"),
        ];

        chat.set_messages(messages);

        assert_eq!(chat.messages().len(), 2);
    }

    #[test]
    fn test_format_timestamp() {
        let colors = create_test_colors();
        let chat = ChatComponent::new(colors, true);

        // Create a timestamp from 5 seconds ago
        let timestamp = Utc::now() - chrono::Duration::seconds(5);
        let result = chat.format_timestamp(timestamp);

        // Should contain "s ago"
        assert!(result.contains("s ago"));
    }

    #[test]
    fn test_with_defaults() {
        let chat = ChatComponent::with_defaults();
        assert!(chat.messages().is_empty());
        assert!(chat.show_timestamps);
    }

    #[test]
    fn test_with_messages() {
        let colors = create_test_colors();
        let messages = vec![
            create_test_message(MessageRole::User, "Hello"),
            create_test_message(MessageRole::Assistant, "World"),
        ];

        let chat = ChatComponent::new(colors, true).with_messages(messages);

        assert_eq!(chat.messages().len(), 2);
    }
}
