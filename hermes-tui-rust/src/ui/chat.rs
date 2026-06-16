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
use crate::ui::cards::CardManager;
use crate::ui::subagent::{SubagentInfo, SubagentList};
use std::collections::HashSet;
/// Chat component for displaying conversation messages
///
/// This component renders a scrollable chat transcript with proper
/// formatting, colors, and timestamps.
/// Chat component for displaying conversation messages
///
/// This component renders a scrollable chat transcript with proper
/// formatting, colors, and timestamps. Supports message selection,
/// inline tool cards, and subagent display.
#[derive(Debug, Clone)]
pub struct ChatComponent {
    /// Messages to display
    messages: Vec<Message>,
    scroll_position: u16,
    /// Visual scroll position with float interpolation (for smooth physics)
    scroll_offset_f32: f32,
    /// Visible height of the chat area in lines
    visible_height: u16,
    /// Chat colors from configuration
    colors: ChatColorsRgb,
    /// Whether to show timestamps
    show_timestamps: bool,
    /// Tracking which system messages are expanded (by `message_id`)
    expanded_systems: HashSet<String>,
    /// Content width used for word-wrap calculations (set each render)
    inner_width: u16,
    /// Currently selected message index (for Normal mode navigation)
    selected_index: Option<usize>,
}

impl ChatComponent {
    /// Create a new chat component with the given colors and timestamp setting
    #[must_use]
    pub fn new(colors: ChatColorsRgb, show_timestamps: bool) -> Self {
        Self {
            messages: Vec::new(),
            scroll_position: 0,
            scroll_offset_f32: 0.0,
            visible_height: 0,
            colors,
            show_timestamps,
            inner_width: 80,
            expanded_systems: HashSet::new(),
            selected_index: None,
        }
    }

    /// Create a new chat component with all defaults
    #[must_use]
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
    #[must_use]
    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    /// Set the visible height of the chat area
    pub fn set_visible_height(&mut self, height: u16) {
        self.visible_height = height;
    }
    /// Set the content width for word-wrap calculations
    pub fn set_inner_width(&mut self, width: u16) {
        self.inner_width = width;
    }
    /// Calculate the height of a message in lines, accounting for word-wrap
    /// and inline rendering (tool cards, subagents).
    #[must_use]
    pub fn message_height(&self, message: &Message, card_manager: &CardManager) -> u16 {
        // Tool messages rendered inline: use card_manager height
        if message.role == MessageRole::Tool && message.message_id.is_some() {
            if let Some(card) = message
                .message_id
                .as_ref()
                .and_then(|id| card_manager.find_by_call_id(id))
            {
                if card.is_expanded() {
                    let w = self.inner_width.saturating_sub(4) as usize;
                    let lines = card
                        .content()
                        .lines()
                        .map(|l| ((l.len() as f64) / w.max(1) as f64).ceil() as u16)
                        .sum::<u16>()
                        .max(1);
                    return lines + 3; // borders + spacing
                }
                return 4; // collapsed tool card
            }
        }

        // Subagent messages: always 2 lines
        if message.role == MessageRole::System
            && message
                .message_id
                .as_deref()
                .is_some_and(|id| id.starts_with("subagent:"))
        {
            return 2;
        }

        let is_user = message.role == MessageRole::User;
        // User messages have border padding (2) + spacing (1) = 3 extra lines
        // Non-user gutter layout has just spacing (1) extra line
        let content_width = if is_user {
            self.inner_width.saturating_sub(2).max(10) as usize
        } else {
            self.inner_width.saturating_sub(4).max(6) as usize
        };
        let mut total_wrapped_lines = 0u16;

        for line in message.content.lines() {
            if line.is_empty() {
                total_wrapped_lines += 1;
            } else if content_width > 0 {
                let line_len = line.len();
                let wrapped = ((line_len as f64) / (content_width as f64)).ceil() as u16;
                total_wrapped_lines += wrapped.max(1);
            } else {
                total_wrapped_lines += 1;
            }
        }

        if total_wrapped_lines == 0 && message.is_streaming() {
            total_wrapped_lines = 1;
        }

        if is_user {
            total_wrapped_lines + 3 // 2 borders + 1 spacing
        } else {
            total_wrapped_lines + 1 // 1 spacing between messages
        }
    }

