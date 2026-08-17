//! Cards module - Tool/message card UI components
//!
//! This module provides card-based UI components for displaying tool results,
//! errors, warnings, and other special message types.

use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Padding, Paragraph, Wrap},
    Frame,
};

use crate::state::config::ChatColorsRgb;

/// Card type for different message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardType {
    /// Information card (blue)
    Info,
    /// Success card (green)
    Success,
    /// Warning card (yellow)
    Warning,
    /// Error card (red)
    Error,
    /// Tool result card
    Tool,
}

/// Tool execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStatus {
    /// Tool is currently running
    Running,
    /// Tool completed successfully
    Completed,
    /// Tool failed with an error
    Failed,
    /// Tool is pending execution
    Pending,
}

pub struct ToolCardData {
    /// Name of the tool
    pub tool_name: String,
    /// Unique call ID for tracking this tool invocation
    pub call_id: String,
    /// Current execution status
    pub status: ToolStatus,
    /// Execution duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Serialized input arguments
    pub arguments: Option<String>,
    /// Serialized output result
    pub result: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

impl ToolCardData {
    pub fn running(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            call_id: String::new(),
            status: ToolStatus::Running,
            duration_ms: None,
            arguments: None,
            result: None,
            error: None,
        }
    }

    /// Create tool card data with a specific `call_id`
    pub fn with_call_id(mut self, call_id: impl Into<String>) -> Self {
        self.call_id = call_id.into();
        self
    }

    /// Mark the tool as completed with result
    pub fn completed(&mut self, result: impl Into<String>) {
        self.status = ToolStatus::Completed;
        self.result = Some(result.into());
    }

    /// Mark the tool as failed with error
    pub fn failed(&mut self, error: impl Into<String>) {
        self.status = ToolStatus::Failed;
        self.error = Some(error.into());
    }

    /// Set tool arguments
    pub fn with_arguments(mut self, arguments: impl Into<String>) -> Self {
        self.arguments = Some(arguments.into());
        self
    }

    /// Set duration in milliseconds
    #[must_use]
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}
#[derive(Debug, Clone)]
pub struct CardComponent {
    /// Card type (determines styling)
    card_type: CardType,
    /// Card title
    title: String,
    /// Card content (legacy/default summary)
    content: String,
    /// Unique call ID for tool tracking
    call_id: String,
    /// Raw tool name (without status icon decoration)
    tool_name: String,
    /// Whether the card is expanded (shows full content)
    expanded: bool,
    /// Animation frame counter for spinner
    spinner_frame: u32,
    /// Current execution status (for tool cards)
    status: Option<ToolStatus>,
    /// Colors from configuration
    colors: ChatColorsRgb,
    /// Tool arguments
    arguments: Option<String>,
    /// Tool result
    result: Option<String>,
    /// Tool error
    error: Option<String>,
}

impl CardComponent {
    /// Create a new card with the given type, title, and content
    pub fn new(
        card_type: CardType,
        title: impl Into<String>,
        content: impl Into<String>,
        colors: ChatColorsRgb,
    ) -> Self {
        Self {
            card_type,
            title: title.into(),
            content: content.into(),
            call_id: String::new(),
            tool_name: String::new(),
            expanded: false,
            spinner_frame: 0,
            status: None,
            colors,
            arguments: None,
            result: None,
            error: None,
        }
    }

    /// Create a new info card
    pub fn info(
        title: impl Into<String>,
        content: impl Into<String>,
        colors: ChatColorsRgb,
    ) -> Self {
        Self::new(CardType::Info, title, content, colors)
    }

    /// Create a new success card
    pub fn success(
        title: impl Into<String>,
        content: impl Into<String>,
        colors: ChatColorsRgb,
    ) -> Self {
        Self::new(CardType::Success, title, content, colors)
    }

    /// Create a new warning card
    pub fn warning(
        title: impl Into<String>,
        content: impl Into<String>,
        colors: ChatColorsRgb,
    ) -> Self {
        Self::new(CardType::Warning, title, content, colors)
    }

    /// Create a new error card
    pub fn error(
        title: impl Into<String>,
        content: impl Into<String>,
        colors: ChatColorsRgb,
    ) -> Self {
        Self::new(CardType::Error, title, content, colors)
    }

