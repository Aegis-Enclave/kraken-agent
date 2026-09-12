//! Input module - Input event handlers
//!
//! This module provides handlers for processing user input events
//! including keyboard, mouse, and paste events.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};

/// Input event enum that wraps crossterm events
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// Key press event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Paste event with text content
    Paste(String),
    /// Resize event
    Resize(u16, u16),
    /// Focus gained
    FocusGained,
    /// Focus lost
    FocusLost,
}

/// Key binding configuration for the TUI
#[derive(Debug, Clone)]
pub struct KeyBindings {
    pub quit: Vec<KeyEvent>,
    pub submit: Vec<KeyEvent>,
    pub command_mode: Vec<KeyEvent>,
    pub exit_command_mode: Vec<KeyEvent>,
    pub history_up: Vec<KeyEvent>,
    pub history_down: Vec<KeyEvent>,
    pub new_session: Vec<KeyEvent>,
    pub resume_session: Vec<KeyEvent>,
    pub list_sessions: Vec<KeyEvent>,
    pub scroll_up: Vec<KeyEvent>,
    pub scroll_down: Vec<KeyEvent>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: vec![
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
                KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            ],
            submit: vec![KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)],
            command_mode: vec![KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)],
            exit_command_mode: vec![KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)],
            history_up: vec![KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)],
            history_down: vec![KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)],
            new_session: vec![KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL)],
            resume_session: vec![KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL)],
            list_sessions: vec![KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL)],
            scroll_up: vec![
                KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
                KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE),
            ],
            scroll_down: vec![
                KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
                KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
            ],
        }
    }
}

impl KeyBindings {
    #[must_use]
    pub fn matches_any(&self, key: &KeyEvent, bindings: &[KeyEvent]) -> bool {
        bindings
            .iter()
            .any(|binding| self.key_matches(key, binding))
    }

    #[must_use]
    pub fn key_matches(&self, a: &KeyEvent, b: &KeyEvent) -> bool {
        a.code == b.code && a.modifiers == b.modifiers
    }

    #[must_use]
    pub fn is_quit(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.quit)
    }
    #[must_use]
    pub fn is_submit(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.submit)
    }
    #[must_use]
    pub fn is_command_mode(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.command_mode)
    }
    #[must_use]
    pub fn is_exit_command_mode(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.exit_command_mode)
    }
    #[must_use]
    pub fn is_history_up(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.history_up)
    }
    #[must_use]
    pub fn is_history_down(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.history_down)
    }
    #[must_use]
    pub fn is_new_session(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.new_session)
    }
    #[must_use]
    pub fn is_resume_session(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.resume_session)
    }
    #[must_use]
    pub fn is_list_sessions(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.list_sessions)
    }
    #[must_use]
    pub fn is_scroll_up(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.scroll_up)
    }
    #[must_use]
    pub fn is_scroll_down(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.scroll_down)
    }
}

/// Mouse button enum without crossterm dependency conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Unknown,
}

impl From<crossterm::event::MouseButton> for MouseButton {
    fn from(button: crossterm::event::MouseButton) -> Self {
        match button {
            crossterm::event::MouseButton::Left => Self::Left,
            crossterm::event::MouseButton::Right => Self::Right,
            crossterm::event::MouseButton::Middle => Self::Middle,
        }
    }
}

/// Our own mouse event kind (different name from crossterm to avoid conflict)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseKind {
    Down,
    Up,
    Drag,
    Moved,
    Scroll,
    Unknown,
}

impl From<crossterm::event::MouseEventKind> for MouseKind {
    fn from(kind: crossterm::event::MouseEventKind) -> Self {
        match kind {
            crossterm::event::MouseEventKind::Down(_) => Self::Down,
            crossterm::event::MouseEventKind::Up(_) => Self::Up,
            crossterm::event::MouseEventKind::Drag(_) => Self::Drag,
            crossterm::event::MouseEventKind::Moved => Self::Moved,
            crossterm::event::MouseEventKind::ScrollDown
            | crossterm::event::MouseEventKind::ScrollUp => Self::Scroll,
            _ => Self::Unknown,
        }
    }
}

/// Parsed mouse event with easier-to-use fields
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParsedMouseEvent {
    pub column: u16,
    pub row: u16,
    pub kind: MouseKind,
    pub modifiers: KeyModifiers,
}