    /// Scroll down by the given amount (in lines)
    pub fn scroll_down(&mut self, amount: u16, card_manager: &CardManager) {
        let max = self.max_scroll_position(card_manager);
        self.scroll_position = self.scroll_position.saturating_add(amount).min(max);
    }

    /// Scroll up by the given amount (in lines)
    pub fn scroll_up(&mut self, amount: u16) {
        self.scroll_position = self.scroll_position.saturating_sub(amount);
    }
    /// Scroll to the bottom
    pub fn scroll_to_bottom(&mut self, card_manager: &CardManager) {
        self.scroll_position = self.max_scroll_position(card_manager);
    }

    /// Scroll to the top
    pub fn scroll_to_top(&mut self) {
        self.scroll_position = 0;
    }

    /// Get the maximum scroll position
    #[must_use]
    pub fn max_scroll_position(&self, card_manager: &CardManager) -> u16 {
        let total_height: u16 = self
            .messages
            .iter()
            .map(|m| self.message_height(m, card_manager))
            .sum();
        total_height.saturating_sub(self.visible_height)
    }

    /// Get the style for a message role
    #[must_use]
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
    #[must_use]
    pub fn get_role_display(&self, role: MessageRole) -> &'static str {
        match role {
            MessageRole::User => " User ",
            MessageRole::Assistant => " ≡ Hermes ",
            MessageRole::System => " System ",
            MessageRole::Tool => " Tool ",
        }
    }

    /// Format a timestamp as a relative time string
    #[must_use]
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
    #[must_use]
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }
    /// Add a single message
    pub fn add_message(&mut self, message: Message, card_manager: &CardManager) {
        self.messages.push(message);
        // Auto-scroll to bottom when new message is added
        self.scroll_to_bottom(card_manager);
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
    pub fn set_messages(&mut self, messages: Vec<Message>, card_manager: &CardManager) {
        self.messages = messages;
        self.scroll_to_bottom(card_manager);
    }

    /// Toggle expansion of a system message
    pub fn toggle_system_expanded(&mut self, message_id: &str) {
        if !self.expanded_systems.remove(message_id) {
            self.expanded_systems.insert(message_id.to_string());
        }
    }

    /// Get the currently selected message index
    #[must_use]
    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Select the next selectable message (for Normal mode)
    pub fn select_next(&mut self, _card_manager: &CardManager) {
        let len = self.messages.len();
        if len == 0 {
            self.selected_index = None;
            return;
        }
        let next = self.selected_index.map_or(0, |i| i + 1);
        self.selected_index = Some(if next >= len { 0 } else { next });
    }

    /// Select the previous selectable message (for Normal mode)
    pub fn select_prev(&mut self, _card_manager: &CardManager) {
        let len = self.messages.len();
        if len == 0 {
            self.selected_index = None;
            return;
        }
        let prev = self
            .selected_index
            .map_or(len - 1, |i| if i == 0 { len - 1 } else { i - 1 });
        self.selected_index = Some(prev);
    }

    /// Ensure the selected message is visible by scrolling if needed
    pub fn ensure_selected_in_view(&mut self, card_manager: &CardManager) {
        let idx = match self.selected_index {
            Some(i) => i,
            None => return,
        };
        let mut offset = 0u16;
        for (i, msg) in self.messages.iter().enumerate() {
            if i >= idx {
                break;
            }
            offset += self.message_height(msg, card_manager);
        }
        let msg_height = self.message_height(&self.messages[idx], card_manager);
        if offset < self.scroll_position {
            self.scroll_position = offset;
        } else if offset + msg_height > self.scroll_position + self.visible_height {
            let need =
                (offset + msg_height).saturating_sub(self.scroll_position + self.visible_height);
            self.scroll_position = self.scroll_position.saturating_add(need);
        }
    }

    /// Build a display line for a subagent (rendered inline in chat transcript)
    #[must_use]
    pub fn build_subagent_line(&self, agent: &SubagentInfo) -> Line<'static> {
        let (icon, icon_style) = agent.status_style();
        let mut spans = Vec::new();
        spans.push(Span::styled(format!(" {icon} "), icon_style));
        if agent.parent_id.is_some() {
            spans.push(Span::styled("└ ", Style::default().fg(Color::DarkGray)));
        }
        spans.push(Span::styled(
            agent.id.clone(),
            Style::default().fg(Color::Cyan).bold(),
        ));
        let max_goal = 40usize;
        let goal = if agent.goal.len() > max_goal {
            format!(": {}...", &agent.goal[..max_goal.saturating_sub(3)])
        } else {
            format!(": {}", agent.goal)
        };
        spans.push(Span::styled(goal, Style::default().fg(Color::White)));
        if let Some(ref summary) = agent.summary {
            let max_sum = 30usize;
            let s = if summary.len() > max_sum {
                format!(" → {}...", &summary[..max_sum.saturating_sub(3)])
            } else {
                format!(" → {summary}")
            };
            spans.push(Span::styled(s, Style::default().fg(Color::DarkGray)));
        }
        Line::from(spans)
    }

    /// Render the chat component
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        card_manager: &CardManager,
        subagent_list: &SubagentList,
        connected: bool,
    ) {
        if self.messages.is_empty() {
            self.render_empty(frame, area, connected);
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(self.colors.border));

        let inner_area = block.inner(area);
        if inner_area.height == 0 || inner_area.width == 0 {
            frame.render_widget(block, area);
            return;
        }

        self.inner_width = inner_area.width;

        let total_height: u16 = self
            .messages
            .iter()
            .map(|m| self.message_height(m, card_manager))
            .sum();
        let max_scroll = total_height.saturating_sub(inner_area.height);
        let target_scroll = self.scroll_position.min(max_scroll);
        self.scroll_offset_f32 += (f32::from(target_scroll) - self.scroll_offset_f32) * 0.3;
        let current_scroll = self.scroll_offset_f32.round() as u16;

        frame.render_widget(block, area);

        let mut current_y = 0u16;
        let mut start_idx = 0usize;
        let mut start_offset = 0u16;

        for (i, msg) in self.messages.iter().enumerate() {
            let msg_height = self.message_height(msg, card_manager);
            if current_y + msg_height > current_scroll {
                start_idx = i;
                start_offset = current_scroll.saturating_sub(current_y);
                break;
            }
            current_y += msg_height;
        }

        let mut y_offset = 0u16;

        for (msg_idx, message) in self.messages.iter().enumerate().skip(start_idx) {
            if y_offset >= inner_area.height {
                break;
            }

            let msg_height = if message.role == MessageRole::Tool && message.message_id.is_some() {
                // Tool messages use card_manager height for inline rendering
                if let Some(card) = message
                    .message_id
                    .as_ref()
                    .and_then(|id| card_manager.find_by_call_id(id))
                {
                    if card.is_expanded() {
                        let w = self.inner_width.saturating_sub(4) as usize;
                        let lines = card
                            .content()
                            .lines()
                            .map(|l: &str| ((l.len() as f64) / w.max(1) as f64).ceil() as u16)
                            .sum::<u16>()
                            .max(1);
                        lines + 3
                    } else {
                        4
                    }
                } else {
                    self.message_height(message, card_manager)
                }
            } else if message.role == MessageRole::System
                && message
                    .message_id
                    .as_deref()
                    .is_some_and(|id: &str| id.starts_with("subagent:"))
            {
                2
            } else {
                self.message_height(message, card_manager)
            };

            let available_height = inner_area.height.saturating_sub(y_offset);
            let render_height = msg_height
                .saturating_sub(start_offset)
                .min(available_height);

            if render_height > 0 {
                let is_selected = self.selected_index == Some(msg_idx);

                let msg_area = Rect {
                    x: inner_area.x,
                    y: inner_area.y + y_offset,
                    width: inner_area.width,
                    height: render_height,
                };

                let prev_is_tool = if msg_idx > 0 {
                    self.messages
                        .get(msg_idx - 1)
                        .is_some_and(|m| m.role == MessageRole::Tool)
                } else {
                    false
                };

                self.render_message(
                    frame,
                    msg_area,
                    message,
                    prev_is_tool,
                    is_selected,
                    card_manager,
                    subagent_list,
                );
                y_offset += render_height;
            }

            start_offset = 0;
        }

        if total_height > inner_area.height {
            let mut scroll_state =
                ScrollbarState::new(total_height as usize).position(current_scroll as usize);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .style(Style::new().fg(self.colors.border))
                .track_symbol(Some("│"))
                .thumb_symbol("█");

            frame.render_stateful_widget(scrollbar, area, &mut scroll_state);
        }
    }

    /// Render a single message
    fn render_message(
        &self,
        frame: &mut Frame,
        area: Rect,
        message: &Message,
        prev_is_tool: bool,
        is_selected: bool,
        card_manager: &CardManager,
        subagents: &SubagentList,
    ) {
        let role_style = self.get_role_style(message.role.clone());
        let role_glyph = match message.role {
            MessageRole::User => "[U]",
            MessageRole::Assistant => "[A]",
            MessageRole::System => "[S]",
            MessageRole::Tool => "[T]",
        };

        // Check if this message has an inline tool card
        let is_tool_with_card = message.role == MessageRole::Tool
            && message
                .message_id
                .as_ref()
                .is_some_and(|id| card_manager.find_by_call_id(id).is_some());
        // Check if this is a subagent inline message
        let is_subagent_msg = message.role == MessageRole::System
            && message
                .message_id
                .as_deref()
                .is_some_and(|id| id.starts_with("subagent:"));

        if message.role == MessageRole::User {
            // User messages keep the bubble style
            let border_style = Style::new().fg(self.colors.user_bg);
            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .style(role_style);

            if self.show_timestamps {
                let ts = message.timestamp.format("%H:%M:%S").to_string();
                block = block.title_bottom(
                    Line::from(vec![Span::styled(
                        format!(" {ts} "),
                        Style::new()
                            .fg(self.colors.timestamp)
                            .add_modifier(Modifier::ITALIC),
                    )])
                    .alignment(ratatui::layout::Alignment::Right),
                );
            }

            let lines = self.render_message_content(message, area.width.saturating_sub(2));
            let paragraph = Paragraph::new(Text::from(lines))
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: false });
            frame.render_widget(paragraph, area);
        } else {
            // Non-user messages use gutter layout
            let mut y = area.y;
            let mut remaining_height = area.height;

            // Response separator for Assistant after Tool
            if prev_is_tool && message.role == MessageRole::Assistant && remaining_height > 0 {
                let sep = Line::from(Span::styled(
                    " └─ Response ",
                    Style::new()
                        .fg(self.colors.border)
                        .add_modifier(Modifier::DIM),
                ));
                frame.render_widget(Paragraph::new(sep), Rect::new(area.x, y, area.width, 1));
                y += 1;
                remaining_height = remaining_height.saturating_sub(1);
            }

            if remaining_height == 0 {
                return;
            }

            // Gutter: selection indicator or role glyph
            let gutter_str = if is_selected { "▸ " } else { role_glyph };
            let gutter_style = if is_selected {
                Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::new()
                    .fg(match message.role {
                        MessageRole::Assistant => self.colors.assistant_text,
                        MessageRole::System => self.colors.system_text,
                        MessageRole::Tool => self.colors.tool_text,
                        _ => self.colors.border,
                    })
                    .add_modifier(Modifier::BOLD)
            };
            let gutter = Line::from(Span::styled(gutter_str, gutter_style));
            frame.render_widget(
                Paragraph::new(gutter),
                Rect::new(area.x, y, 3, 1.min(remaining_height)),
            );

            if remaining_height == 0 {
                return;
            }

            // Content area to the right of gutter
            let content_width = area.width.saturating_sub(4);
            let content_area = Rect::new(area.x + 3, y, content_width, remaining_height);
            if content_width < 2 || remaining_height == 0 {
                return;
            }

            // Inline tool card rendering
            if is_tool_with_card {
                if let Some(card) = message
                    .message_id
                    .as_ref()
                    .and_then(|id| card_manager.find_by_call_id(id))
                {
                    card.render(frame, content_area);
                    return;
                }
            }

            // Inline subagent rendering
            if is_subagent_msg {
                if let Some(agent) = message
                    .message_id
                    .as_ref()
                    .and_then(|id| id.strip_prefix("subagent:"))
                    .and_then(|agent_id| subagents.agents().iter().find(|a| a.id == agent_id))
                {
                    let line = self.build_subagent_line(agent);
                    frame.render_widget(Paragraph::new(line), content_area);
                    return;
                }
            }

            // Get message content, possibly truncated for system messages
            let display_content = if message.role == MessageRole::System
                && message.content.len() > 400
                && !message
                    .message_id
                    .as_deref()
                    .is_some_and(|id| self.expanded_systems.contains(id))
            {
                format!("{}...\n (press Enter to expand)", &message.content[..397])
            } else {
                message.content.clone()
            };

            let temp_msg = Message {
                content: display_content,
                role: message.role.clone(),
                ..message.clone()
            };
            let lines = self.render_message_content(&temp_msg, content_width);

            let paragraph =
                Paragraph::new(Text::from(lines)).wrap(ratatui::widgets::Wrap { trim: false });
            frame.render_widget(paragraph, content_area);

            // Timestamp on the right edge for non-user messages
            if self.show_timestamps && remaining_height > 0 {
                let ts = message.timestamp.format("%H:%M").to_string();
                let ts_span = Span::styled(
                    format!(" {ts} "),
                    Style::new()
                        .fg(self.colors.timestamp)
                        .add_modifier(Modifier::DIM),
                );
                frame.render_widget(
                    Paragraph::new(Line::from(ts_span)),
                    Rect::new(
                        area.x + area.width.saturating_sub(ts.len() as u16 + 2),
                        y,
                        ts.len() as u16 + 2,
                        1,
                    ),
                );
            }
        }
    }

    /// Render message content with markdown and syntax highlighting
    fn render_message_content<'a>(&self, message: &'a Message, area_width: u16) -> Vec<Line<'a>> {
        let rendered =
            crate::utils::markdown::render_markdown(&message.content, &self.colors, area_width);

        // Post-process rendered markdown to apply syntect highlighting to fenced code blocks
        // and decode base64 images into Sixel graphics.
        let mut result = Vec::new();
        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_lines: Vec<String> = Vec::new();

        for line in rendered {
            let text: String = line.to_string();

            // Check for base64 image data
            if !in_code_block && text.starts_with("data:image/") && text.contains(";base64,") {
                if let Some(b64) = text.split(";base64,").nth(1) {
                    if let Ok(Some(sixel)) =
                        crate::utils::sixel::encode_base64_image(b64.trim(), 400)
                    {
                        // Push the raw sixel string. This is a best-effort inline render.
                        result.push(Line::from(Span::raw(sixel)));
                        continue;
                    }
                }
            }

            if !in_code_block && text.starts_with("```") {
                in_code_block = true;
                code_lang = text.trim_start_matches('`').trim().to_string();
                code_lines.clear();
                continue;
            }
            if in_code_block && text == "```" {
                in_code_block = false;
                result.extend(self.highlight_code(&code_lines, &code_lang));
                code_lines.clear();
                code_lang.clear();
                continue;
            }
            if in_code_block {
                code_lines.push(text);
                continue;
            }
            result.push(line);
        }

        if in_code_block && !code_lines.is_empty() {
            result.extend(self.highlight_code(&code_lines, &code_lang));
        }

        // If streaming, add a cursor
        if message.is_streaming() {
            if result.is_empty() {
                result.push(Line::from(Span::styled(
                    "▊",
                    Style::new().fg(Color::Yellow),
                )));
            } else if let Some(last_line) = result.last_mut() {
                last_line
                    .spans
                    .push(Span::styled("▊", Style::new().fg(Color::Yellow)));
            }
        }

        result
    }

    /// Highlight a code block using the syntax highlighter
    fn highlight_code<'a>(&self, code_lines: &[String], lang: &str) -> Vec<Line<'a>> {
        use crate::utils::syntax::SyntaxHighlighter;

        if code_lines.is_empty() {
            return vec![Line::from(Span::raw("(empty code block)"))];
        }

        let code = code_lines.join("\n");
        let highlighter = SyntaxHighlighter::default();
        let highlighted = highlighter.highlight(&code, Some(lang));

        if highlighted.is_empty() {
            // Fallback: plain text
            code_lines
                .iter()
                .map(|l| Line::from(Span::raw(l.clone())))
                .collect()
        } else {
            highlighted
        }
    }

    /// Render empty state (Landing Page)
    fn render_empty(&self, frame: &mut Frame, area: Rect, connected: bool) {
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
                ratatui::layout::Constraint::Min(1),     // Remaining space for info
            ])
            .split(inner_area);
        // Left: ASCII Art
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

        // Simple time-based animation offset
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as usize;
        let offset = (time / 100) % 6;

        for (i, line) in hero_lines.iter().enumerate() {
            // Color gradient that animates
            let color_idx = (i + offset) % 6;
            let color = match color_idx {
                0 => Color::Rgb(166, 226, 46),  // Neon Green
                1 => Color::Rgb(102, 217, 239), // Cyan
                2 => Color::Rgb(174, 129, 255), // Purple
                3 => Color::Rgb(249, 38, 114),  // Pink
                4 => Color::Rgb(253, 151, 31),  // Orange
                _ => Color::Rgb(117, 113, 94),  // Gray
            };
            hero_spans.push(Line::from(Span::styled(*line, Style::default().fg(color))));
        }

        let hero_para = Paragraph::new(hero_spans)
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(self.colors.border))
                    .padding(Padding::new(2, 2, 2, 2)),
            );
        frame.render_widget(hero_para, layout[0]);

        // Right: Info blocks
        let mut info_text = Vec::new();

        // Version header
        info_text.push(Line::from(vec![Span::styled(
            "Hermes Agent v0.16.0 (2026.6.5) · upstream bd16e524",
            Style::default().fg(Color::Rgb(230, 219, 116)).bold(),
        )]));

        // Connection Warning (if disconnected)
        if !connected {
            info_text.push(Line::from(""));
            let status_span = Span::styled(
                "STATUS: DISCONNECTED (Check hermes-tui.log)",
                Style::default().fg(Color::Rgb(249, 38, 114)).bold(),
            );
            info_text.push(Line::from(status_span).alignment(ratatui::layout::Alignment::Center));
            info_text.push(Line::from(""));
        }

        // Available Tools
        info_text.push(Line::from(Span::styled(
            "Available Tools",
            Style::default().fg(Color::Rgb(230, 219, 116)).bold(),
        )));
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
                Span::styled(
                    format!("  {name}: "),
                    Style::default().fg(Color::Rgb(117, 113, 94)),
                ),
                Span::styled(desc, Style::default().fg(Color::Rgb(248, 248, 242))),
            ]));
        }
        info_text.push(Line::from(Span::styled(
            "  (and 22 more toolsets...)",
            Style::default().fg(Color::Rgb(117, 113, 94)).italic(),
        )));
        info_text.push(Line::from(""));

        // MCP Servers
        info_text.push(Line::from(Span::styled(
            "MCP Servers",
            Style::default().fg(Color::Rgb(230, 219, 116)).bold(),
        )));
        info_text.push(Line::from(vec![
            Span::styled(
                "  playwright (stdio) ",
                Style::default().fg(Color::Rgb(248, 248, 242)),
            ),
            Span::styled(
                "– connecting",
                Style::default().fg(Color::Rgb(230, 219, 116)).italic(),
            ),
        ]));
        info_text.push(Line::from(""));

        // Available Skills
        info_text.push(Line::from(Span::styled(
            "Available Skills",
            Style::default().fg(Color::Rgb(230, 219, 116)).bold(),
        )));
        let skills = [
            ("autonomous-ai-agents", "coding-agents, hermes-agent, ..."),
            ("creative", "architecture-diagram, ascii-art, ..."),
            ("data-science", "jupyter-live-kernel"),
            ("devops", "kanban-orchestrator, kanban-worker, ..."),
            ("email", "himalaya"),
        ];
        for (name, desc) in skills {
            info_text.push(Line::from(vec![
                Span::styled(
                    format!("  {name}: "),
                    Style::default().fg(Color::Rgb(117, 113, 94)),
                ),
                Span::styled(desc, Style::default().fg(Color::Rgb(248, 248, 242))),
            ]));
        }
        info_text.push(Line::from(""));

        // Footer counts
        info_text.push(Line::from(vec![Span::styled(
            "  41 tools · 1321 skills · /help for commands",
            Style::default().fg(Color::Rgb(117, 113, 94)).italic(),
        )]));

        let info_para =
            Paragraph::new(info_text).block(Block::default().padding(Padding::new(4, 0, 0, 0)));
        frame.render_widget(info_para, layout[1]);
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
    assert_eq!(chat.get_role_display(MessageRole::Assistant), " ≡ Hermes ");
    assert_eq!(chat.get_role_display(MessageRole::System), " System ");
    assert_eq!(chat.get_role_display(MessageRole::Tool), " Tool ");
}

