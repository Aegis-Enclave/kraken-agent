//! Config module - Configuration state management
//!
//! This module provides the configuration and theme management for the TUI.

use std::collections::HashMap;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use crate::error::{TuiError, TuiResult};

/// Serializable color type that can be converted to/from ratatui Color
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
#[derive(Default)]
pub enum SerialColor {
    /// Reset to default
    #[default]
    Default,
    /// Indexed color (0-255)
    Index(u8),
    /// RGB color
    Rgb { r: u8, g: u8, b: u8 },
    /// Named color
    Named(String),
}

impl From<Color> for SerialColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Reset => Self::Default,
            Color::Indexed(i) => Self::Index(i),
            Color::Rgb(r, g, b) => Self::Rgb { r, g, b },
            Color::Black => Self::Named("black".to_string()),
            Color::Red => Self::Named("red".to_string()),
            Color::Green => Self::Named("green".to_string()),
            Color::Yellow => Self::Named("yellow".to_string()),
            Color::Blue => Self::Named("blue".to_string()),
            Color::Magenta => Self::Named("magenta".to_string()),
            Color::Cyan => Self::Named("cyan".to_string()),
            Color::Gray => Self::Named("gray".to_string()),
            Color::DarkGray => Self::Named("dark_gray".to_string()),
            Color::LightRed => Self::Named("light_red".to_string()),
            Color::LightGreen => Self::Named("light_green".to_string()),
            Color::LightYellow => Self::Named("light_yellow".to_string()),
            Color::LightBlue => Self::Named("light_blue".to_string()),
            Color::LightMagenta => Self::Named("light_magenta".to_string()),
            Color::LightCyan => Self::Named("light_cyan".to_string()),
            Color::White => Self::Named("white".to_string()),
        }
    }
}

impl From<SerialColor> for Color {
    fn from(serial: SerialColor) -> Self {
        match serial {
            SerialColor::Default => Color::Reset,
            SerialColor::Index(i) => Color::Indexed(i),
            SerialColor::Rgb { r, g, b } => Color::Rgb(r, g, b),
            SerialColor::Named(name) => match name.to_lowercase().as_str() {
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "gray" => Color::Gray,
                "dark_gray" => Color::DarkGray,
                "light_red" => Color::LightRed,
                "light_green" => Color::LightGreen,
                "light_yellow" => Color::LightYellow,
                "light_blue" => Color::LightBlue,
                "light_magenta" => Color::LightMagenta,
                "light_cyan" => Color::LightCyan,
                "white" => Color::White,
                _ => Color::Reset,
            },
        }
    }
}

// ============================================================================
// Theme Configuration
// ============================================================================

/// Color configuration for the TUI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeColors {
    #[serde(default = "default_background")]
    pub background: SerialColor,
    #[serde(default = "default_text")]
    pub text: SerialColor,
    #[serde(default = "default_primary")]
    pub primary: SerialColor,
    #[serde(default = "default_secondary")]
    pub secondary: SerialColor,
    #[serde(default = "default_accent")]
    pub accent: SerialColor,
    #[serde(default = "default_error")]
    pub error: SerialColor,
    #[serde(default = "default_success")]
    pub success: SerialColor,
    #[serde(default = "default_warning")]
    pub warning: SerialColor,
}

fn default_background() -> SerialColor {
    SerialColor::Default
}
fn default_text() -> SerialColor {
    SerialColor::Default
}
fn default_primary() -> SerialColor {
    SerialColor::Index(13)
}
fn default_secondary() -> SerialColor {
    SerialColor::Index(14)
}
fn default_accent() -> SerialColor {
    SerialColor::Index(11)
}
fn default_error() -> SerialColor {
    SerialColor::Named("red".to_string())
}
fn default_success() -> SerialColor {
    SerialColor::Named("green".to_string())
}
fn default_warning() -> SerialColor {
    SerialColor::Named("yellow".to_string())
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            background: default_background(),
            text: default_text(),
            primary: default_primary(),
            secondary: default_secondary(),
            accent: default_accent(),
            error: default_error(),
            success: default_success(),
            warning: default_warning(),
        }
    }
}

