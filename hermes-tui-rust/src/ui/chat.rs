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
use crate::state::{capabilities::Capabilities, config::ChatColorsRgb, messages::Message};
use crate::ui::cards::CardManager;
use crate::ui::subagent::{SubagentInfo, SubagentList};
use std::cell::{Cell, RefCell};
use std::collections::HashSet;

/// Chat state for displaying conversation messages
///
/// This struct holds the mutable state of the chat component,
/// such as scroll position and selection.
#[derive(Debug, Clone, Default)]
pub struct ChatState {
    pub scroll_position: u16,
    /// Visual scroll position with float interpolation (for smooth physics)
    pub scroll_offset_f32: f32,
    /// Visible height of the chat area in lines
    pub visible_height: u16,
    /// Content width used for word-wrap calculations (set each render)
    pub inner_width: u16,
    /// Currently selected message index (for Normal mode navigation)
    pub selected_index: Option<usize>,
    /// Tracking which system messages are expanded (by `message_id`)
    pub expanded_systems: HashSet<String>,
    /// Number of new messages that arrived while the user was scrolled
    /// up. Drives the "↓ N new" pill.
    pub pending_new_count: u16,
    /// Whether the scroll is currently at the bottom. Recomputed on each
    /// render based on `scroll_position + visible_height` vs total height.
    pub at_bottom: bool,
}

impl ChatState {
    pub fn new() -> Self {
        Self::default()
    }
}

// ============================================================================
// Message actions (Phase 5)
// ============================================================================

/// An action that can be performed on a chat message. The component decides
/// which actions are available for which role; the app loop performs them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatAction {
    /// Copy the message content to the clipboard.
    Copy,
    /// Replace the composer input with the message content.
    YankToComposer,
    /// Truncate the session at the message and re-submit.
    Regenerate,
    /// Edit a user message in place; resubmit on save.
    Edit,
    /// Drop this message and everything after it.
    Delete,
    /// Create a new session branched at this message.
    Branch,
}

/// What the chat component wants the app to do. Returned by
/// `ChatComponent::perform_action`; consumed by the app loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatCommand {
    /// Insert this text into the composer buffer.
    Yank(String),
    /// Copy this text to the clipboard.
    Copy(String),
    /// Edit a message: replace its content and resubmit.
    Edit {
        message_id: String,
        new_content: String,
    },
    /// Re-submit by truncating the session at the given message index.
    Regenerate { from_index: usize },
    /// Drop everything from the given index forward.
    Delete { from_index: usize },
    /// Create a new session branched at the given message id with this content.
    Branch {
        from_message_id: String,
        content: String,
    },
    /// No action.
    Noop,
}

#[derive(Debug)]
pub struct ChatComponent {
    /// Messages to display
    messages: Vec<Message>,
    /// Chat colors from configuration
    colors: ChatColorsRgb,
    /// Whether to show timestamps
    show_timestamps: bool,
    /// Whether to show the Kraken ASCII logo when the chat is empty
    show_logo_on_empty: Cell<bool>,
    /// Monotonic version counter, bumped on any message mutation.
    height_version: u64,
    /// Cached per-message heights; `RefCell` because render is `&self`.
    height_cache: RefCell<HeightCache>,
    /// Gateway-reported capabilities shown on the empty-state landing page.
    capabilities: RefCell<Capabilities>,
}

/// Cached per-message heights used by the chat renderer.
#[derive(Debug, Clone, Default)]
struct HeightCache {
    version: u64,
    width: u16,
    heights: Vec<u16>,
    total: u64,
}

impl ChatComponent {
    /// Create a new chat component with the given colors and timestamp setting
    #[must_use]
    pub fn new(colors: ChatColorsRgb, show_timestamps: bool) -> Self {
        Self {
            messages: Vec::new(),
            colors,
            show_timestamps,
            show_logo_on_empty: Cell::new(true),
            height_version: 0,
            height_cache: RefCell::new(HeightCache::default()),
            capabilities: RefCell::new(Capabilities::default()),
        }
    }

