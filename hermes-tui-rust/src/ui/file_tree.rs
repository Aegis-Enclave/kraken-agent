use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Padding},
    Frame,
};
use std::fs;
use std::path::PathBuf;

pub struct FileTree {
    pub current_dir: PathBuf,
    pub items: Vec<PathBuf>,
    pub state: ListState,
}

impl FileTree {
    pub fn new() -> Self {
        let mut tree = Self {
            current_dir: PathBuf::from("."),
            items: Vec::new(),
            state: ListState::default(),
        };
        tree.refresh();
        if !tree.items.is_empty() {
            tree.state.select(Some(0));
        }
        tree
    }

    pub fn refresh(&mut self) {
        let mut new_items = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if name_str == "target" || name_str == ".git" || name_str == ".hermes" {
                        continue;
                    }
                }
                new_items.push(path);
            }
        }
        new_items.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();
            if a_is_dir != b_is_dir {
                b_is_dir.cmp(&a_is_dir)
            } else {
                a.cmp(b)
            }
        });
        self.items = new_items;
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn selected_path(&self) -> Option<PathBuf> {
        self.state.selected().map(|i| self.items[i].clone())
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        is_active: bool,
        animation_frame: u64,
        is_running: bool,
    ) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|path| {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                let is_dir = path.is_dir();

                let (icon, color) = if is_dir {
                    ("📁 ", Color::Rgb(250, 189, 47)) // Gruvbox Yellow
                } else {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let icon = match ext {
                        "rs" => "🦀 ",
                        "toml" => "⚙️ ",
                        "md" => "📝 ",
                        "js" => "📜 ",
                        "json" => "📊 ",
                        _ => "📄 ",
                    };
                    let color = match ext {
                        "rs" => Color::Rgb(222, 165, 132),
                        "toml" => Color::Rgb(184, 187, 38),
                        _ => Color::Gray,
                    };
                    (icon, color)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(icon, Style::default().fg(color)),
                    Span::styled(name, Style::default().fg(color)),
                ]))
            })
            .collect();

        let border_style = if is_active {
            Style::default().fg(Color::Rgb(250, 189, 47))
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let border_type = if is_active {
            BorderType::Thick
        } else {
            BorderType::Plain
        };

        let block = Block::default()
            .title(Span::styled(
                " TREE ",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style)
            .padding(Padding::new(1, 1, 1, 1));

        frame.render_stateful_widget(
            List::new(items)
                .block(block)
                .highlight_style(
                    Style::default()
                        .bg(Color::Rgb(60, 56, 54))
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> "),
            area,
            &mut self.state,
        );

        crate::ui::borders::render_gradient_border(
            frame.buffer_mut(),
            area,
            animation_frame,
            is_active,
            is_running,
        );
    }
}
