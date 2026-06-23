use crate::state::config::ThemeColors;
use ratatui::{layout::Rect, Frame};

pub struct DashboardView;

impl DashboardView {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        colors: &ThemeColors,
        animation_frame: u64,
        is_running: bool,
        cpu_usage: f32,
        memory_usage: f32,
        cpu_history: &[u64],
        memory_history: &[u64],
        token_speed_history: &[u64],
    ) {
        // TODO: This is a prototype layout matching the React example.
        // The telemetry values are currently hardcoded or derived from time.
        // Future work: Connect this to real system metrics (CPU/MEM/NET).

        use ratatui::layout::{Alignment, Constraint, Direction, Layout};
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, BorderType, Borders, Gauge, Padding, Paragraph, Sparkline};

        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as usize;
        let offset = (time / 100) % 6;

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // Title/Banner
                Constraint::Min(20),   // Content
            ])
            .split(area);

        // Title
        // ── Gruvbox vibrant palette ──
        const GRUVBOX_COLORS: &[Color] = &[
            Color::Rgb(251, 73, 52),   // bright_red    #fb4934
            Color::Rgb(184, 187, 38),  // bright_green  #b8bb26
            Color::Rgb(250, 189, 47),  // bright_yellow #fabd2f
            Color::Rgb(131, 165, 152), // bright_blue   #83a598
            Color::Rgb(211, 134, 155), // bright_purple #d3869b
            Color::Rgb(142, 192, 124), // bright_aqua   #8ec07c
            Color::Rgb(254, 128, 25),  // bright_orange #fe8019
            Color::Rgb(104, 157, 106), // aqua          #689d6a
        ];
        const NUM_COLORS: usize = GRUVBOX_COLORS.len();

        let title_text = r"██   ██ ██████  █████  ██   ██ ███████ ███    ██     █████  ██████  ███████ ███    ██ ████████