impl From<&MouseEvent> for ParsedMouseEvent {
    fn from(event: &MouseEvent) -> Self {
        Self {
            column: event.column,
            row: event.row,
            kind: MouseKind::from(event.kind),
            modifiers: event.modifiers,
        }
    }
}

/// Input processor for handling raw input events
#[derive(Debug, Default)]
pub struct InputProcessor {
    key_bindings: KeyBindings,
}

/// Action to take based on input event
#[derive(Debug, Clone, PartialEq)]
pub enum InputAction {
    Quit,
    Submit,
    EnterCommandMode,
    ExitCommandMode,
    HistoryUp,
    HistoryDown,
    NewSession,
    ResumeSession,
    ListSessions,
    ScrollUp,
    ScrollDown,
    PassThrough(KeyEvent),
    None,
}

/// Action to take based on mouse event
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseAction {
    ScrollUp(u16, u16),
    ScrollDown(u16, u16),
    LeftClick(u16, u16),
    RightClick(u16, u16),
    PassThrough,
    None,
}

impl InputProcessor {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn with_bindings(bindings: KeyBindings) -> Self {
        Self {
            key_bindings: bindings,
        }
    }

    #[must_use]
    pub fn process_key(&self, key: KeyEvent) -> InputAction {
        if self.key_bindings.is_quit(&key) {
            return InputAction::Quit;
        }
        if self.key_bindings.is_submit(&key) {
            return InputAction::Submit;
        }
        if self.key_bindings.is_command_mode(&key) {
            return InputAction::EnterCommandMode;
        }
        if self.key_bindings.is_exit_command_mode(&key) {
            return InputAction::ExitCommandMode;
        }
        if self.key_bindings.is_history_up(&key) {
            return InputAction::HistoryUp;
        }
        if self.key_bindings.is_history_down(&key) {
            return InputAction::HistoryDown;
        }
        if self.key_bindings.is_new_session(&key) {
            return InputAction::NewSession;
        }
        if self.key_bindings.is_resume_session(&key) {
            return InputAction::ResumeSession;
        }
        if self.key_bindings.is_list_sessions(&key) {
            return InputAction::ListSessions;
        }
        if self.key_bindings.is_scroll_up(&key) {
            return InputAction::ScrollUp;
        }
        if self.key_bindings.is_scroll_down(&key) {
            return InputAction::ScrollDown;
        }
        InputAction::PassThrough(key)
    }

    #[must_use]
    pub fn process_mouse(&self, mouse: &MouseEvent) -> MouseAction {
        use crossterm::event::{MouseButton, MouseEventKind};
        let parsed = ParsedMouseEvent::from(mouse);
        match mouse.kind {
            MouseEventKind::ScrollUp => MouseAction::ScrollUp(parsed.column, parsed.row),
            MouseEventKind::ScrollDown => MouseAction::ScrollDown(parsed.column, parsed.row),
            MouseEventKind::Down(button) => match button {
                MouseButton::Left => MouseAction::LeftClick(parsed.column, parsed.row),
                MouseButton::Right => MouseAction::RightClick(parsed.column, parsed.row),
                _ => MouseAction::PassThrough,
            },
            _ => MouseAction::PassThrough,
        }
    }

    #[must_use]
    pub fn key_bindings(&self) -> &KeyBindings {
        &self.key_bindings
    }
    pub fn key_bindings_mut(&mut self) -> &mut KeyBindings {
        &mut self.key_bindings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_key_bindings() {
        let bindings = KeyBindings::default();
        assert!(!bindings.quit.is_empty());
        assert!(!bindings.submit.is_empty());
        assert!(!bindings.command_mode.is_empty());
    }

    #[test]
    fn test_quit_key_matching() {
        let bindings = KeyBindings::default();
        assert!(bindings.is_quit(&KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));
        assert!(bindings.is_quit(&KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL)));
        assert!(bindings.is_quit(&KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
    }

    #[test]
    fn test_input_processor() {
        let processor = InputProcessor::new();
        assert_eq!(
            processor.process_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            InputAction::Quit
        );
        assert_eq!(
            processor.process_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            InputAction::Submit
        );
        match processor.process_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)) {
            InputAction::PassThrough(k) => assert_eq!(k.code, KeyCode::Char('a')),
            _ => panic!("Expected PassThrough"),
        }
    }
}