#[test]
fn test_message_height() {
    let colors = create_test_colors();
    let mut chat = ChatComponent::new(colors, true);
    chat.set_visible_height(80);

    let card_manager = CardManager::new(ChatColorsRgb {
        user_bg: ratatui::style::Color::Reset,
        user_text: ratatui::style::Color::Reset,
        assistant_bg: ratatui::style::Color::Reset,
        assistant_text: ratatui::style::Color::Reset,
        system_bg: ratatui::style::Color::Reset,
        system_text: ratatui::style::Color::Reset,
        tool_bg: ratatui::style::Color::Reset,
        tool_text: ratatui::style::Color::Reset,
        code_bg: ratatui::style::Color::Reset,
        code_text: ratatui::style::Color::Reset,
        border: ratatui::style::Color::Reset,
        timestamp: ratatui::style::Color::Reset,
    });
    let msg = create_test_message(MessageRole::User, "Hello");
    assert!(chat.message_height(&msg, &card_manager) >= 3);
}

#[test]
fn test_add_message() {
    let colors = create_test_colors();
    let mut chat = ChatComponent::new(colors, true);
    let card_manager = CardManager::new(ChatColorsRgb {
        user_bg: ratatui::style::Color::Reset,
        user_text: ratatui::style::Color::Reset,
        assistant_bg: ratatui::style::Color::Reset,
        assistant_text: ratatui::style::Color::Reset,
        system_bg: ratatui::style::Color::Reset,
        system_text: ratatui::style::Color::Reset,
        tool_bg: ratatui::style::Color::Reset,
        tool_text: ratatui::style::Color::Reset,
        code_bg: ratatui::style::Color::Reset,
        code_text: ratatui::style::Color::Reset,
        border: ratatui::style::Color::Reset,
        timestamp: ratatui::style::Color::Reset,
    });

    let msg = create_test_message(MessageRole::User, "Hello");
    chat.add_message(msg, &card_manager);

    assert_eq!(chat.messages().len(), 1);
}

