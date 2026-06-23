//! Mouse module - Mouse event handlers
//!
//! This module provides handlers for mouse events including
//! clicks, drags, scrolls, and hover detection.

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::handlers::input::ParsedMouseEvent;

/// Mouse interaction context for tracking state
#[derive(Debug, Clone, Default)]
pub struct MouseContext {
    pub position: (u16, u16),
    pub pressed_buttons: Vec<MouseButton>,
    pub is_dragging: bool,
    pub drag_start: Option<(u16, u16)>,
    pub last_click: Option<(u16, u16)>,
    pub last_click_time: Option<std::time::Instant>,
}

impl MouseContext {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, event: &MouseEvent) {
        let parsed = ParsedMouseEvent::from(event);
        self.position = (parsed.column, parsed.row);

        match event.kind {
            MouseEventKind::Down(button) => {
                self.pressed_buttons.push(button);
                self.last_click = Some((parsed.column, parsed.row));
                self.last_click_time = Some(std::time::Instant::now());
                self.drag_start = Some((parsed.column, parsed.row));
            }
            MouseEventKind::Up(button) => {
                self.pressed_buttons.retain(|&b| b != button);
                self.is_dragging = false;
            }
            MouseEventKind::Drag(button) => {
                if !self.pressed_buttons.contains(&button) {
                    self.pressed_buttons.push(button);
                }
                self.is_dragging = true;
            }
            _ => {}
        }
    }

    #[must_use]
    pub fn position(&self) -> (u16, u16) {
        self.position
    }

    #[must_use]
    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    #[must_use]
    pub fn drag_start(&self) -> Option<(u16, u16)> {
        self.drag_start
    }

    #[must_use]
    pub fn check_double_click(&self, event: &MouseEvent) -> bool {
        if let MouseEventKind::Down(button) = event.kind {
            if button == MouseButton::Left {
                if let (Some(last_pos), Some(last_time)) = (self.last_click, self.last_click_time) {
                    let parsed = ParsedMouseEvent::from(event);
                    let is_same_position = last_pos == (parsed.column, parsed.row);
                    let is_fast = last_time.elapsed() < std::time::Duration::from_millis(300);
                    return is_same_position && is_fast;
                }
            }
        }
        false
    }

    pub fn clear(&mut self) {
        self.pressed_buttons.clear();
        self.is_dragging = false;
        self.drag_start = None;
    }
}

/// Handles mouse events within the chat area
#[derive(Debug, Default)]
pub struct ChatMouseHandler {
    click_padding: u16,
}

impl ChatMouseHandler {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn with_padding(padding: u16) -> Self {
        Self {
            click_padding: padding,
        }
    }

    #[must_use]
    pub fn handle_chat_mouse(
        &self,
        event: &MouseEvent,
        chat_area: (u16, u16, u16, u16),
        _message_count: usize,
        visible_start: usize,
        visible_count: usize,
    ) -> ChatMouseAction {
        let parsed = ParsedMouseEvent::from(event);
        let (x, y, width, height) = chat_area;

        if !self.is_in_area(parsed.column, parsed.row, x, y, width, height) {
            return ChatMouseAction::None;
        }

        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let relative_y = parsed.row.saturating_sub(y);
                if relative_y < height {
                    let message_index = visible_start + relative_y as usize;
                    if message_index < visible_start + visible_count {
                        return ChatMouseAction::MessageClick(message_index);
                    }
                }
                ChatMouseAction::ChatClick(parsed.column, parsed.row)
            }
            MouseEventKind::Down(MouseButton::Right) => {
                ChatMouseAction::ContextMenu(parsed.column, parsed.row)
            }
            MouseEventKind::ScrollUp => ChatMouseAction::ScrollUp,
            MouseEventKind::ScrollDown => ChatMouseAction::ScrollDown,
            _ => ChatMouseAction::None,
        }
    }

    fn is_in_area(&self, col: u16, row: u16, x: u16, y: u16, width: u16, height: u16) -> bool {
        col >= x && col < x + width && row >= y && row < y + height
    }

    #[must_use]
    pub fn click_padding(&self) -> u16 {
        self.click_padding
    }
}

/// Handles mouse events within the input area
#[derive(Debug, Default)]
pub struct InputMouseHandler;

