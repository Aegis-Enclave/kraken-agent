use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Padding},
    Frame,
};
use std::fs;
use std::path::PathBuf;
use tui_textarea::TextArea;

pub struct Editor {
    pub textarea: TextArea<'static>,
    pub current_file: Option<PathBuf>,
}

impl Editor {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(Color::DarkGray))
                .padding(Padding::new(1, 1, 1, 1)),
        );

        Self {
            textarea,
            current_file: None,
        }
    }

    pub fn load_file(&mut self, path: PathBuf) {
        if let Ok(content) = fs::read_to_string(&path) {
            let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            self.textarea = TextArea::new(lines);
            self.current_file = Some(path);
        }
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        is_active: bool,
        animation_frame: u64,
        is_running: bool,
    ) {
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

        let file_name = self
            .current_file
            .as_ref()
            .map(|p| {
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_else(|| "No file open".to_string());

        let title = format!(" 📝 {} ", file_name);

        let block = Block::default()
            .title(Span::styled(
                title,
                Style::default().fg(Color::Rgb(250, 189, 47)),
            ))
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style)
            .padding(Padding::new(1, 1, 1, 1));

        self.textarea.set_block(block);

        if is_active {
            self.textarea
                .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
            self.textarea
                .set_cursor_line_style(Style::default().bg(Color::Rgb(60, 56, 54)));
        } else {
            self.textarea.set_cursor_style(Style::default());
            self.textarea.set_cursor_line_style(Style::default());
        }

        frame.render_widget(&self.textarea, area);
        crate::ui::borders::render_gradient_border(
            frame.buffer_mut(),
            area,
            animation_frame,
            is_active,
            is_running,
        );
    }
}
