//! Model picker component for TUI
//!
//! This module provides an overlay dialog for choosing a model,
//! organized by provider.

use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, ListState, Row, Table, TableState},
    Frame,
};

use crate::protocol::types::ModelOptionProvider;
use crate::state::config::ChatColorsRgb;
use crate::ui::borders::render_gradient_border;

/// Current stage of the model picker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelPickerStage {
    /// Selecting a provider
    Provider,
    /// Selecting a model within the current provider
    Model,
}

/// Model picker popup component
#[derive(Debug, Clone)]
pub struct ModelPicker {
    /// Providers to select from
    providers: Vec<ModelOptionProvider>,
    /// Visibility state
    visible: bool,
    /// Loading state (waiting for gateway)
    loading: bool,
    /// Current navigation stage
    stage: ModelPickerStage,
    /// Selected provider index
    provider_index: usize,
    /// Selected model index within the current provider
    model_index: usize,
    /// List state for provider scrolling
    provider_state: ListState,
    /// Table state for model scrolling
    model_state: TableState,
}

impl ModelPicker {
    /// Create a new model picker instance
    #[must_use]
    pub fn new(_colors: ChatColorsRgb) -> Self {
        Self {
            providers: Vec::new(),
            visible: false,
            loading: false,
            stage: ModelPickerStage::Provider,
            provider_index: 0,
            model_index: 0,
            provider_state: ListState::default(),
            model_state: TableState::default(),
        }
    }

    /// Show the model picker in loading state
    pub fn show_loading(&mut self) {
        self.visible = true;
        self.loading = true;
        self.providers.clear();
        self.stage = ModelPickerStage::Provider;
        self.provider_index = 0;
        self.model_index = 0;
        self.provider_state.select(Some(0));
        self.model_state.select(Some(0));
    }

    /// Show the model picker with providers
    pub fn show(&mut self, providers: Vec<ModelOptionProvider>) {
        self.providers = providers;
        self.loading = false;
        self.stage = ModelPickerStage::Provider;
        self.provider_index = 0;
        self.model_index = 0;
        self.provider_state.select(Some(0));
        self.model_state.select(Some(0));
        self.visible = !self.providers.is_empty();
    }

    /// Hide the model picker
    pub fn hide(&mut self) {
        self.visible = false;
        self.loading = false;
        self.providers.clear();
        self.stage = ModelPickerStage::Provider;
        self.provider_index = 0;
        self.model_index = 0;
        self.provider_state.select(None);
        self.model_state.select(None);
    }

    /// Check if the model picker is visible
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Check if the model picker is loading
    #[must_use]
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Get the current stage
    pub fn stage(&self) -> ModelPickerStage {
        self.stage
    }

    /// Get the currently selected provider
    #[must_use]
    pub fn selected_provider(&self) -> Option<&ModelOptionProvider> {
        if self.visible {
            self.providers.get(self.provider_index)
        } else {
            None
        }
    }

    /// Get the currently selected model name
    /// Only returns a value if in Model stage
    #[must_use]
    pub fn selected_model(&self) -> Option<String> {
        if self.stage != ModelPickerStage::Model {
            return None;
        }
        let provider = self.selected_provider()?;
        let models = provider.models.as_ref()?;
        models.get(self.model_index).cloned()
    }

    /// Select the next item in the current stage
    pub fn select_next(&mut self) {
        match self.stage {
            ModelPickerStage::Provider => {
                if !self.providers.is_empty() {
                    self.provider_index = (self.provider_index + 1) % self.providers.len();
                    self.provider_state.select(Some(self.provider_index));
                    self.model_index = 0;
                    self.model_state.select(Some(0));
                }
            }
            ModelPickerStage::Model => {
                if let Some(provider) = self.providers.get(self.provider_index) {
                    if let Some(models) = &provider.models {
                        if !models.is_empty() {
                            self.model_index = (self.model_index + 1) % models.len();
                            self.model_state.select(Some(self.model_index));
                        }
                    }
                }
            }
        }
    }

    /// Select the previous item in the current stage
    pub fn select_prev(&mut self) {
        match self.stage {
            ModelPickerStage::Provider => {
                if !self.providers.is_empty() {
                    self.provider_index = if self.provider_index == 0 {
                        self.providers.len() - 1
                    } else {
                        self.provider_index - 1
                    };
                    self.provider_state.select(Some(self.provider_index));
                    self.model_index = 0;
                    self.model_state.select(Some(0));
                }
            }
            ModelPickerStage::Model => {
                if let Some(provider) = self.providers.get(self.provider_index) {
                    if let Some(models) = &provider.models {
                        if !models.is_empty() {
                            self.model_index = if self.model_index == 0 {
                                models.len() - 1
                            } else {
                                self.model_index - 1
                            };
                            self.model_state.select(Some(self.model_index));
                        }
                    }
                }
            }
        }
    }

    /// Enter the model list for the selected provider
    pub fn enter_provider(&mut self) -> bool {
        if self.stage == ModelPickerStage::Provider {
            if let Some(provider) = self.providers.get(self.provider_index) {
                if provider.models.as_ref().is_some_and(|m| !m.is_empty()) {
                    self.stage = ModelPickerStage::Model;
                    self.model_index = 0;
                    self.model_state.select(Some(0));
                    return true;
                }
            }
        }
        false
    }

    /// Go back to provider selection
    pub fn back_to_providers(&mut self) {
        self.stage = ModelPickerStage::Provider;
        self.model_index = 0;
        self.model_state.select(Some(0));
    }