    /// Create a new chat component with all defaults
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(ChatColorsRgb::default(), true)
    }

    /// Set the messages to display
    #[must_use]
    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self.height_version = self.height_version.wrapping_add(1);
        self
    }

    /// Calculate the height of a message in lines, accounting for word-wrap
    /// and inline rendering (tool cards, subagents).
    #[must_use]
    pub fn message_height(
        &self,
        message: &Message,
        card_manager: &CardManager,
        inner_width: u16,
    ) -> u16 {
        // Tool messages rendered inline: use card_manager height
        if message.role == MessageRole::Tool && message.message_id.is_some() {
            if let Some(card) = message
                .message_id
                .as_ref()
                .and_then(|id| card_manager.find_by_call_id(id))
            {
                if card.is_expanded() {
                    let w = inner_width.saturating_sub(4) as usize;
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

        // Subagent messages render 1 line per wrapped chunk (1-2 today).
        if message.role == MessageRole::System
            && message
                .message_id
                .as_deref()
                .is_some_and(|id| id.starts_with("subagent:"))
        {
            // 1 base line; +1 if there's a summary (we can't know without the
            // subagent list, so the render path recomputes if needed). Use 1
            // for the cache and let the height cache invalidate if the
            // summary shows up. 2 is the upper bound used elsewhere.
            return 2;
        }

        let is_user = message.role == MessageRole::User;
        let is_streaming_assistant =
            message.role == MessageRole::Assistant && message.is_streaming();

        // User messages have border padding (2) + spacing (1) = 3 extra lines
        // Assistant streaming also has a border.
        let mut content_width = if is_user {
            inner_width.saturating_sub(2).max(10) as usize
        } else {
            inner_width.saturating_sub(4).max(6) as usize
        };

        if is_streaming_assistant {
            content_width = content_width.saturating_sub(2).max(4);
        }

        let mut total_wrapped_lines = 0u16;

        // Add reasoning height if present
        if let Some(reasoning) = &message.reasoning {
            let r_width = content_width.saturating_sub(2).max(4);
            let wrapped = crate::utils::text::wrap_display_cells(reasoning, r_width);
            total_wrapped_lines += (wrapped.len() + 2) as u16; // 2 for borders
            total_wrapped_lines += 1; // Spacing after reasoning block
        }

        for line in message.content.lines() {
            if line.is_empty() {
                total_wrapped_lines += 1;
            } else if content_width > 0 {
                let line_cells = crate::utils::text::display_width(line);
                let wrapped = ((line_cells as f64) / (content_width as f64)).ceil() as u16;
                total_wrapped_lines += wrapped.max(1);
            } else {
                total_wrapped_lines += 1;
            }
        }

        // Add warning height if present
        if let Some(warning) = &message.warning {
            total_wrapped_lines += 1; // Blank line before warning
            let line_cells = crate::utils::text::display_width(&format!(" ⚠️ Warning: {warning}"));
            let wrapped = ((line_cells as f64) / (content_width as f64)).ceil() as u16;
            total_wrapped_lines += wrapped.max(1);
        }

        if total_wrapped_lines == 0 && message.is_streaming() {
            total_wrapped_lines = 1;
        }

        if is_user || (message.role == MessageRole::Assistant && message.is_streaming()) {
            total_wrapped_lines + 3 // 2 borders + 1 spacing
        } else {
            total_wrapped_lines + 1 // 1 spacing between messages
        }
    }

    /// Scroll down by the given amount (in lines)
    pub fn scroll_down(&self, state: &mut ChatState, amount: u16, card_manager: &CardManager) {
        let max = self.max_scroll_position(state, card_manager);
        state.scroll_position = state.scroll_position.saturating_add(amount).min(max);
    }

    /// Scroll up by the given amount (in lines)
    pub fn scroll_up(&self, state: &mut ChatState, amount: u16) {
        state.scroll_position = state.scroll_position.saturating_sub(amount);
        self.mark_user_scrolled(state);
    }

    /// Scroll to the bottom (also clears the "↓ N new" pill).
    pub fn scroll_to_bottom(&self, state: &mut ChatState, card_manager: &CardManager) {
        state.scroll_position = self.max_scroll_position(state, card_manager);
        state.at_bottom = true;
        state.pending_new_count = 0;
    }

    /// Scroll to the top
    pub fn scroll_to_top(&self, state: &mut ChatState) {
        state.scroll_position = 0;
        self.mark_user_scrolled(state);
    }

    /// Jump to the bottom (clears pending counter) — for the pill click and
    /// the `G` keybinding.
    pub fn jump_to_bottom(&self, state: &mut ChatState, card_manager: &CardManager) {
        self.scroll_to_bottom(state, card_manager);
    }

    /// Get the maximum scroll position
    #[must_use]
    pub fn max_scroll_position(&self, state: &ChatState, card_manager: &CardManager) -> u16 {
        let total_height: u16 = self
            .messages
            .iter()
            .map(|m| self.message_height(m, card_manager, state.inner_width))
            .sum();
        total_height.saturating_sub(state.visible_height)
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
    pub fn add_message(
        &mut self,
        message: Message,
        state: &mut ChatState,
        card_manager: &CardManager,
    ) {
        self.messages.push(message);
        self.bump_height_version();
        // Only auto-scroll to bottom if the user is currently at the bottom.
        // Otherwise, increment the pending count and let the user click
        // the "↓ N new" pill (or press `G`) to jump down.
        if state.at_bottom {
            self.scroll_to_bottom(state, card_manager);
        } else {
            state.pending_new_count = state.pending_new_count.saturating_add(1);
        }
    }

    /// Update an existing message (for streaming deltas)
    pub fn update_message(&mut self, updated_message: Message) {
        if let Some(index) = self
            .messages
            .iter()
            .position(|m| m.message_id == updated_message.message_id)
        {
            self.messages[index] = updated_message;
            self.bump_height_version();
        }
    }

    /// Mark that the user has scrolled the chat (e.g. via wheel/key). Disables
    /// auto-scroll-on-new-message until they return to the bottom.
    pub fn mark_user_scrolled(&self, state: &mut ChatState) {
        state.at_bottom = false;
    }

    /// Clear all messages
    pub fn clear_messages(&mut self, state: &mut ChatState) {
        self.messages.clear();
        self.bump_height_version();
        self.height_cache.replace(HeightCache::default());
        state.scroll_position = 0;
        state.scroll_offset_f32 = 0.0;
    }

    /// Set all messages at once
    pub fn set_messages(
        &mut self,
        messages: Vec<Message>,
        state: &mut ChatState,
        card_manager: &CardManager,
    ) {
        self.messages = messages;
        self.bump_height_version();
        self.scroll_to_bottom(state, card_manager);
    }

    /// Bump the height-cache invalidation version.
    fn bump_height_version(&mut self) {
        self.height_version = self.height_version.wrapping_add(1);
    }

    /// Ensure the height cache is up to date for the given width; recompute if not.
    /// Takes `&self` by using interior mutability on the cache.
    fn ensure_heights(&self, inner_width: u16, card_manager: &CardManager) {
        {
            let cache = self.height_cache.borrow();
            if cache.version == self.height_version
                && cache.width == inner_width
                && cache.heights.len() == self.messages.len()
            {
                return;
            }
        }
        let mut new_heights = Vec::with_capacity(self.messages.len());
        let mut total: u64 = 0;
        for m in &self.messages {
            let h = u64::from(self.message_height(m, card_manager, inner_width));
            new_heights.push(h as u16);
            total = total.saturating_add(h);
        }
        self.height_cache.replace(HeightCache {
            version: self.height_version,
            width: inner_width,
            heights: new_heights,
            total,
        });
    }

    /// Read the cached heights (used by render).
    fn cached_heights(&self) -> std::cell::Ref<'_, HeightCache> {
        self.height_cache.borrow()
    }

    /// Set whether to show the logo when chat is empty
    pub fn set_show_logo_on_empty(&self, show: bool) {
        self.show_logo_on_empty.set(show);
    }

    /// Update the capabilities shown on the empty-state landing page.
    pub fn set_capabilities(&self, capabilities: Capabilities) {
        self.capabilities.replace(capabilities);
    }

    /// Toggle expansion of a system message
    pub fn toggle_system_expanded(&self, state: &mut ChatState, message_id: &str) {
        if !state.expanded_systems.remove(message_id) {
            state.expanded_systems.insert(message_id.to_string());
        }
    }

    /// Get the currently selected message index
    #[must_use]
    pub fn get_selected_index(&self, state: &ChatState) -> Option<usize> {
        state.selected_index
    }

    /// Select the next selectable message (for Normal mode)
    pub fn select_next(&self, state: &mut ChatState, _card_manager: &CardManager) {
        let len = self.messages.len();
        if len == 0 {
            state.selected_index = None;
            return;
        }
        let next = state.selected_index.map_or(0, |i| i + 1);
        state.selected_index = Some(if next >= len { 0 } else { next });
    }

    /// Select the previous selectable message (for Normal mode)
    pub fn select_prev(&self, state: &mut ChatState, _card_manager: &CardManager) {
        let len = self.messages.len();
        if len == 0 {
            state.selected_index = None;
            return;
        }
        let prev = state
            .selected_index
            .map_or(len - 1, |i| if i == 0 { len - 1 } else { i - 1 });
        state.selected_index = Some(prev);
    }

    /// Ensure the selected message is visible by scrolling if needed
    pub fn ensure_selected_in_view(&self, state: &mut ChatState, card_manager: &CardManager) {
        let idx = match state.selected_index {
            Some(i) => i,
            None => return,
        };
        let mut offset = 0u16;
        for (i, msg) in self.messages.iter().enumerate() {
            if i >= idx {
                break;
            }
            offset += self.message_height(msg, card_manager, state.inner_width);
        }
        let msg_height = self.message_height(&self.messages[idx], card_manager, state.inner_width);
        if offset < state.scroll_position {
            state.scroll_position = offset;
        } else if offset + msg_height > state.scroll_position + state.visible_height {
            let need =
                (offset + msg_height).saturating_sub(state.scroll_position + state.visible_height);
            state.scroll_position = state.scroll_position.saturating_add(need);
        }
    }

    /// Build display lines for a subagent (rendered inline in chat transcript).
    /// The lines wrap cell-aware to `max_width`.
    #[must_use]
    pub fn build_subagent_lines(&self, agent: &SubagentInfo, max_width: u16) -> Vec<Line<'static>> {
        let (icon, icon_style) = agent.status_style();
        // Hard cap (in display cells) for the goal and summary text. Long
        // values are truncated with an ellipsis to keep the line scannable.
        let max_goal = max_width.saturating_sub(20) as usize;
        let goal_text = if max_goal == 0 {
            String::new()
        } else {
            let g = &agent.goal;
            crate::utils::text::truncate_to_cells(g, max_goal, "…")
        };
        let summary_text = agent.summary.as_deref().map(|s| {
            crate::utils::text::truncate_to_cells(s, max_width.saturating_sub(20) as usize, "…")
        });

        // Build the fixed prefix: " icon [tree] id: <goal>" — keep the goal on
        // the first line so the agent ID stays attached.
        let mut first_spans: Vec<Span<'static>> = Vec::new();
        first_spans.push(Span::styled(format!(" {icon} "), icon_style));
        if agent.parent_id.is_some() {
            first_spans.push(Span::styled("└ ", Style::default().fg(self.colors.border)));
        }
        first_spans.push(Span::styled(
            agent.id.clone(),
            Style::default().fg(self.colors.tool_text).bold(),
        ));
        first_spans.push(Span::styled(
            format!(": {goal_text}"),
            Style::default().fg(self.colors.assistant_text),
        ));

        let first_line = Line::from(first_spans);

        // Optional summary as a second line.
        let summary_line = summary_text.map(|s| {
            Line::from(Span::styled(
                format!(" → {s}"),
                Style::default().fg(self.colors.timestamp),
            ))
        });

        let mut out = vec![first_line];
        if let Some(sl) = summary_line {
            out.push(sl);
        }
        out
    }

    /// Render the chat component
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &mut ChatState,
        card_manager: &CardManager,
        subagent_list: &SubagentList,
        connected: bool,
        animation_frame: u64,
    ) {
        if self.messages.is_empty() {
            let caps = self.capabilities.borrow().clone();
            self.render_empty(frame, area, &caps, connected, animation_frame);
            return;
        }

        let inner_area = area;
        if inner_area.height == 0 || inner_area.width == 0 {
            return;
        }

        state.inner_width = inner_area.width;
        state.visible_height = inner_area.height;

        // Refresh height cache (cheap no-op if already valid).
        self.ensure_heights(state.inner_width, card_manager);
        let cache = self.cached_heights();
        let total_height = cache.total.min(u16::MAX as u64) as u16;
        drop(cache);

        let max_scroll = total_height.saturating_sub(inner_area.height);
        let target_scroll = state.scroll_position.min(max_scroll);
        // Compute "at bottom" after clamping. A 2-line slop keeps tiny float
        // drift from breaking the sticky-bottom detection.
        state.at_bottom = target_scroll + state.visible_height + 2 >= total_height;
        state.scroll_offset_f32 += (f32::from(target_scroll) - state.scroll_offset_f32) * 0.3;
        let current_scroll = state.scroll_offset_f32.round() as u16;

        let mut current_y = 0u16;
        let mut start_idx = 0usize;
        let mut start_offset = 0u16;

        {
            let cache = self.cached_heights();
            for (i, msg_height) in cache.heights.iter().enumerate() {
                if current_y + *msg_height > current_scroll {
                    start_idx = i;
                    start_offset = current_scroll.saturating_sub(current_y);
                    break;
                }
                current_y += *msg_height;
            }
        }

        let mut y_offset = 0u16;

        for (msg_idx, message) in self.messages.iter().enumerate().skip(start_idx) {
            if y_offset >= inner_area.height {
                break;
            }

            let cached_height = self.cached_heights().heights[msg_idx];
            let msg_height = if message.role == MessageRole::Tool && message.message_id.is_some() {
                // Tool messages use card_manager height for inline rendering
                if let Some(card) = message
                    .message_id
                    .as_ref()
                    .and_then(|id| card_manager.find_by_call_id(id))
                {
                    if card.is_expanded() {
                        let w = state.inner_width.saturating_sub(4) as usize;
                        let lines = card
                            .content()
                            .lines()
                            .map(|l: &str| {
                                let cells = crate::utils::text::display_width(l);
                                ((cells as f64) / w.max(1) as f64).ceil() as u16
                            })
                            .sum::<u16>()
                            .max(1);
                        lines + 3
                    } else {
                        4
                    }
                } else {
                    cached_height
                }
            } else if message.role == MessageRole::System
                && message
                    .message_id
                    .as_deref()
                    .is_some_and(|id: &str| id.starts_with("subagent:"))
            {
                2
            } else {
                cached_height
            };

            let available_height = inner_area.height.saturating_sub(y_offset);
            let render_height = msg_height
                .saturating_sub(start_offset)
                .min(available_height);

            if render_height > 0 {
                let is_selected = state.selected_index == Some(msg_idx);

                let msg_area = Rect {
                    x: inner_area.x,
                    y: inner_area.y + y_offset,
                    width: inner_area.width,
                    height: render_height,
                };

                // Paint selection background first so the message renders on top.
                if is_selected && render_height > 0 {
                    let bg = Paragraph::new("").style(Style::new().bg(self.colors.selection_bg));
                    frame.render_widget(bg, msg_area);
                }

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
                    state,
                    card_manager,
                    subagent_list,
                    animation_frame,
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

        // "↓ N new" pill (sticky-bottom companion).
        if state.pending_new_count > 0 {
            let n = state.pending_new_count;
            let label = format!(" ↓ {n} new ⏎ ");
            let label_width = crate::utils::text::display_width(&label) as u16;
            let pill_w = label_width + 2;
            let pill_w = pill_w.min(area.width);
            let pill_area = Rect {
                x: area.x + area.width.saturating_sub(pill_w + 1),
                y: area.y + area.height.saturating_sub(1),
                width: pill_w,
                height: 1,
            };
            let pill_style = Style::new()
                .fg(self.colors.pill_fg)
                .bg(self.colors.pill_bg)
                .add_modifier(Modifier::BOLD);
            let pill = Paragraph::new(label)
                .style(pill_style)
                .alignment(ratatui::layout::Alignment::Right);
            frame.render_widget(pill, pill_area);
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
        state: &ChatState,
        card_manager: &CardManager,
        subagents: &SubagentList,
        animation_frame: u64,
    ) {
        let role_style = self.get_role_style(message.role.clone());
        let role_glyph: &str = match message.role {
            MessageRole::User => &self.colors.role_glyph_user,
            MessageRole::Assistant => &self.colors.role_glyph_assistant,
            MessageRole::System => &self.colors.role_glyph_system,
            MessageRole::Tool => &self.colors.role_glyph_tool,
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
            let border_style = Style::new().fg(self.colors.user_bubble_border);
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
                        .fg(self.colors.accent_response_separator)
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
            let gutter_str = if is_selected { "▶ " } else { role_glyph };
            let gutter_style = if is_selected {
                Style::new()
                    .fg(self.colors.selection_accent)
                    .add_modifier(Modifier::BOLD)
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
            // For selected messages, paint a 2-cell accent bar to the left of
            // the gutter.
            if is_selected && area.width >= 6 {
                let bar = "██".repeat(remaining_height as usize);
                let bar_line = Line::from(Span::styled(
                    bar,
                    Style::new()
                        .fg(self.colors.selection_accent)
                        .add_modifier(Modifier::BOLD),
                ));
                frame.render_widget(
                    Paragraph::new(bar_line),
                    Rect::new(area.x, y, 2, remaining_height),
                );
            }
            frame.render_widget(
                Paragraph::new(gutter),
                Rect::new(area.x, y, 3, 1.min(remaining_height)),
            );

            if remaining_height == 0 {
                return;
            }

            // Content area to the right of gutter (and accent bar if selected)
            let gutter_offset = if is_selected { 6 } else { 4 };
            let mut content_width = area.width.saturating_sub(gutter_offset);
            let content_x_offset = if is_selected { 5 } else { 3 };

            // Add reasoning block if present (Phase 4 DSL extensions)
            if let Some(reasoning) = &message.reasoning {
                let r_width = content_width.saturating_sub(2).max(4);
                let r_lines =
                    crate::utils::markdown::render_markdown(reasoning, &self.colors, r_width);
                let r_height = (r_lines.len() + 2) as u16;
                let r_rect = Rect {
                    x: area.x + content_x_offset,
                    y,
                    width: content_width,
                    height: r_height.min(remaining_height),
                };

                let r_block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(self.colors.accent_reasoning).dim())
                    .title(Line::from(vec![Span::styled(
                        " 🧠 Reasoning ",
                        Style::default().fg(self.colors.accent_reasoning).bold(),
                    )]));

                let r_para = Paragraph::new(Text::from(r_lines))
                    .block(r_block)
                    .wrap(ratatui::widgets::Wrap { trim: false });

                frame.render_widget(r_para, r_rect);

                let consumed = r_height.min(remaining_height);
                y += consumed;
                remaining_height = remaining_height.saturating_sub(consumed);

                if remaining_height > 0 {
                    y += 1;
                    remaining_height = remaining_height.saturating_sub(1);
                }
            }

            let mut content_area = Rect::new(
                area.x + content_x_offset,
                y,
                content_width,
                remaining_height,
            );
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
                    card.render(frame, content_area, animation_frame);
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
                    let lines = self.build_subagent_lines(agent, content_width);
                    let para = Paragraph::new(lines);
                    frame.render_widget(para, content_area);
                    return;
                }
            }

            // Animated border for streaming assistant messages
            if message.role == MessageRole::Assistant && message.is_streaming() {
                let border_block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(self.colors.assistant_text).dim());

                frame.render_widget(border_block.clone(), content_area);
                crate::ui::borders::render_gradient_border(
                    frame.buffer_mut(),
                    content_area,
                    animation_frame,
                    true,
                    true,
                );

                content_area = border_block.inner(content_area);
                content_width = content_area.width;
            }

            // Get message content, possibly truncated for system messages
            let display_content = if message.role == MessageRole::System
                && message.content.chars().count() > 400
                && !message
                    .message_id
                    .as_deref()
                    .is_some_and(|id| state.expanded_systems.contains(id))
            {
                let truncated = crate::utils::text::truncate_safe(&message.content, 397);
                format!("{}...\n (press Enter to expand)", truncated)
            } else {
                message.content.clone()
            };

            let temp_msg = Message {
                content: display_content,
                role: message.role.clone(),
                ..message.clone()
            };
            let mut lines = self.render_message_content(&temp_msg, content_width);

            // Add warning if present
            if let Some(warning) = &message.warning {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled(
                        " ⚠️ Warning: ",
                        Style::default().fg(self.colors.accent_warning).bold(),
                    ),
                    Span::styled(warning, Style::default().fg(self.colors.accent_warning)),
                ]));
            }

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
                    Style::new().fg(self.colors.selection_accent),
                )));
            } else if let Some(last_line) = result.last_mut() {
                last_line.spans.push(Span::styled(
                    "▊",
                    Style::new().fg(self.colors.selection_accent),
                ));
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
    fn render_empty(
        &self,
        frame: &mut Frame,
        area: Rect,
        capabilities: &Capabilities,
        connected: bool,
        animation_frame: u64,
    ) {
        let inner_area = area;

        // Build info text (shared between logo and no-logo modes)
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
            format!(
                "  {} tools · {} skills · /help for commands",
                capabilities.tool_count, capabilities.skill_count
            ),
            Style::default().fg(Color::Rgb(117, 113, 94)).italic(),
        )]));

        if self.show_logo_on_empty.get() {
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
                " ⢠⣧⣶⣥⡤⣄ ⣸⣿⣿⠘  ⢀⣴⣿⣿⡿⠛⣿⣿⣧⠈⢿⠿⠟⠛⠻⠿⠄",
                "⣰⣿⣿⠛⠻⣿⣿⡦⢹⣿⣷   ⢊⣿⣿⡏  ⢸⣿⣿⡇ ⢀⣠⣄⣾⠄",
                " ⣠⣿⠿⠛ ⢀⣿⣿⣷⠘⢿⣿⣦⡀ ⢸⢿⣿⣿⣄ ⣸⣿⣿⡇⣪⣿⡿⠿⣿⣷⡄",
                " ⠙⠃   ⣼⣿⡟  ⠈⠻⣿⣿⣦⣌⡇⠻⣿⣿⣷⣿⣿⣿ ⣿⣿⡇ ⠛⠻⢷⣄",
                "    ⢻⣿⣿⣄   ⠈⠻⣿⣿⣿⣷⣿⣿⣿⣿⣿⡟ ⠫⢿⣿⡆",
                "     ⠻⣿⣿⣿⣿⣶⣶⣾⣿⣿⣿⣿⣿⣿⣿⣿⡟⢀⣀⣤⣾⡿⠃",
                " from the abyss",
            ];

            let mut hero_spans = Vec::new();

            // Animation offset derived from the render frame so the hero
            // pauses with the rest of the UI when animations halt.
            let offset = (animation_frame as usize / 4) % 6;

            for (i, line) in hero_lines.iter().enumerate() {
                let color_idx = (i + offset) % 6;
                let color = match color_idx {
                    0 => Color::Rgb(166, 226, 46),
                    1 => Color::Rgb(102, 217, 239),
                    2 => Color::Rgb(174, 129, 255),
                    3 => Color::Rgb(249, 38, 114),
                    4 => Color::Rgb(253, 151, 31),
                    _ => Color::Rgb(117, 113, 94),
                };
                hero_spans.push(Line::from(Span::styled(*line, Style::default().fg(color))));
            }

            let hero_para = Paragraph::new(hero_spans)
                .alignment(ratatui::layout::Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Double)
                        .border_style(Style::default().fg(self.colors.hero_border))
                        .padding(Padding::new(2, 2, 2, 2)),
                );
            frame.render_widget(hero_para, layout[0]);

            // Info panel on the right
            let info_para =
                Paragraph::new(info_text).block(Block::default().padding(Padding::new(4, 0, 0, 0)));
            frame.render_widget(info_para, layout[1]);
        } else {
            // No logo — just show the info panel using the full area
            let info_para =
                Paragraph::new(info_text).block(Block::default().padding(Padding::new(2, 0, 0, 0)));
            frame.render_widget(info_para, inner_area);
        }
    }
}

