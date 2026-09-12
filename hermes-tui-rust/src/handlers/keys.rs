//! Keys module - Keyboard shortcut handlers
//!
//! This module provides specialized handlers for keyboard shortcuts
//! and key-based navigation in the TUI.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Keyboard navigation handler for chat transcript
#[derive(Debug)]
pub struct ChatKeyboardHandler {
    scroll_step: u16,
    page_step: u16,
}

impl Default for ChatKeyboardHandler {
    fn default() -> Self {
        Self {
            scroll_step: 1,
            page_step: 10,
        }
    }
}

impl ChatKeyboardHandler {
    /// Create a new chat keyboard handler
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new chat keyboard handler with custom scroll steps
    #[must_use]
    pub fn with_steps(scroll_step: u16, page_step: u16) -> Self {
        Self {
            scroll_step,
            page_step,
        }
    }

    /// Handle a key event for chat navigation
    /// Returns the scroll amount (positive = down, negative = up)
    #[must_use]
    pub fn handle_key(&self, key: &KeyEvent, visible_height: u16) -> i16 {
        let code = key.code;
        let modifiers = key.modifiers;

        // Handle scroll up
        if modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('u') {
            return -(visible_height as i16 / 2); // Half page up
        }

        // Handle scroll down
        if modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('d') {
            return visible_height as i16 / 2; // Half page down
        }

        // Handle page up
        if code == KeyCode::PageUp {
            return -(self.page_step as i16);
        }

        // Handle page down
        if code == KeyCode::PageDown {
            return self.page_step as i16;
        }

        // Handle arrow up
        if code == KeyCode::Up {
            return -1;
        }

        // Handle arrow down
        if code == KeyCode::Down {
            return 1;
        }

        // Handle home (go to top)
        if code == KeyCode::Home {
            return i16::MIN + 1; // Special value for "go to top"
        }

        // Handle end (go to bottom)
        if code == KeyCode::End {
            return i16::MAX - 1; // Special value for "go to bottom"
        }

        0 // No scroll
    }

    /// Get the scroll step size
    #[must_use]
    pub fn scroll_step(&self) -> u16 {
        self.scroll_step
    }

    /// Get the page step size
    #[must_use]
    pub fn page_step(&self) -> u16 {
        self.page_step
    }
}

/// Keyboard handler for input composer
#[derive(Debug, Default)]
pub struct InputKeyboardHandler {
    tab_width: usize,
}

impl InputKeyboardHandler {
    /// Create a new input keyboard handler
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new input keyboard handler with custom tab width
    #[must_use]
    pub fn with_tab_width(tab_width: usize) -> Self {
        Self { tab_width }
    }

    /// Handle a key event for text input
    /// Returns the action to take
    #[must_use]
    pub fn handle_input_key(
        &self,
        key: KeyEvent,
        input: &str,
        cursor_pos: usize,
    ) -> TextInputAction {
        let code = key.code;
        let modifiers = key.modifiers;

        // Handle backspace
        if code == KeyCode::Backspace {
            if cursor_pos > 0 {
                return TextInputAction::DeleteBeforeCursor;
            }
            return TextInputAction::None;
        }

        // Handle delete
        if code == KeyCode::Delete {
            if cursor_pos < input.len() {
                return TextInputAction::DeleteAfterCursor;
            }
            return TextInputAction::None;
        }

        // Handle left arrow
        if code == KeyCode::Left {
            if modifiers.contains(KeyModifiers::CONTROL) {
                // Move to previous word
                return TextInputAction::MoveToPreviousWord;
            }
            if cursor_pos > 0 {
                return TextInputAction::MoveCursorLeft;
            }
            return TextInputAction::None;
        }

        // Handle right arrow
        if code == KeyCode::Right {
            if modifiers.contains(KeyModifiers::CONTROL) {
                // Move to next word
                return TextInputAction::MoveToNextWord;
            }
            if cursor_pos < input.len() {
                return TextInputAction::MoveCursorRight;
            }
            return TextInputAction::None;
        }

        // Handle home
        if code == KeyCode::Home {
            if modifiers.contains(KeyModifiers::SHIFT) {
                return TextInputAction::SelectToStart;
            }
            return TextInputAction::MoveToStart;
        }

        // Handle end
        if code == KeyCode::End {
            if modifiers.contains(KeyModifiers::SHIFT) {
                return TextInputAction::SelectToEnd;
            }
            return TextInputAction::MoveToEnd;
        }

        // Handle tab
        if code == KeyCode::Tab {
            if modifiers.contains(KeyModifiers::SHIFT) {
                return TextInputAction::Unindent;
            }
            return TextInputAction::Indent(self.tab_width);
        }

        // Handle backtab (shift+tab)
        if code == KeyCode::BackTab {
            return TextInputAction::Unindent;
        }

        // Handle character input
        if let KeyCode::Char(c) = code {
            return TextInputAction::InsertChar(c);
        }

        TextInputAction::None
    }