    /// Render the model picker popup
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        animation_frame: u64,
        is_running: bool,
    ) {
        if !self.visible {
            return;
        }

        // Center popup
        let popup_area = self.get_popup_rect(area);

        frame.render_widget(Clear, popup_area);

        // Render animated gradient border
        render_gradient_border(
            frame.buffer_mut(),
            popup_area,
            animation_frame,
            true,
            is_running,
        );

        if self.loading {
            self.render_loading(frame, popup_area);
            return;
        }

        if self.providers.is_empty() {
            return;
        }

        match self.stage {
            ModelPickerStage::Provider => self.render_providers(frame, popup_area),
            ModelPickerStage::Model => self.render_models(frame, popup_area),
        }
    }

    fn render_loading(&self, frame: &mut Frame, area: Rect) {
        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 2,
            vertical: 2,
        });
        let text = vec![
            Line::from(""),
            Line::from(" Fetching model options from gateway... ")
                .alignment(ratatui::layout::Alignment::Center),
            Line::from(" Please wait... ")
                .alignment(ratatui::layout::Alignment::Center)
                .style(Style::default().fg(Color::Gray).italic()),
        ];
        frame.render_widget(
            ratatui::widgets::Paragraph::new(text).block(Block::default()),
            inner,
        );
    }

    fn get_popup_rect(&self, area: Rect) -> Rect {
        let width = 80.min(area.width);
        let height = 24.min(area.height);
        let x = area.x + (area.width - width) / 2;
        let y = area.y + (area.height - height) / 2;
        Rect::new(x, y, width, height)
    }

    fn render_providers(&mut self, frame: &mut Frame, area: Rect) {
        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 2,
            vertical: 2,
        });

        let items: Vec<ListItem> = self
            .providers
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let current = p.is_current.unwrap_or(false);
                let auth = p.authenticated.unwrap_or(true);
                let model_count = p
                    .total_models
                    .or(p.models.as_ref().map(std::vec::Vec::len))
                    .unwrap_or(0);

                let icon = if current { " ◈ " } else { " ◇ " };
                let auth_status = if auth {
                    Span::styled(
                        " AUTH ",
                        Style::default().fg(Color::Green).bg(Color::Rgb(30, 40, 30)),
                    )
                } else {
                    Span::styled(
                        " NOAUTH ",
                        Style::default().fg(Color::Red).bg(Color::Rgb(40, 30, 30)),
                    )
                };

                let content = Line::from(vec![
                    Span::styled(
                        icon,
                        Style::default().fg(if i == self.provider_index {
                            Color::Yellow
                        } else {
                            Color::DarkGray
                        }),
                    ),
                    Span::styled(format!("{:<15}", p.name), Style::default().bold()),
                    Span::raw(" │ "),
                    auth_status,
                    Span::raw(" │ "),
                    Span::styled(
                        format!("{:>3} models", model_count),
                        Style::default().fg(Color::Gray),
                    ),
                ]);

                let style = if i == self.provider_index {
                    Style::default().bg(Color::Rgb(40, 40, 50))
                } else {
                    Style::default()
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let block = Block::default()
            .title(Span::styled(" SELECT PROVIDER ", Style::default().bold()))
            .title_alignment(ratatui::layout::Alignment::Center);

        let list = List::new(items).block(block);
        frame.render_stateful_widget(list, inner, &mut self.provider_state);

        // Render helper hints
        let hints = Line::from(vec![
            " ↑↓ ".bold().yellow(),
            "navigate".into(),
            " │ ".into(),
            " Enter ".bold().yellow(),
            "select provider".into(),
            " │ ".into(),
            " Esc ".bold().yellow(),
            "cancel".into(),
        ])
        .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(
            ratatui::widgets::Paragraph::new(hints),
            Rect::new(area.x, area.y + area.height - 2, area.width, 1),
        );
    }

    fn render_models(&mut self, frame: &mut Frame, area: Rect) {
        let provider = match self.providers.get(self.provider_index) {
            Some(p) => p,
            None => return,
        };
        let models = match &provider.models {
            Some(m) => m,
            None => return,
        };

        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 2,
            vertical: 2,
        });

        let header = Row::new(vec!["  #", "  Model Name"])
            .style(Style::default().fg(Color::Yellow).bold())
            .height(1)
            .bottom_margin(1);

        let rows: Vec<Row> = models
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let style = if i == self.model_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::Rgb(40, 40, 50))
                        .bold()
                } else {
                    Style::default()
                };

                let selector = if i == self.model_index { " > " } else { "   " };

                Row::new(vec![
                    format!("  {:02}", i + 1),
                    format!("{}{}", selector, m),
                ])
                .style(style)
            })
            .collect();

        let widths = [Constraint::Length(6), Constraint::Min(40)];

        let title = format!(" MODELS: {} ", provider.name.to_uppercase());
        let block = Block::default()
            .title(Span::styled(title, Style::default().bold()))
            .title_alignment(ratatui::layout::Alignment::Center);

        let table = Table::new(rows, widths)
            .header(header)
            .block(block)
            .column_spacing(2);

        frame.render_stateful_widget(table, inner, &mut self.model_state);

        // Render helper hints
        let hints = Line::from(vec![
            " ↑↓ ".bold().yellow(),
            "navigate".into(),
            " │ ".into(),
            " Enter ".bold().yellow(),
            "apply model".into(),
            " │ ".into(),
            " Esc ".bold().yellow(),
            "back".into(),
        ])
        .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(
            ratatui::widgets::Paragraph::new(hints),
            Rect::new(area.x, area.y + area.height - 2, area.width, 1),
        );
    }
}