impl ThemeColors {
    /// Convert all colors to ratatui Color for runtime use
    #[must_use]
    pub fn to_rgb_colors(&self) -> ThemeColorsRgb {
        ThemeColorsRgb {
            background: Color::from(self.background.clone()),
            text: Color::from(self.text.clone()),
            primary: Color::from(self.primary.clone()),
            secondary: Color::from(self.secondary.clone()),
            accent: Color::from(self.accent.clone()),
            error: Color::from(self.error.clone()),
            success: Color::from(self.success.clone()),
            warning: Color::from(self.warning.clone()),
        }
    }
}

/// Runtime color struct using ratatui Color (not serializable)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColorsRgb {
    pub background: Color,
    pub text: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
}

/// Chat-specific colors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatColors {
    #[serde(default = "default_user_bg")]
    pub user_bg: SerialColor,
    #[serde(default = "default_user_text")]
    pub user_text: SerialColor,
    #[serde(default = "default_assistant_bg")]
    pub assistant_bg: SerialColor,
    #[serde(default = "default_assistant_text")]
    pub assistant_text: SerialColor,
    #[serde(default = "default_system_bg")]
    pub system_bg: SerialColor,
    #[serde(default = "default_system_text")]
    pub system_text: SerialColor,
    #[serde(default = "default_tool_bg")]
    pub tool_bg: SerialColor,
    #[serde(default = "default_tool_text")]
    pub tool_text: SerialColor,
    #[serde(default = "default_code_bg")]
    pub code_bg: SerialColor,
    #[serde(default = "default_code_text")]
    pub code_text: SerialColor,
    #[serde(default = "default_border")]
    pub border: SerialColor,
    #[serde(default = "default_timestamp")]
    pub timestamp: SerialColor,
}

fn default_user_bg() -> SerialColor {
    SerialColor::Index(238)
}
fn default_user_text() -> SerialColor {
    SerialColor::Index(252)
}
fn default_assistant_bg() -> SerialColor {
    SerialColor::Index(236)
}
fn default_assistant_text() -> SerialColor {
    SerialColor::Index(248)
}
fn default_system_bg() -> SerialColor {
    SerialColor::Index(235)
}
fn default_system_text() -> SerialColor {
    SerialColor::Index(245)
}
fn default_tool_bg() -> SerialColor {
    SerialColor::Index(237)
}
fn default_tool_text() -> SerialColor {
    SerialColor::Index(243)
}
fn default_code_bg() -> SerialColor {
    SerialColor::Index(233)
}
fn default_code_text() -> SerialColor {
    SerialColor::Index(252)
}
fn default_border() -> SerialColor {
    SerialColor::Index(240)
}
fn default_timestamp() -> SerialColor {
    SerialColor::Index(246)
}

impl Default for ChatColors {
    fn default() -> Self {
        Self {
            user_bg: default_user_bg(),
            user_text: default_user_text(),
            assistant_bg: default_assistant_bg(),
            assistant_text: default_assistant_text(),
            system_bg: default_system_bg(),
            system_text: default_system_text(),
            tool_bg: default_tool_bg(),
            tool_text: default_tool_text(),
            code_bg: default_code_bg(),
            code_text: default_code_text(),
            border: default_border(),
            timestamp: default_timestamp(),
        }
    }
}

impl ChatColors {
    /// Convert all colors to ratatui Color for runtime use
    #[must_use]
    pub fn to_rgb_colors(&self) -> ChatColorsRgb {
        ChatColorsRgb {
            user_bg: Color::from(self.user_bg.clone()),
            user_text: Color::from(self.user_text.clone()),
            assistant_bg: Color::from(self.assistant_bg.clone()),
            assistant_text: Color::from(self.assistant_text.clone()),
            system_bg: Color::from(self.system_bg.clone()),
            system_text: Color::from(self.system_text.clone()),
            tool_bg: Color::from(self.tool_bg.clone()),
            tool_text: Color::from(self.tool_text.clone()),
            code_bg: Color::from(self.code_bg.clone()),
            code_text: Color::from(self.code_text.clone()),
            border: Color::from(self.border.clone()),
            timestamp: Color::from(self.timestamp.clone()),
        }
    }
}