    /// Get the tab width
    #[must_use]
    pub fn tab_width(&self) -> usize {
        self.tab_width
    }
}

/// Action for text input manipulation
#[derive(Debug, Clone, PartialEq)]
pub enum TextInputAction {
    /// Insert a character at cursor position
    InsertChar(char),
    /// Delete character before cursor
    DeleteBeforeCursor,
    /// Delete character after cursor
    DeleteAfterCursor,
    /// Move cursor left
    MoveCursorLeft,
    /// Move cursor right
    MoveCursorRight,
    /// Move to start of line
    MoveToStart,
    /// Move to end of line
    MoveToEnd,
    /// Move to previous word
    MoveToPreviousWord,
    /// Move to next word
    MoveToNextWord,
    /// Select from cursor to start
    SelectToStart,
    /// Select from cursor to end
    SelectToEnd,
    /// Indent (insert spaces)
    Indent(usize),
    /// Unindent (remove leading whitespace)
    Unindent,
    /// No action
    None,
}

/// Keyboard shortcut registry for custom shortcuts
#[derive(Debug, Default)]
pub struct ShortcutRegistry {
    shortcuts: Vec<Shortcut>,
}

impl ShortcutRegistry {
    /// Create a new shortcut registry
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new shortcut
    pub fn register(&mut self, shortcut: Shortcut) {
        self.shortcuts.push(shortcut);
    }

    /// Register multiple shortcuts
    pub fn register_many(&mut self, shortcuts: Vec<Shortcut>) {
        self.shortcuts.extend(shortcuts);
    }

    /// Find shortcut by key event
    #[must_use]
    pub fn find_by_key(&self, key: &KeyEvent) -> Option<&Shortcut> {
        self.shortcuts.iter().find(|s| s.key == *key)
    }

    /// Find shortcut by action name
    #[must_use]
    pub fn find_by_action(&self, action: &str) -> Vec<&Shortcut> {
        self.shortcuts
            .iter()
            .filter(|s| s.action == action)
            .collect()
    }

    /// Get all shortcuts
    #[must_use]
    pub fn all(&self) -> &[Shortcut] {
        &self.shortcuts
    }

    /// Clear all shortcuts
    pub fn clear(&mut self) {
        self.shortcuts.clear();
    }
}

/// A keyboard shortcut definition
#[derive(Debug, Clone, PartialEq)]
pub struct Shortcut {
    /// The key combination
    pub key: KeyEvent,
    /// The action name
    pub action: String,
    /// Human-readable description
    pub description: String,
    /// Priority (higher = checked first)
    pub priority: u8,
}

impl Shortcut {
    /// Create a new shortcut
    pub fn new(key: KeyEvent, action: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key,
            action: action.into(),
            description: description.into(),
            priority: 0,
        }
    }

    /// Create a new shortcut with priority
    pub fn with_priority(
        key: KeyEvent,
        action: impl Into<String>,
        description: impl Into<String>,
        priority: u8,
    ) -> Self {
        Self {
            key,
            action: action.into(),
            description: description.into(),
            priority,
        }
    }
}

