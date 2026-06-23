use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};

pub struct KanbanView;

impl KanbanView {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        colors: &crate::state::config::ThemeColors,
        animation_frame: u64,
        is_running: bool,
    ) {
        use crate::ui::borders::render_gradient_border;

        let main_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(colors.primary.clone().into()))
            .padding(Padding::new(1, 1, 1, 1));
        let inner_area = main_block.inner(area);
        frame.render_widget(main_block, area);
        render_gradient_border(
            frame.buffer_mut(),
            area,
            animation_frame,
            true,
            is_running,
        );

        // Column layout
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .spacing(1)
            .split(inner_area);

        let column_titles = ["Backlog", "In Progress", "Done"];
        let column_colors = [
            Color::Rgb(250, 189, 47),
            Color::Rgb(131, 165, 152),
            Color::Rgb(166, 226, 46),
        ];
        let items = [
            vec!["Task planning", "Research", "Spec review"],
            vec!["Implement parser", "Write tests"],
            vec!["Setup CI", "Documentation"],
        ];

        for (i, chunk) in cols.iter().enumerate() {
            let col_block = Block::default()
                .title(Span::styled(
                    format!(" {} ", column_titles[i]),
                    Style::default()
                        .fg(column_colors[i])
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(column_colors[i]))
                .padding(Padding::new(1, 1, 1, 1));
            let col_inner = col_block.inner(*chunk);
            frame.render_widget(col_block, *chunk);

            let mut lines = Vec::new();
            for card in &items[i] {
                let card_style = Style::default()
                    .fg(Color::Rgb(235, 219, 178))
                    .bg(Color::Rgb(40, 40, 40));
                lines.push(Line::from(Span::styled(
                    format!(" • {card}"),
                    card_style,
                )));
            }
            if lines.is_empty() {
                lines.push(Line::from(Span::styled(
                    " (empty) ",
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                )));
            }
            frame.render_widget(Paragraph::new(lines), col_inner);
        }
    }
}