/// Runtime color struct using ratatui Color (not serializable)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChatColorsRgb {
    pub user_bg: Color,
    pub user_text: Color,
    pub assistant_bg: Color,
    pub assistant_text: Color,
    pub system_bg: Color,
    pub system_text: Color,
    pub tool_bg: Color,
    pub tool_text: Color,
    pub code_bg: Color,
    pub code_text: Color,
    pub border: Color,
    pub timestamp: Color,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeConfig {
    pub name: String,
    pub colors: ThemeColors,
    pub chat: ChatColors,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            colors: ThemeColors::default(),
            chat: ChatColors::default(),
        }
    }
}

impl ThemeConfig {
    /// Convert all colors to ratatui Color for runtime use
    #[must_use]
    pub fn to_rgb_colors(&self) -> ThemeConfigRgb {
        ThemeConfigRgb {
            name: self.name.clone(),
            colors: self.colors.to_rgb_colors(),
            chat: self.chat.to_rgb_colors(),
        }
    }
}

/// Runtime color config using ratatui Color (not serializable)
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeConfigRgb {
    pub name: String,
    pub colors: ThemeColorsRgb,
    pub chat: ChatColorsRgb,
}

/// Predefined themes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinTheme {
    Default,
    Dark,
    Light,
    Gruvbox,
}

impl BuiltinTheme {
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            Self::Default,
            Self::Dark,
            Self::Light,
            Self::Gruvbox,
        ]
    }

    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "default" => Some(Self::Default),
            "dark" => Some(Self::Dark),
            "light" => Some(Self::Light),
            "gruvbox" => Some(Self::Gruvbox),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_config(&self) -> ThemeConfig {
        match self {
            Self::Default => default_theme(),
            Self::Dark => dark_theme(),
            Self::Light => light_theme(),
            Self::Gruvbox => gruvbox_theme(),
        }
    }
}

fn default_theme() -> ThemeConfig {
    ThemeConfig {
        name: "default".to_string(),
        colors: ThemeColors::default(),
        chat: ChatColors::default(),
    }
}

fn dark_theme() -> ThemeConfig {
    ThemeConfig {
        name: "dark".to_string(),
        colors: ThemeColors {
            background: SerialColor::Named("black".to_string()),
            text: SerialColor::Named("white".to_string()),
            primary: SerialColor::Named("cyan".to_string()),
            secondary: SerialColor::Named("blue".to_string()),
            accent: SerialColor::Named("yellow".to_string()),
            error: SerialColor::Named("red".to_string()),
            success: SerialColor::Named("green".to_string()),
            warning: SerialColor::Named("yellow".to_string()),
        },
        chat: ChatColors {
            user_bg: SerialColor::Index(59),
            user_text: SerialColor::Named("white".to_string()),
            assistant_bg: SerialColor::Index(24),
            assistant_text: SerialColor::Named("white".to_string()),
            system_bg: SerialColor::Index(23),
            system_text: SerialColor::Named("white".to_string()),
            tool_bg: SerialColor::Index(23),
            tool_text: SerialColor::Named("white".to_string()),
            code_bg: SerialColor::Index(16),
            code_text: SerialColor::Named("white".to_string()),
            border: SerialColor::Index(59),
            timestamp: SerialColor::Index(245),
        },
    }
}