#[test]
fn test_scroll() {
    let colors = create_test_colors();
    let mut chat = ChatComponent::new(colors, false);
    let card_manager = CardManager::new(ChatColorsRgb {
        user_bg: ratatui::style::Color::Reset,
        user_text: ratatui::style::Color::Reset,
        assistant_bg: ratatui::style::Color::Reset,
        assistant_text: ratatui::style::Color::Reset,
        system_bg: ratatui::style::Color::Reset,
        system_text: ratatui::style::Color::Reset,
        tool_bg: ratatui::style::Color::Reset,
        tool_text: ratatui::style::Color::Reset,
        code_bg: ratatui::style::Color::Reset,
        code_text: ratatui::style::Color::Reset,
        border: ratatui::style::Color::Reset,
        timestamp: ratatui::style::Color::Reset,
    });
    chat.set_visible_height(10);

    for i in 0..20 {
        chat.add_message(
            create_test_message(MessageRole::User, &format!("Message {}", i)),
            &card_manager,
        );
    }

    assert!(chat.max_scroll_position(&card_manager) > 0);

    chat.scroll_to_bottom(&card_manager);
    assert_eq!(
        chat.scroll_position,
        chat.max_scroll_position(&card_manager)
    );

    chat.scroll_to_top();
    assert_eq!(chat.scroll_position, 0);

    chat.scroll_down(5, &card_manager);
    assert!(chat.scroll_position >= 5);

    chat.scroll_up(3);
    assert!(chat.scroll_position >= 2);
}

#[test]
fn test_set_messages() {
    let colors = create_test_colors();
    let card_manager = CardManager::new(ChatColorsRgb {
        user_bg: ratatui::style::Color::Reset,
        user_text: ratatui::style::Color::Reset,
        assistant_bg: ratatui::style::Color::Reset,
        assistant_text: ratatui::style::Color::Reset,
        system_bg: ratatui::style::Color::Reset,
        system_text: ratatui::style::Color::Reset,
        tool_bg: ratatui::style::Color::Reset,
        tool_text: ratatui::style::Color::Reset,
        code_bg: ratatui::style::Color::Reset,
        code_text: ratatui::style::Color::Reset,
        border: ratatui::style::Color::Reset,
        timestamp: ratatui::style::Color::Reset,
    });
    let mut chat = ChatComponent::new(colors, true);

    let messages = vec![
        create_test_message(MessageRole::User, "Hello"),
        create_test_message(MessageRole::Assistant, "World"),
    ];

    chat.set_messages(messages, &card_manager);

    assert_eq!(chat.messages().len(), 2);
}