impl ChatComponent {
    // ------------------------------------------------------------------
    // Phase 5: message actions
    // ------------------------------------------------------------------

    /// Returns the actions available for the message at the given index, with
    /// a short label for the action menu.
    #[must_use]
    pub fn available_actions(&self, idx: usize) -> Vec<(ChatAction, &'static str)> {
        let role = match self.messages.get(idx) {
            Some(m) => m.role.clone(),
            None => return Vec::new(),
        };
        match role {
            MessageRole::User => vec![
                (ChatAction::Copy, "Copy"),
                (ChatAction::YankToComposer, "Yank to composer"),
                (ChatAction::Edit, "Edit"),
                (ChatAction::Delete, "Delete"),
                (ChatAction::Branch, "Branch from here"),
            ],
            MessageRole::Assistant => vec![
                (ChatAction::Copy, "Copy"),
                (ChatAction::YankToComposer, "Yank to composer"),
                (ChatAction::Regenerate, "Regenerate"),
                (ChatAction::Delete, "Delete"),
                (ChatAction::Branch, "Branch from here"),
            ],
            MessageRole::System | MessageRole::Tool => vec![
                (ChatAction::Copy, "Copy"),
                (ChatAction::YankToComposer, "Yank to composer"),
                (ChatAction::Delete, "Delete"),
            ],
        }
    }