fn light_theme() -> ThemeConfig {
    ThemeConfig {
        name: "light".to_string(),
        colors: ThemeColors::default(),
        chat: ChatColors {
            user_bg: SerialColor::Index(253),
            user_text: SerialColor::Named("black".to_string()),
            assistant_bg: SerialColor::Index(252),
            assistant_text: SerialColor::Named("black".to_string()),
            system_bg: SerialColor::Index(251),
            system_text: SerialColor::Named("black".to_string()),
            tool_bg: SerialColor::Index(251),
            tool_text: SerialColor::Named("black".to_string()),
            code_bg: SerialColor::Index(250),
            code_text: SerialColor::Named("black".to_string()),
            border: SerialColor::Index(248),
            timestamp: SerialColor::Index(246),
        },
    }
}

fn gruvbox_theme() -> ThemeConfig {
    ThemeConfig {
        name: "gruvbox".to_string(),
        colors: ThemeColors {
            background: SerialColor::Rgb {
                r: 40,
                g: 40,
                b: 40,
            }, // #282828
            text: SerialColor::Rgb {
                r: 235,
                g: 219,
                b: 178,
            }, // #ebdbb2
            primary: SerialColor::Rgb {
                r: 211,
                g: 134,
                b: 155,
            }, // #d3869b
            secondary: SerialColor::Rgb {
                r: 131,
                g: 165,
                b: 152,
            }, // #83a598
            accent: SerialColor::Rgb {
                r: 250,
                g: 189,
                b: 47,
            }, // #fabd2f
            error: SerialColor::Rgb {
                r: 204,
                g: 36,
                b: 29,
            }, // #cc241d
            success: SerialColor::Rgb {
                r: 184,
                g: 187,
                b: 38,
            }, // #b8bb26
            warning: SerialColor::Rgb {
                r: 215,
                g: 153,
                b: 33,
            }, // #d79921
        },
        chat: ChatColors {
            user_bg: SerialColor::Rgb {
                r: 60,
                g: 56,
                b: 54,
            }, // #3c3836
            user_text: SerialColor::Rgb {
                r: 235,
                g: 219,
                b: 178,
            },
            assistant_bg: SerialColor::Rgb {
                r: 40,
                g: 40,
                b: 40,
            }, // #282828
            assistant_text: SerialColor::Rgb {
                r: 235,
                g: 219,
                b: 178,
            },
            system_bg: SerialColor::Rgb {
                r: 27,
                g: 32,
                b: 33,
            }, // #1d2021
            system_text: SerialColor::Rgb {
                r: 146,
                g: 131,
                b: 116,
            }, // #928374
            tool_bg: SerialColor::Rgb {
                r: 40,
                g: 40,
                b: 40,
            },
            tool_text: SerialColor::Rgb {
                r: 142,
                g: 192,
                b: 124,
            }, // #8ec07c
            code_bg: SerialColor::Rgb {
                r: 27,
                g: 32,
                b: 33,
            }, // #1d2021
            code_text: SerialColor::Rgb {
                r: 235,
                g: 219,
                b: 178,
            },
            border: SerialColor::Rgb {
                r: 168,
                g: 153,
                b: 132,
            }, // #a89984
            timestamp: SerialColor::Rgb {
                r: 146,
                g: 131,
                b: 116,
            }, // #928374
        },
    }
}

/// Theme manager
#[derive(Debug, Clone)]
pub struct ThemeManager {
    current_theme: ThemeConfig,
    custom_themes: HashMap<String, ThemeConfig>,
}