    /// Create a new tool card
    pub fn tool(
        title: impl Into<String>,
        content: impl Into<String>,
        colors: ChatColorsRgb,
    ) -> Self {
        Self::new(CardType::Tool, title, content, colors)
    }

    /// Create a new tool card from `ToolCardData`
    #[must_use]
    pub fn tool_card(data: &ToolCardData, colors: ChatColorsRgb, spinner_frame: u32) -> Self {
        let title = Self::tool_card_title(data);
        let content = Self::tool_card_content(data, spinner_frame);
        Self {
            card_type: CardType::Tool,
            title,
            content,
            call_id: data.call_id.clone(),
            tool_name: data.tool_name.clone(),
            expanded: false,
            spinner_frame: 0,
            status: Some(data.status),
            colors,
            arguments: data.arguments.clone(),
            result: data.result.clone(),
            error: data.error.clone(),
        }
    }

    /// Update this card from `ToolCardData` (preserves expanded state)
    pub fn update_from_data(&mut self, data: &ToolCardData) {
        self.title = Self::tool_card_title(data);
        self.content = Self::tool_card_content(data, self.spinner_frame);
        self.tool_name = data.tool_name.clone();
        self.status = Some(data.status);
        self.arguments = data.arguments.clone();
        self.result = data.result.clone();
        self.error = data.error.clone();
    }

