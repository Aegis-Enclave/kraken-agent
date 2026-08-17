use crate::state::config::ThemeColors;
use crate::ui::chat::ChatComponent;
use crate::ui::editor::Editor;
use crate::ui::file_tree::FileTree;
use ratatui::{layout::Rect, Frame};

pub struct IdeState {
    pub file_tree: FileTree,
    pub editor: Editor,
    pub focus_tree: bool, // true for tree, false for editor
}

impl Default for IdeState {
    fn default() -> Self {
        Self {
            file_tree: FileTree::new(),
            editor: Editor::new(),
            focus_tree: true,
        }
    }
}

pub struct IdeView;

impl IdeView {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        _colors: &ThemeColors,
        chat_component: &ChatComponent,
        chat_state: &mut crate::ui::chat::ChatState,
        ide_state: &mut IdeState,
        connected: bool,
        card_manager: &crate::ui::cards::CardManager,
        subagent_list: &crate::ui::subagent::SubagentList,
        animation_frame: u64,
        is_running: bool,
    ) {
        use ratatui::layout::{Constraint, Direction, Layout};
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::text::Span;
        use ratatui::widgets::{Block, BorderType, Borders};

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(30),     // Tree
                Constraint::Percentage(40), // Editor
                Constraint::Min(40),        // Chat
            ])
            .spacing(1)
            .split(area);

        // 1. File Tree
        ide_state.file_tree.render(
            frame,
            chunks[0],
            ide_state.focus_tree,
            animation_frame,
            is_running,
        );

        // 2. Editor
        ide_state.editor.render(
            frame,
            chunks[1],
            !ide_state.focus_tree,
            animation_frame,
            is_running,
        );

        // 3. Chat
        let chat_block = Block::default()
            .title(Span::styled(
                " CHAT ",
                Style::default()
                    .fg(Color::Rgb(211, 134, 155))
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::DarkGray));

        let chat_inner = chat_block.inner(chunks[2]);
        frame.render_widget(chat_block, chunks[2]);
        crate::ui::borders::render_gradient_border(
            frame.buffer_mut(),
            chunks[2],
            animation_frame,
            false,
            is_running,
        );

        chat_state.visible_height = chat_inner.height.saturating_sub(2);
        chat_component.set_show_logo_on_empty(false);
        chat_component.render(
            frame,
            chat_inner,
            chat_state,
            card_manager,
            subagent_list,
            connected,
            animation_frame,
        );
    }
}