/// Global keyboard state manager
#[derive(Debug)]
pub struct KeyboardState {
    /// Currently pressed modifier keys
    pub active_modifiers: KeyModifiers,
    /// Whether caps lock is active
    pub caps_lock: bool,
    /// Whether num lock is active
    pub num_lock: bool,
}
impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            active_modifiers: KeyModifiers::NONE,
            caps_lock: false,
            num_lock: false,
        }
    }
}

impl KeyboardState {
    /// Create a new keyboard state
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Update state from a key event
    pub fn update_from_key(&mut self, key: &KeyEvent) {
        self.active_modifiers = key.modifiers;
    }

    /// Check if a modifier is active
    #[must_use]
    pub fn has_modifier(&self, modifier: KeyModifiers) -> bool {
        self.active_modifiers.contains(modifier)
    }

    /// Reset to default state
    pub fn reset(&mut self) {
        self.active_modifiers = KeyModifiers::NONE;
        self.caps_lock = false;
        self.num_lock = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_keyboard_handler_scroll() {
        let handler = ChatKeyboardHandler::new();

        let page_up = KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE);
        let page_down = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);
        let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);

        assert_eq!(handler.handle_key(&page_up, 100), -10); // page_step = 10
        assert_eq!(handler.handle_key(&page_down, 100), 10);
        assert_eq!(handler.handle_key(&up, 100), -1);
        assert_eq!(handler.handle_key(&down, 100), 1);
    }

    #[test]
    fn test_chat_keyboard_handler_home_end() {
        let handler = ChatKeyboardHandler::new();

        let home = KeyEvent::new(KeyCode::Home, KeyModifiers::NONE);
        let end = KeyEvent::new(KeyCode::End, KeyModifiers::NONE);

        assert_eq!(handler.handle_key(&home, 100), i16::MIN + 1);
        assert_eq!(handler.handle_key(&end, 100), i16::MAX - 1);
    }

    #[test]
    fn test_input_keyboard_handler_basic() {
        let handler = InputKeyboardHandler::new();

        let backspace = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        let del = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        let left = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        let right = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);

        assert_eq!(
            handler.handle_input_key(backspace, "test", 2),
            TextInputAction::DeleteBeforeCursor
        );
        assert_eq!(
            handler.handle_input_key(del, "test", 2),
            TextInputAction::DeleteAfterCursor
        );
        assert_eq!(
            handler.handle_input_key(left, "test", 2),
            TextInputAction::MoveCursorLeft
        );
        assert_eq!(
            handler.handle_input_key(right, "test", 2),
            TextInputAction::MoveCursorRight
        );
    }

    #[test]
    fn test_input_keyboard_handler_word_navigation() {
        let handler = InputKeyboardHandler::new();

        let ctrl_left = KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL);
        let ctrl_right = KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL);

        assert_eq!(
            handler.handle_input_key(ctrl_left, "hello world", 5),
            TextInputAction::MoveToPreviousWord
        );
        assert_eq!(
            handler.handle_input_key(ctrl_right, "hello world", 5),
            TextInputAction::MoveToNextWord
        );
    }

    #[test]
    fn test_shortcut_registry() {
        let mut registry = ShortcutRegistry::new();

        let shortcut = Shortcut::new(
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
            "save",
            "Save current session",
        );

        registry.register(shortcut.clone());

        let ctrl_s = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert!(registry.find_by_key(&ctrl_s).is_some());
        assert_eq!(registry.find_by_action("save").len(), 1);
    }

    #[test]
    fn test_keyboard_state() {
        let mut state = KeyboardState::new();

        let ctrl_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        state.update_from_key(&ctrl_a);

        assert!(state.has_modifier(KeyModifiers::CONTROL));
        assert!(!state.has_modifier(KeyModifiers::SHIFT));
    }
}