    /// Toggle the expanded state of the card
    pub fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }

    /// Set the spinner animation frame
    pub fn set_spinner_frame(&mut self, frame: u32) {
        self.spinner_frame = frame;
    }

    /// Get the `call_id`
    #[must_use]
    pub fn call_id(&self) -> &str {
        &self.call_id
    }

    /// Get the raw tool name (without status icon decoration)
    #[must_use]
    pub fn tool_name(&self) -> &str {
        &self.tool_name
    }

    /// Get tool arguments
    #[must_use]
    pub fn arguments(&self) -> Option<&String> {
        self.arguments.as_ref()
    }

    /// Check if the card is expanded
    #[must_use]
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Update tool card content for a running tool with throbber spinner
    #[must_use]
    pub fn running_spinner(frame: u32) -> String {
        let spinner_chars = [
            "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦",
            "⠧", "⠇", "⠏", "⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈",
        ];
        let idx = (frame as usize) % spinner_chars.len();

        let dots = match (frame / 5) % 4 {
            0 => ".  ",
            1 => ".. ",
            2 => "...",
            _ => "   ",
        };

        format!("{} Running{}", spinner_chars[idx], dots)
    }

    /// Get animated status icon for tool cards
    fn status_icon(status: ToolStatus, frame: u64) -> Span<'static> {
        match status {
            ToolStatus::Running => {
                let icons = [" ◴ ", " ◷ ", " ◶ ", " ◵ "];
                let idx = (frame as usize / 2) % icons.len();
                Span::styled(icons[idx], Style::default().fg(Color::Yellow).bold())
            }
            ToolStatus::Completed => Span::styled(" ✓ ", Style::default().fg(Color::Green).bold()),
            ToolStatus::Failed => Span::styled(" ✗ ", Style::default().fg(Color::Red).bold()),
            ToolStatus::Pending => {
                let icons = [" ○ ", " ◌ ", " ◍ ", " ◌ "];
                let idx = (frame as usize / 8) % icons.len();
                Span::styled(icons[idx], Style::default().fg(Color::DarkGray))
            }
        }
    }
    /// Get a tool-specific icon
    fn tool_type_icon(tool_name: &str) -> &'static str {
        let name = tool_name.to_lowercase();
        if name.contains("search") || name.contains("grep") || name.contains("find") {
            "🔍"
        } else if name.contains("read") || name.contains("cat") {
            "📖"
        } else if name.contains("write") || name.contains("edit") || name.contains("replace") {
            "📝"
        } else if name.contains("shell") || name.contains("run") || name.contains("exec") {
            "⚡"
        } else if name.contains("git") {
            "🌿"
        } else if name.contains("test") {
            "🧪"
        } else if name.contains("list") || name.contains("ls") {
            "📁"
        } else {
            "🛠️"
        }
    }

    /// Build the title for a tool card
    fn tool_card_title(data: &ToolCardData) -> String {
        let duration = data
            .duration_ms
            .map_or(String::new(), |ms| format!(" ({:.1}s)", ms as f64 / 1000.0));
        format!("{}{}", data.tool_name, duration)
    }

    /// Build the content for a tool card
    /// When expanded, shows full content; when collapsed, shows summary only
    fn tool_card_content(data: &ToolCardData, spinner_frame: u32) -> String {
        let mut parts = Vec::new();

        if let Some(args) = &data.arguments {
            parts.push(format!("Args: {args}"));
        }

        // For collapsed view, show only a single-line summary
        let result_text = match data.status {
            ToolStatus::Running => {
                // Use the running_spinner with throttled frame
                let throttled = spinner_frame / 4;
                Self::running_spinner(throttled)
            }
            ToolStatus::Completed => {
                if let Some(result) = &data.result {
                    if result.len() > 120 {
                        format!("Result: {}...", &result[..117])
                    } else {
                        format!("Result: {result}")
                    }
                } else {
                    "Completed".to_string()
                }
            }
            ToolStatus::Failed => {
                if let Some(error) = &data.error {
                    format!("Error: {error}")
                } else {
                    "Failed (no error message)".to_string()
                }
            }
            ToolStatus::Pending => "Waiting...".to_string(),
        };
        parts.push(result_text);

        parts.join("\n")
    }
    /// Get the card type
    #[must_use]
    pub fn card_type(&self) -> CardType {
        self.card_type
    }

    /// Set the card type
    pub fn set_card_type(&mut self, card_type: CardType) {
        self.card_type = card_type;
    }

    /// Get the title
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set the title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Get the content
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Set the content
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    /// Set the colors
    pub fn set_colors(&mut self, colors: ChatColorsRgb) {
        self.colors = colors;
    }

    /// Get the appropriate background color for the card type
    fn bg_color(&self) -> ratatui::style::Color {
        match self.card_type {
            CardType::Info => self.colors.code_bg,
            CardType::Success => self.colors.code_bg,
            CardType::Warning => self.colors.code_bg,
            CardType::Error => self.colors.code_bg,
            CardType::Tool => self.colors.tool_bg,
        }
    }

    /// Get the appropriate text color for the card type
    fn text_color(&self) -> ratatui::style::Color {
        match self.card_type {
            CardType::Info => self.colors.assistant_text,
            CardType::Success => ratatui::style::Color::Green,
            CardType::Warning => ratatui::style::Color::Yellow,
            CardType::Error => ratatui::style::Color::Red,
            CardType::Tool => {
                let name = self.tool_name.to_lowercase();
                if name.contains("shell") || name.contains("run") {
                    Color::Rgb(255, 95, 0) // Orange for execution
                } else if name.contains("write") || name.contains("edit") {
                    Color::Rgb(0, 255, 135) // Spring green for edits
                } else if name.contains("search") || name.contains("grep") {
                    Color::Rgb(0, 175, 255) // Deep sky blue for search
                } else {
                    self.colors.tool_text
                }
            }
        }
    }

    /// Render a small inline sparkline for activity
    fn render_mini_sparkline(frame: u64, width: usize) -> String {
        let chars = [" ", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
        let mut s = String::with_capacity(width);
        for i in 0..width {
            let t = (frame as f64 * 0.2) + (i as f64 * 0.5);
            let val = (t.sin() * 0.5 + 0.5) * 7.0;
            s.push_str(chars[val.round() as usize]);
        }
        s
    }

    /// Render the card
    pub fn render(&self, frame: &mut Frame, area: Rect, animation_frame: u64) {
        let is_tool = self.card_type == CardType::Tool;
        let is_running_tool = is_tool && self.status == Some(ToolStatus::Running);

        // Animated background color for running tools
        let bg_color = if is_running_tool {
            let pulse = (animation_frame as f64 * 0.1).sin() * 0.05 + 0.95;
            match self.bg_color() {
                Color::Rgb(r, g, b) => {
                    let r = (r as f64 * pulse).clamp(0.0, 255.0) as u8;
                    let g = (g as f64 * pulse).clamp(0.0, 255.0) as u8;
                    let b = (b as f64 * pulse).clamp(0.0, 255.0) as u8;
                    Color::Rgb(r, g, b)
                }
                _ => self.bg_color(),
            }
        } else {
            self.bg_color()
        };

        // Render the gradient border (animated if running)
        crate::ui::borders::render_gradient_border(
            frame.buffer_mut(),
            area,
            animation_frame,
            is_running_tool,
            is_running_tool,
        );

        // Build title with animated elements
        let mut title_spans = Vec::new();
        if is_tool {
            // Dropdown/Expand indicator
            let expand_icon = if self.expanded { " ▼ " } else { " ▶ " };
            title_spans.push(Span::styled(expand_icon, Style::default().fg(Color::Gray)));

            // Status icon (animated)
            if let Some(status) = self.status {
                title_spans.push(Self::status_icon(status, animation_frame));
            }

            // Tool type icon
            title_spans.push(Span::raw(format!(
                " {} ",
                Self::tool_type_icon(&self.tool_name)
            )));

            // Title text with shimmer effect if running
            let text_color = self.text_color();
            let title_style = if is_running_tool {
                let shimmer = ((animation_frame as f64 * 0.2).sin() * 0.5 + 0.5) > 0.7;
                if shimmer {
                    Style::default().fg(Color::White).bold()
                } else {
                    Style::default().fg(text_color).bold()
                }
            } else {
                Style::default().fg(text_color).bold()
            };
            title_spans.push(Span::styled(&self.title, title_style));

            // Add activity sparkline if running
            if is_running_tool {
                title_spans.push(Span::raw("  "));
                title_spans.push(Span::styled(
                    Self::render_mini_sparkline(animation_frame, 8),
                    Style::default().fg(Color::Rgb(0, 255, 255)),
                ));
            }
        } else {
            title_spans.push(Span::styled(
                format!(" {} ", self.title),
                Style::default().bold(),
            ));
        }

        // Create a block for the title and background
        let block = Block::default().title(Line::from(title_spans)).bg(bg_color);

        // Inner area (accounting for the border we just drew)
        let inner_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        // Render the background/title block
        frame.render_widget(block, area);

        if inner_area.height == 0 || inner_area.width == 0 {
            return;
        }

        let mut lines: Vec<Line> = Vec::new();

        if is_tool {
            // Always show detailed breakdown
            if let Some(args) = &self.arguments {
                let arg_style = if is_running_tool {
                    let pulse = (animation_frame as f64 * 0.15).cos() * 0.5 + 0.5;
                    if pulse > 0.8 {
                        Color::White
                    } else {
                        Color::Gray
                    }
                } else {
                    Color::Gray
                };
                lines.push(Line::from(vec![
                    Span::styled(" Args: ", Style::default().fg(arg_style).italic()),
                    Span::styled(args, Style::default().fg(self.colors.code_text)),
                ]));
            }

            if let Some(res) = &self.result {
                lines.push(Line::from(vec![Span::styled(
                    " Result: ",
                    Style::default().fg(Color::Gray).italic(),
                )]));

                // Parse ANSI codes in result
                let text = crate::utils::ansi::ansi_to_text(res);
                for line in text.lines {
                    let mut spans = vec![Span::raw("   ")];
                    spans.extend(line.spans);
                    lines.push(Line::from(spans));
                }
            } else if let Some(err) = &self.error {
                lines.push(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(" Error: ", Style::default().fg(Color::Red).bold()),
                    Span::styled(err, Style::default().fg(Color::Red)),
                ]));
            } else if self.status == Some(ToolStatus::Running) {
                // Large activity indicator
                let loader = match (animation_frame / 4) % 4 {
                    0 => "▖ ",
                    1 => "▘ ",
                    2 => "▝ ",
                    _ => "▗ ",
                };
                lines.push(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(loader, Style::default().fg(Color::Rgb(0, 255, 255)).bold()),
                    Span::styled(
                        Self::running_spinner(animation_frame as u32),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        Self::render_mini_sparkline(animation_frame, 20),
                        Style::default().fg(Color::Rgb(0, 150, 255)),
                    ),
                ]));
            }
        } else {
            // Standard Card View
            for line in self.content.lines() {
                lines.push(Line::from(Span::styled(
                    line,
                    Style::new().fg(self.text_color()),
                )));
            }
        }

        // Create paragraph with content and wrapping
        let paragraph = Paragraph::new(Text::from(lines))
            .wrap(Wrap { trim: false })
            .style(Style::new().bg(bg_color))
            .block(Block::new().padding(Padding::new(1, 1, 0, 1)));

        frame.render_widget(paragraph, inner_area);
    }
}