    /// Translate a `ChatAction` into a `ChatCommand` for the app loop.
    /// Returns `Noop` if the index is out of bounds.
    #[must_use]
    pub fn perform_action(&self, action: ChatAction, idx: usize) -> ChatCommand {
        let message = match self.messages.get(idx) {
            Some(m) => m,
            None => return ChatCommand::Noop,
        };
        match action {
            ChatAction::Copy => ChatCommand::Copy(message.content.clone()),
            ChatAction::YankToComposer => ChatCommand::Yank(message.content.clone()),
            ChatAction::Edit => match &message.message_id {
                Some(id) => ChatCommand::Edit {
                    message_id: id.clone(),
                    new_content: message.content.clone(),
                },
                None => ChatCommand::Noop,
            },
            ChatAction::Regenerate => ChatCommand::Regenerate { from_index: idx },
            ChatAction::Delete => ChatCommand::Delete { from_index: idx },
            ChatAction::Branch => match &message.message_id {
                Some(id) => ChatCommand::Branch {
                    from_message_id: id.clone(),
                    content: message.content.clone(),
                },
                None => ChatCommand::Noop,
            },
        }
    }

    /// Find the message index for a given `message_id`, or `None`.
    #[must_use]
    pub fn index_of_message_id(&self, id: &str) -> Option<usize> {
        self.messages
            .iter()
            .position(|m| m.message_id.as_deref() == Some(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::MessageRole;
    use crate::state::messages::Message;

    fn make_user(s: &str) -> Message {
        Message::user(s)
    }

    #[test]
    fn test_message_height_cjk() {
        let chat = ChatComponent::with_defaults();
        let m = make_user("你好世界");
        let h = chat.message_height(&m, &CardManager::new(ChatColorsRgb::default()), 40);
        // 4 CJK chars (8 cells) at width 38 → 1 wrapped line, plus 3 for borders.
        assert!(h >= 4);
    }

    #[test]
    fn test_height_cache_invalidates_on_add() {
        let mut chat = ChatComponent::with_defaults();
        let mut state = ChatState::new();
        let cards = CardManager::new(ChatColorsRgb::default());
        chat.add_message(make_user("hello"), &mut state, &cards);
        let v1 = chat.height_version;
        chat.add_message(make_user("world"), &mut state, &cards);
        let v2 = chat.height_version;
        assert!(v2 > v1);
    }

    #[test]
    fn test_height_cache_invalidates_on_update() {
        let mut chat = ChatComponent::with_defaults();
        let mut state = ChatState::new();
        let cards = CardManager::new(ChatColorsRgb::default());
        let mut msg = make_user("hello");
        msg.message_id = Some("abc".to_string());
        chat.add_message(msg, &mut state, &cards);
        let v1 = chat.height_version;
        let mut updated = make_user("hello world");
        updated.message_id = Some("abc".to_string());
        chat.update_message(updated);
        let v2 = chat.height_version;
        assert!(v2 > v1);
    }

    #[test]
    fn test_ensure_heights_caches() {
        let mut chat = ChatComponent::with_defaults();
        let mut state = ChatState::new();
        let cards = CardManager::new(ChatColorsRgb::default());
        chat.add_message(make_user("hello"), &mut state, &cards);
        chat.add_message(make_user("world"), &mut state, &cards);
        chat.ensure_heights(80, &cards);
        let first = chat.cached_heights().heights.clone();
        chat.ensure_heights(80, &cards);
        let second = chat.cached_heights().heights.clone();
        assert_eq!(first, second);
        // Widening the chat should invalidate the cache.
        chat.ensure_heights(120, &cards);
        let third = chat.cached_heights().heights.clone();
        assert_eq!(
            first, third,
            "width change with same content shouldn't change heights"
        );
    }

    #[test]
    fn test_mark_user_scrolled_clears_at_bottom() {
        let mut state = ChatState::new();
        state.at_bottom = true;
        let chat = ChatComponent::with_defaults();
        chat.mark_user_scrolled(&mut state);
        assert!(!state.at_bottom);
    }

    #[test]
    fn test_pending_new_increments_when_offscreen() {
        let mut chat = ChatComponent::with_defaults();
        let mut state = ChatState::new();
        let cards = CardManager::new(ChatColorsRgb::default());
        state.at_bottom = false;
        chat.add_message(make_user("first"), &mut state, &cards);
        assert_eq!(state.pending_new_count, 1);
        chat.add_message(make_user("second"), &mut state, &cards);
        assert_eq!(state.pending_new_count, 2);
    }

    #[test]
    fn test_jump_to_bottom_clears_pending() {
        let chat = ChatComponent::with_defaults();
        let mut state = ChatState::new();
        let cards = CardManager::new(ChatColorsRgb::default());
        state.pending_new_count = 5;
        state.at_bottom = false;
        chat.jump_to_bottom(&mut state, &cards);
        assert_eq!(state.pending_new_count, 0);
        assert!(state.at_bottom);
    }

    fn make_assistant(s: &str) -> Message {
        Message::assistant(s)
    }

    #[test]
    fn test_available_actions_user() {
        let mut chat = ChatComponent::with_defaults();
        let mut state = ChatState::new();
        let cards = CardManager::new(ChatColorsRgb::default());
        chat.add_message(make_user("hello"), &mut state, &cards);
        let actions = chat.available_actions(0);
        let kinds: Vec<ChatAction> = actions.iter().map(|(a, _)| *a).collect();
        assert!(kinds.contains(&ChatAction::Copy));
        assert!(kinds.contains(&ChatAction::YankToComposer));
        assert!(kinds.contains(&ChatAction::Edit));
        assert!(kinds.contains(&ChatAction::Delete));
        assert!(kinds.contains(&ChatAction::Branch));
    }

    #[test]
    fn test_available_actions_assistant() {
        let mut chat = ChatComponent::with_defaults();
        let mut state = ChatState::new();
        let cards = CardManager::new(ChatColorsRgb::default());
        chat.add_message(make_assistant("hi"), &mut state, &cards);
        let actions = chat.available_actions(0);
        let kinds: Vec<ChatAction> = actions.iter().map(|(a, _)| *a).collect();
        assert!(kinds.contains(&ChatAction::Regenerate));
        assert!(!kinds.contains(&ChatAction::Edit));
    }

    #[test]
    fn test_perform_action_yank_returns_content() {
        let mut chat = ChatComponent::with_defaults();
        let mut state = ChatState::new();
        let cards = CardManager::new(ChatColorsRgb::default());
        chat.add_message(make_user("hello world"), &mut state, &cards);
        match chat.perform_action(ChatAction::YankToComposer, 0) {
            ChatCommand::Yank(s) => assert_eq!(s, "hello world"),
            _ => panic!("expected Yank"),
        }
    }

    #[test]
    fn test_perform_action_regenerate_uses_index() {
        let mut chat = ChatComponent::with_defaults();
        let mut state = ChatState::new();
        let cards = CardManager::new(ChatColorsRgb::default());
        chat.add_message(make_user("hi"), &mut state, &cards);
        chat.add_message(make_assistant("hello"), &mut state, &cards);
        match chat.perform_action(ChatAction::Regenerate, 1) {
            ChatCommand::Regenerate { from_index } => assert_eq!(from_index, 1),
            _ => panic!("expected Regenerate"),
        }
    }

    #[test]
    fn test_perform_action_out_of_bounds_is_noop() {
        let chat = ChatComponent::with_defaults();
        assert_eq!(chat.perform_action(ChatAction::Copy, 99), ChatCommand::Noop);
    }

    #[test]
    fn test_subagent_lines_has_id_and_goal() {
        use crate::ui::subagent::SubagentInfo;
        let chat = ChatComponent::with_defaults();
        let mut info = SubagentInfo::new("a1", "find the bug", None);
        let lines = chat.build_subagent_lines(&info, 80);
        assert!(!lines.is_empty());
        let rendered: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(rendered.contains("a1"));
        assert!(rendered.contains("find the bug"));
    }

    #[test]
    fn test_subagent_lines_truncates_long_goal() {
        use crate::ui::subagent::SubagentInfo;
        let chat = ChatComponent::with_defaults();
        let long_goal = "x".repeat(500);
        let info = SubagentInfo::new("a1", &long_goal, None);
        let lines = chat.build_subagent_lines(&info, 40);
        let first: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        // The goal is truncated to <= 40 - 20 = 20 cells.
        assert!(first.chars().count() < long_goal.chars().count());
        assert!(first.contains("…"));
    }

    #[test]
    fn test_subagent_lines_summary_adds_second_line() {
        use crate::ui::subagent::SubagentInfo;
        let chat = ChatComponent::with_defaults();
        let mut info = SubagentInfo::new("a1", "short goal", None);
        info.mark_completed("a brief summary".to_string());
        let lines = chat.build_subagent_lines(&info, 80);
        assert_eq!(lines.len(), 2);
        let second: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(second.contains("→"));
    }
}