impl InputMouseHandler {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn handle_input_mouse(
        &self,
        event: &MouseEvent,
        input_area: (u16, u16, u16, u16),
        _text_length: usize,
    ) -> (InputMouseAction, Option<usize>) {
        let parsed = ParsedMouseEvent::from(event);
        let (x, _y, width, _height) = input_area;

        if parsed.column < x || parsed.column >= x + width {
            return (InputMouseAction::None, None);
        }

        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let cursor_pos = (parsed.column - x) as usize;
                (InputMouseAction::SetCursor(cursor_pos), Some(cursor_pos))
            }
            MouseEventKind::Down(MouseButton::Right) => {
                let cursor_pos = (parsed.column - x) as usize;
                (InputMouseAction::ContextMenuAt(cursor_pos), Some(cursor_pos))
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                let cursor_pos = (parsed.column - x) as usize;
                (InputMouseAction::DragCursor(cursor_pos), Some(cursor_pos))
            }
            _ => (InputMouseAction::None, None),
        }
    }
}

/// Toolbar button configuration
#[derive(Debug, Clone)]
pub struct ToolbarButton {
    pub id: String,
    pub label: String,
    pub area: Option<(u16, u16, u16, u16)>,
}

impl ToolbarButton {
    #[must_use]
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            area: None,
        }
    }

    pub fn set_area(&mut self, x: u16, y: u16, width: u16, height: u16) {
        self.area = Some((x, y, width, height));
    }
}

/// Handles mouse events within the toolbar area
#[derive(Debug, Default)]
pub struct ToolbarMouseHandler {
    buttons: Vec<ToolbarButton>,
}

impl ToolbarMouseHandler {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_buttons(&mut self, buttons: Vec<ToolbarButton>) {
        self.buttons = buttons;
    }

    pub fn add_button(&mut self, button: ToolbarButton) {
        self.buttons.push(button);
    }

    #[must_use]
    pub fn handle_toolbar_mouse(&self, event: &MouseEvent) -> ToolbarMouseAction {
        let parsed = ParsedMouseEvent::from(event);

        for button in &self.buttons {
            if let Some((x, y, width, height)) = button.area {
                if parsed.column >= x
                    && parsed.column < x + width
                    && parsed.row >= y
                    && parsed.row < y + height
                {
                    return match event.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            ToolbarMouseAction::ButtonPress(button.id.clone())
                        }
                        MouseEventKind::Up(MouseButton::Left) => {
                            ToolbarMouseAction::ButtonRelease(button.id.clone())
                        }
                        _ => ToolbarMouseAction::None,
                    };
                }
            }
        }
        ToolbarMouseAction::None
    }

    #[must_use]
    pub fn buttons(&self) -> &[ToolbarButton] {
        &self.buttons
    }
}

/// Chat mouse actions
#[derive(Debug, Clone, PartialEq)]
pub enum ChatMouseAction {
    MessageClick(usize),
    ChatClick(u16, u16),
    ScrollUp,
    ScrollDown,
    ContextMenu(u16, u16),
    None,
}

/// Input mouse actions
#[derive(Debug, Clone, PartialEq)]
pub enum InputMouseAction {
    SetCursor(usize),
    DragCursor(usize),
    ContextMenuAt(usize),
    None,
}

/// Toolbar mouse actions
#[derive(Debug, Clone, PartialEq)]
pub enum ToolbarMouseAction {
    ButtonPress(String),
    ButtonRelease(String),
    None,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_mouse_context_update() {
        let mut context = MouseContext::new();
        let down_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        context.update(&down_event);
        assert_eq!(context.position(), (10, 5));
        assert!(context.is_button_pressed(MouseButton::Left));
        assert_eq!(context.drag_start(), Some((10, 5)));
    }

    #[test]
    fn test_mouse_context_double_click() {
        let mut context = MouseContext::new();
        let first_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        context.update(&first_click);
        std::thread::sleep(std::time::Duration::from_millis(100));
        let second_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        assert!(context.check_double_click(&second_click));
    }

    #[test]
    fn test_chat_mouse_handler() {
        let handler = ChatMouseHandler::new();
        let click_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 5,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        let scroll_up = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 5,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        let action = handler.handle_chat_mouse(&click_event, (0, 0, 100, 100), 10, 0, 10);
        match action {
            ChatMouseAction::MessageClick(_) => {}
            _ => panic!("Expected MessageClick"),
        }
        assert_eq!(
            handler.handle_chat_mouse(&scroll_up, (0, 0, 100, 100), 10, 0, 10),
            ChatMouseAction::ScrollUp
        );
    }

    #[test]
    fn test_input_mouse_handler() {
        let handler = InputMouseHandler::new();
        let click_event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 5,
            row: 0,
            modifiers: KeyModifiers::NONE,
        };
        let (action, pos) = handler.handle_input_mouse(&click_event, (0, 0, 100, 1), 50);
        match action {
            InputMouseAction::SetCursor(p) => {
                assert_eq!(p, 5);
                assert_eq!(pos, Some(5));
            }
            _ => panic!("Expected SetCursor"),
        }
    }
}