/// Card manager for handling multiple visible cards
#[derive(Debug, Clone)]
pub struct CardManager {
    /// List of active cards
    cards: Vec<CardComponent>,
    /// Colors from configuration
    colors: ChatColorsRgb,
}

impl CardManager {
    /// Create a new card manager
    #[must_use]
    pub fn new(colors: ChatColorsRgb) -> Self {
        Self {
            cards: Vec::new(),
            colors,
        }
    }

    /// Create a new card manager with defaults
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(ChatColorsRgb::default())
    }

    /// Add a card
    pub fn add_card(&mut self, card: CardComponent) {
        self.cards.push(card);
    }

    /// Remove a card by index
    pub fn remove_card(&mut self, index: usize) -> Option<CardComponent> {
        if index < self.cards.len() {
            Some(self.cards.remove(index))
        } else {
            None
        }
    }

    /// Clear all cards
    pub fn clear(&mut self) {
        self.cards.clear();
    }

    /// Get the number of cards
    #[must_use]
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Check if there are any cards
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Get all cards
    #[must_use]
    pub fn cards(&self) -> &[CardComponent] {
        &self.cards
    }

    /// Add an info card
    pub fn add_info(&mut self, title: impl Into<String>, content: impl Into<String>) {
        self.add_card(CardComponent::info(title, content, self.colors.clone()));
    }

    /// Add a success card
    pub fn add_success(&mut self, title: impl Into<String>, content: impl Into<String>) {
        self.add_card(CardComponent::success(title, content, self.colors.clone()));
    }

    /// Add a warning card
    pub fn add_warning(&mut self, title: impl Into<String>, content: impl Into<String>) {
        self.add_card(CardComponent::warning(title, content, self.colors.clone()));
    }

    /// Add an error card
    pub fn add_error(&mut self, title: impl Into<String>, content: impl Into<String>) {
        self.add_card(CardComponent::error(title, content, self.colors.clone()));
    }

    /// Add a tool card
    pub fn add_tool(&mut self, title: impl Into<String>, content: impl Into<String>) {
        self.add_card(CardComponent::tool(title, content, self.colors.clone()));
    }

    /// Add a tool card from `ToolCardData`
    pub fn add_tool_card(&mut self, data: ToolCardData) {
        // Use 0 as initial frame; the tick_spinners loop will advance it
        self.add_card(CardComponent::tool_card(&data, self.colors.clone(), 0));
    }

    /// Update the status of an existing tool card by `call_id`
    pub fn update_tool_status(
        &mut self,
        call_id: &str,
        status: ToolStatus,
        result: Option<String>,
        error: Option<String>,
    ) -> bool {
        if let Some(card) = self.cards.iter_mut().find(|c| c.call_id() == call_id) {
            let data = ToolCardData {
                tool_name: card.tool_name().to_string(), // use raw name, not decorated title
                call_id: call_id.to_string(),
                status,
                duration_ms: None,
                arguments: card.arguments().cloned(), // Preserve arguments
                result,
                error,
            };
            card.update_from_data(&data);
            true
        } else {
            false
        }
    }

    /// Find a card by `call_id`
    #[must_use]
    pub fn find_by_call_id(&self, call_id: &str) -> Option<&CardComponent> {
        self.cards.iter().find(|c| c.call_id() == call_id)
    }

    /// Find a card by `call_id` (mutable)
    pub fn find_by_call_id_mut(&mut self, call_id: &str) -> Option<&mut CardComponent> {
        self.cards.iter_mut().find(|c| c.call_id() == call_id)
    }

    /// Update a tool card by `call_id` with new data
    pub fn update_by_call_id(&mut self, call_id: &str, data: &ToolCardData) -> bool {
        if let Some(card) = self.find_by_call_id_mut(call_id) {
            card.update_from_data(data);
            true
        } else {
            false
        }
    }

    /// Toggle expansion of a tool card by `call_id`
    pub fn toggle_expanded(&mut self, call_id: &str) -> bool {
        if let Some(card) = self.find_by_call_id_mut(call_id) {
            card.toggle_expanded();
            true
        } else {
            false
        }
    }

    /// Advance spinner frames for all tool cards
    pub fn tick_spinners(&mut self) {
        for card in &mut self.cards {
            if card.card_type() == CardType::Tool {
                card.spinner_frame = card.spinner_frame.wrapping_add(1);
            }
        }
    }

    /// Set the colors for all future cards
    pub fn set_colors(&mut self, colors: ChatColorsRgb) {
        self.colors = colors;
    }

    /// Update colors for all existing cards
    pub fn update_colors(&mut self, colors: ChatColorsRgb) {
        for card in &mut self.cards {
            card.set_colors(colors.clone());
        }
        self.colors = colors;
    }

    /// Render all cards in a stack (vertical layout)
    pub fn render_stack(&self, frame: &mut Frame, area: Rect, animation_frame: u64) {
        if self.cards.is_empty() {
            return;
        }

        // Calculate card heights (each card gets equal height)
        let card_height = area.height / self.cards.len() as u16;

        for (i, card) in self.cards.iter().enumerate() {
            let card_area = Rect {
                x: area.x,
                y: area.y + (i as u16 * card_height),
                width: area.width,
                height: card_height,
            };
            card.render(frame, card_area, animation_frame);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_colors() -> ChatColorsRgb {
        ChatColorsRgb::default()
    }

    #[test]
    fn test_card_new() {
        let colors = create_test_colors();
        let card = CardComponent::new(CardType::Info, "Test", "Content", colors);
        assert_eq!(card.card_type(), CardType::Info);
        assert_eq!(card.title(), "Test");
        assert_eq!(card.content(), "Content");
    }

    #[test]
    fn test_card_info() {
        let colors = create_test_colors();
        let card = CardComponent::info("Info", "Info content", colors);
        assert_eq!(card.card_type(), CardType::Info);
    }

    #[test]
    fn test_card_success() {
        let colors = create_test_colors();
        let card = CardComponent::success("Success", "Success content", colors);
        assert_eq!(card.card_type(), CardType::Success);
    }

    #[test]
    fn test_card_warning() {
        let colors = create_test_colors();
        let card = CardComponent::warning("Warning", "Warning content", colors);
        assert_eq!(card.card_type(), CardType::Warning);
    }

    #[test]
    fn test_card_error() {
        let colors = create_test_colors();
        let card = CardComponent::error("Error", "Error content", colors);
        assert_eq!(card.card_type(), CardType::Error);
    }

    #[test]
    fn test_card_tool() {
        let colors = create_test_colors();
        let card = CardComponent::tool("Tool", "Tool content", colors);
        assert_eq!(card.card_type(), CardType::Tool);
    }

    #[test]
    fn test_card_setters() {
        let colors = create_test_colors();
        let mut card = CardComponent::new(CardType::Info, "Test", "Content", colors);

        card.set_card_type(CardType::Success);
        assert_eq!(card.card_type(), CardType::Success);

        card.set_title("New Title");
        assert_eq!(card.title(), "New Title");

        card.set_content("New Content");
        assert_eq!(card.content(), "New Content");
    }

    #[test]
    fn test_card_manager_new() {
        let colors = create_test_colors();
        let manager = CardManager::new(colors);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_card_manager_add() {
        let colors = create_test_colors();
        let mut manager = CardManager::new(colors);

        manager.add_info("Info", "Content");
        assert_eq!(manager.len(), 1);

        manager.add_success("Success", "Content");
        assert_eq!(manager.len(), 2);
    }

    #[test]
    fn test_card_manager_remove() {
        let colors = create_test_colors();
        let mut manager = CardManager::new(colors);

        manager.add_info("Info", "Content");
        manager.add_success("Success", "Content");

        let removed = manager.remove_card(0);
        assert!(removed.is_some());
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_card_manager_clear() {
        let colors = create_test_colors();
        let mut manager = CardManager::new(colors);

        manager.add_info("Info", "Content");
        manager.add_success("Success", "Content");

        assert_eq!(manager.len(), 2);
        manager.clear();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_card_manager_with_defaults() {
        let manager = CardManager::with_defaults();
        assert!(manager.is_empty());
    }
}