impl ThemeManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_theme: ThemeConfig::default(),
            custom_themes: HashMap::new(),
        }
    }

    pub fn load_builtin(&mut self, theme: BuiltinTheme) {
        self.current_theme = theme.to_config();
    }

    pub fn load_theme(&mut self, name: &str) -> TuiResult<()> {
        if let Some(builtin) = BuiltinTheme::from_name(name) {
            self.current_theme = builtin.to_config();
            Ok(())
        } else if let Some(custom) = self.custom_themes.get(name) {
            self.current_theme = custom.clone();
            Ok(())
        } else {
            Err(TuiError::config(format!("Theme '{name}' not found")))
        }
    }

    #[must_use]
    pub fn current_theme(&self) -> &ThemeConfig {
        &self.current_theme
    }

    pub fn current_theme_mut(&mut self) -> &mut ThemeConfig {
        &mut self.current_theme
    }

    pub fn add_custom_theme(&mut self, theme: ThemeConfig) {
        self.custom_themes.insert(theme.name.clone(), theme);
    }

    #[must_use]
    pub fn all_theme_names(&self) -> Vec<String> {
        let mut names: Vec<String> = BuiltinTheme::all()
            .iter()
            .map(|t| t.to_config().name.clone())
            .collect();

        names.extend(self.custom_themes.keys().cloned());
        names.sort();
        names
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Display configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DisplayConfig {
    #[serde(default = "default_show_timestamps")]
    pub show_timestamps: bool,
    #[serde(default = "default_show_session_name")]
    pub show_session_name: bool,
    #[serde(default = "default_syntax_highlighting")]
    pub syntax_highlighting: bool,
    #[serde(default)]
    pub max_message_width: u16,
}

fn default_show_timestamps() -> bool {
    true
}
fn default_show_session_name() -> bool {
    true
}
fn default_syntax_highlighting() -> bool {
    true
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_timestamps: default_show_timestamps(),
            show_session_name: default_show_session_name(),
            syntax_highlighting: default_syntax_highlighting(),
            max_message_width: 0,
        }
    }
}

/// Editor configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EditorConfig {
    #[serde(default = "default_auto_indent")]
    pub auto_indent: bool,
    #[serde(default = "default_tab_width")]
    pub tab_width: u8,
    #[serde(default = "default_history_size")]
    pub history_size: usize,
}

fn default_auto_indent() -> bool {
    true
}
fn default_tab_width() -> u8 {
    4
}
fn default_history_size() -> usize {
    100
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            auto_indent: default_auto_indent(),
            tab_width: default_tab_width(),
            history_size: default_history_size(),
        }
    }
}

/// Input mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
    Command,
}

/// Pane that can receive focus for keyboard navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FocusPane {
    #[default]
    Chat,
    Composer,
    Toolbar,
    Sidebar,
}

/// Main TUI configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TuiConfig {
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub editor: EditorConfig,
    #[serde(default)]
    pub default_mode: InputMode,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            display: DisplayConfig::default(),
            editor: EditorConfig::default(),
            default_mode: InputMode::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_colors_default() {
        let colors = ThemeColors::default();
        assert!(colors.background != SerialColor::Named("red".to_string()));
    }

    #[test]
    fn test_theme_colors_to_rgb() {
        let colors = ThemeColors::default();
        let rgb = colors.to_rgb_colors();
        // Just verify conversion works
        assert_eq!(rgb.background, Color::from(colors.background));
    }

    #[test]
    fn test_chat_colors_to_rgb() {
        let colors = ChatColors::default();
        let rgb = colors.to_rgb_colors();
        assert_eq!(rgb.user_bg, Color::from(colors.user_bg));
    }

    #[test]
    fn test_serial_color_conversion() {
        // Test round-trip conversion
        let serial = SerialColor::Named("red".to_string());
        let color = Color::from(serial.clone());
        let serial2 = SerialColor::from(color);
        assert_eq!(serial, serial2);

        // Test indexed
        let serial = SerialColor::Index(238);
        let color = Color::from(serial.clone());
        let serial2 = SerialColor::from(color);
        assert_eq!(serial, serial2);
    }

    #[test]
    fn test_builtin_theme_all() {
        let themes = BuiltinTheme::all();
        assert!(themes.len() >= 1);
    }

    #[test]
    fn test_theme_manager() {
        let mut manager = ThemeManager::new();
        assert_eq!(manager.current_theme().name, "default");

        manager.load_builtin(BuiltinTheme::Dark);
        assert_eq!(manager.current_theme().name, "dark");
    }

    #[test]
    fn test_tui_config_default() {
        let config = TuiConfig::default();
        assert_eq!(config.theme.name, "default");
        assert!(config.display.show_timestamps);
    }

    #[test]
    fn test_theme_config_serialization() {
        let config = ThemeConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: ThemeConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }
}