██  ██  ██   ██ ██   ██ ██  ██  ██      ████   ██    ██   ██ ██       ██      ████   ██    ██   
█████   ██████  ███████ █████   █████   ██ ██  ██    ███████ ██   ███ █████   ██ ██  ██    ██   
██  ██  ██   ██ ██   ██ ██  ██  ██      ██  ██ ██    ██   ██ ██    ██ ██      ██  ██ ██    ██   
██   ██ ██   ██ ██   ██ ██   ██ ███████ ██   ████    ██   ██  ██████  ███████ ██   ████    ██";

        let anim = animation_frame as usize;
        let mut title_lines = Vec::new();
        for (i, line) in title_text.lines().enumerate() {
            let c = GRUVBOX_COLORS[(i + anim) % NUM_COLORS];
            title_lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(c).add_modifier(Modifier::BOLD),
            )));
        }
        let title_para = Paragraph::new(title_lines)
            .alignment(Alignment::Center)
            .block(Block::default().padding(Padding::new(0, 0, 1, 0)));
        frame.render_widget(title_para, chunks[0]);

        // Content border
        let content_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(colors.primary.clone().into()))
            .padding(Padding::new(2, 2, 1, 1));
        let inner_area = content_block.inner(chunks[1]);
        frame.render_widget(content_block, chunks[1]);
        crate::ui::borders::render_gradient_border(
            frame.buffer_mut(),
            chunks[1],
            animation_frame,
            true,
            is_running,
        );

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Left column (GIF/Telemetry)
                Constraint::Percentage(70), // Right column (Lists)
            ])
            .spacing(2)
            .split(inner_area);

        // Left Column
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // Spacer (was GIF area)
                Constraint::Min(20),    // Telemetry
            ])
            .spacing(1)
            .split(content_chunks[0]);

        // Telemetry
        let cpu = cpu_usage as u16;
        let mem = memory_usage as u16;
        let net = 200 + (offset * 100);

        let tel_block = Block::default()
            .title(Span::styled(
                " TELEMETRY ",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::new(1, 1, 1, 1));

        let tel_inner = tel_block.inner(left_chunks[1]);
        frame.render_widget(tel_block, left_chunks[1]);

        let tel_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // CPU Gauge
                Constraint::Length(3), // CPU Sparkline
                Constraint::Length(2), // MEM Gauge
                Constraint::Length(3), // MEM Sparkline
                Constraint::Length(2), // NET
                Constraint::Length(4), // TOKEN SPEED Sparkline
                Constraint::Min(0),
            ])
            .spacing(0)
            .split(tel_inner);

        // CPU Gauge
        let cpu_gauge = Gauge::default()
            .block(Block::default().title("CPU Usage"))
            .gauge_style(
                Style::default()
                    .fg(Color::Rgb(131, 165, 152))
                    .bg(Color::Rgb(40, 40, 40)),
            )
            .percent(cpu);
        frame.render_widget(cpu_gauge, tel_layout[0]);

        // CPU Sparkline (Dynamic Bins)
        let cpu_bins = tel_layout[1].width as usize;
        let cpu_data: Vec<u64> = cpu_history
            .iter()
            .rev()
            .take(cpu_bins)
            .copied()
            .collect::<Vec<u64>>()
            .into_iter()
            .rev()
            .collect();
        let cpu_spark = Sparkline::default()
            .data(&cpu_data)
            .style(Style::default().fg(Color::Rgb(131, 165, 152)));
        frame.render_widget(cpu_spark, tel_layout[1]);

        // MEM Gauge
        let mem_gauge = Gauge::default()
            .block(Block::default().title("Memory"))
            .gauge_style(
                Style::default()
                    .fg(Color::Rgb(211, 134, 155))
                    .bg(Color::Rgb(40, 40, 40)),
            )
            .percent(mem);
        frame.render_widget(mem_gauge, tel_layout[2]);

        // MEM Sparkline (Dynamic Bins)
        let mem_bins = tel_layout[3].width as usize;
        let mem_data: Vec<u64> = memory_history
            .iter()
            .rev()
            .take(mem_bins)
            .copied()
            .collect::<Vec<u64>>()
            .into_iter()
            .rev()
            .collect();
        let mem_spark = Sparkline::default()
            .data(&mem_data)
            .style(Style::default().fg(Color::Rgb(211, 134, 155)));
        frame.render_widget(mem_spark, tel_layout[3]);

        // NET Info
        let is_streaming = offset % 2 == 0;
        let net_text = Line::from(vec![
            Span::styled("NET ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if is_streaming {
                    ">> STREAMING "
                } else {
                    "<> IDLE      "
                },
                Style::default().fg(Color::Rgb(250, 189, 47)),
            ),
            Span::styled(format!("{net}kb/s"), Style::default().fg(Color::Gray)),
        ]);
        frame.render_widget(Paragraph::new(net_text), tel_layout[4]);

        // TOKEN SPEED Sparkline (Dynamic Bins)
        let token_bins = tel_layout[5].width as usize;
        let token_data: Vec<u64> = token_speed_history
            .iter()
            .rev()
            .take(token_bins)
            .copied()
            .collect::<Vec<u64>>()
            .into_iter()
            .rev()
            .collect();
        let token_spark = Sparkline::default()
            .block(
                Block::default().title(Span::styled(
                    "TOKEN SPEED (tokens/s)",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )),
            )
            .data(&token_data)
            .max(100)
            .style(Style::default().fg(Color::Rgb(250, 189, 47)));
        frame.render_widget(token_spark, tel_layout[5]);

        // Right Column
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Available Tools
                Constraint::Length(4), // MCP Servers
                Constraint::Min(10),   // Available Skills
            ])
            .spacing(1)
            .split(content_chunks[1]);

        // Available Tools
        let mut tools_lines = Vec::new();
        tools_lines.push(Line::from(vec![
            Span::styled("browser:       ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "browser_back, browser_click, browser_close, browser_open",
                Style::default().fg(Color::Gray),
            ),
        ]));
        tools_lines.push(Line::from(vec![
            Span::styled("browser-cdp:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "browser_cdp_call, browser_dialog_accept, browser_dialog_dismiss",
                Style::default().fg(Color::Gray),
            ),
        ]));
        tools_lines.push(Line::from(vec![
            Span::styled("clarify:       ", Style::default().fg(Color::DarkGray)),
            Span::styled("clarify", Style::default().fg(Color::Rgb(211, 134, 155))),
        ]));
        tools_lines.push(Line::from(vec![
            Span::styled("code_execution:", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "execute_code",
                Style::default().fg(Color::Rgb(250, 189, 47)),
            ),
        ]));
        tools_lines.push(Line::from(Span::styled(
            "(and 22 more toolsets...)",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )));
        frame.render_widget(
            Paragraph::new(tools_lines).block(
                Block::default()
                    .title(Span::styled(
                        " Available Tools ",
                        Style::default()
                            .fg(Color::Rgb(250, 189, 47))
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::DarkGray)),
            ),
            right_chunks[0],
        );

        // MCP Servers
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"][offset % 10];
        let mcp_lines = vec![Line::from(vec![
            Span::styled(
                "playwright (stdio) — ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("connecting {spinner}"),
                Style::default().fg(Color::Rgb(250, 189, 47)),
            ),
        ])];
        frame.render_widget(
            Paragraph::new(mcp_lines).block(
                Block::default()
                    .title(Span::styled(
                        " MCP Servers ",
                        Style::default()
                            .fg(Color::Rgb(250, 189, 47))
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::DarkGray)),
            ),
            right_chunks[1],
        );

        // Available Skills
        let mut skills_lines = Vec::new();
        let skills = [
            ("autonomous-ai-agents", "coding-agents, hermes-agent..."),
            ("creative", "architecture-diagram, ascii-art..."),
            ("data-science", "jupyter-live-kernel"),
            ("devops", "kanban-orchestrator, kanban-worker..."),
            ("email", "himalaya"),
            ("fullstack-webdev", "React Patterns, Tailwind CSS..."),
        ];
        for (cat, tools) in skills {
            skills_lines.push(Line::from(vec![
                Span::styled(format!("{cat:25}"), Style::default().fg(Color::DarkGray)),
                Span::styled(tools, Style::default().fg(Color::Gray)),
            ]));
        }
        frame.render_widget(
            Paragraph::new(skills_lines).block(
                Block::default().title(Span::styled(
                    " Available Skills ",
                    Style::default()
                        .fg(Color::Rgb(250, 189, 47))
                        .add_modifier(Modifier::BOLD),
                )),
            ),
            right_chunks[2],
        );
    }
}
