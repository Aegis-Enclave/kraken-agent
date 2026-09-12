use crate::state::config::ThemeColors;
use ratatui::{layout::Rect, Frame};

/// Format a byte-per-second value into a human-readable string.
fn format_speed(bytes_per_sec: f32) -> String {
    if bytes_per_sec >= 1_048_576.0 {
        format!("{:.1} MB/s", bytes_per_sec / 1_048_576.0)
    } else if bytes_per_sec >= 1024.0 {
        format!("{:.0} KB/s", bytes_per_sec / 1024.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

pub struct DashboardView;

impl DashboardView {
    #[allow(clippy::too_many_arguments)]
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
        net_rx_speed: f32,
        net_tx_speed: f32,
        _net_rx_history: &[u64],
        _net_tx_history: &[u64],
    ) {
        use ratatui::layout::{Alignment, Constraint, Direction, Layout};
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, BorderType, Borders, Gauge, Padding, Paragraph, Sparkline};

        // ── Gruvbox vibrant palette ──
        const GRUVBOX_RED: Color = Color::Rgb(251, 73, 52);
        const GRUVBOX_GREEN: Color = Color::Rgb(184, 187, 38);
        const GRUVBOX_YELLOW: Color = Color::Rgb(250, 189, 47);
        const GRUVBOX_BLUE: Color = Color::Rgb(131, 165, 152);
        const GRUVBOX_PURPLE: Color = Color::Rgb(211, 134, 155);
        const GRUVBOX_AQUA: Color = Color::Rgb(142, 192, 124);
        const GRUVBOX_ORANGE: Color = Color::Rgb(254, 128, 25);
        const GRUVBOX_BG: Color = Color::Rgb(40, 40, 40);
        const GRUVBOX_FG: Color = Color::Rgb(235, 219, 178);
        const GRUVBOX_DIM: Color = Color::Rgb(146, 131, 116);

        let anim = animation_frame as usize;

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // Title/Banner
                Constraint::Min(20),   // Content
            ])
            .split(area);

        // ── Animated KRAKEN ASCII title ──
        const TITLE_COLORS: &[Color] = &[
            GRUVBOX_RED, GRUVBOX_GREEN, GRUVBOX_YELLOW, GRUVBOX_BLUE,
            GRUVBOX_PURPLE, GRUVBOX_AQUA, GRUVBOX_ORANGE,
        ];
        const NUM_COLORS: usize = TITLE_COLORS.len();

        let title_text = r"██   ██ ██████  █████  ██   ██ ███████ ███    ██     █████  ██████  ███████ ███    ██ ████████
██  ██  ██   ██ ██   ██ ██  ██  ██      ████   ██    ██   ██ ██       ██      ████   ██    ██
█████   ██████  ███████ █████   █████   ██ ██  ██    ███████ ██   ███ █████   ██ ██  ██    ██
██  ██  ██   ██ ██   ██ ██  ██  ██      ██  ██ ██    ██   ██ ██    ██ ██      ██  ██ ██    ██
██   ██ ██   ██ ██   ██ ██   ██ ███████ ██   ████    ██   ██  ██████  ███████ ██   ████    ██";

        let mut title_lines = Vec::new();
        for (i, line) in title_text.lines().enumerate() {
            let c = TITLE_COLORS[(i + anim) % NUM_COLORS];
            title_lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(c).add_modifier(Modifier::BOLD),
            )));
        }
        let title_para = Paragraph::new(title_lines)
            .alignment(Alignment::Center)
            .block(Block::default().padding(Padding::new(0, 0, 1, 0)));
        frame.render_widget(title_para, chunks[0]);

        // ── Content border ──
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
                Constraint::Percentage(30), // Left column (Telemetry)
                Constraint::Percentage(70), // Right column (Lists)
            ])
            .spacing(2)
            .split(inner_area);

        // ── Left Column ──
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // Spacer (was GIF area)
                Constraint::Min(20),    // Telemetry
            ])
            .spacing(1)
            .split(content_chunks[0]);

        // ── Telemetry Block ──
        let tel_block = Block::default()
            .title(Span::styled(
                " TELEMETRY ",
                Style::default()
                    .fg(GRUVBOX_AQUA)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(GRUVBOX_DIM))
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
                Constraint::Length(2), // NET RX Gauge
                Constraint::Length(2), // NET TX Gauge
                Constraint::Length(3), // TOKEN SPEED Sparkline
                Constraint::Min(0),
            ])
            .spacing(0)
            .split(tel_inner);

        let cpu = cpu_usage as u16;
        let mem = memory_usage as u16;

        // ── CPU Gauge ──
        let cpu_gauge = Gauge::default()
            .block(Block::default().title(Span::styled(
                format!("CPU Usage: {cpu}%"),
                Style::default().fg(GRUVBOX_BLUE).add_modifier(Modifier::BOLD),
            )))
            .gauge_style(
                Style::default()
                    .fg(GRUVBOX_BLUE)
                    .bg(GRUVBOX_BG),
            )
            .percent(cpu);
        frame.render_widget(cpu_gauge, tel_layout[0]);

        // ── CPU Sparkline ──
        let cpu_bins = tel_layout[1].width as usize;
        let cpu_spark_data: Vec<u64> = cpu_history
            .iter()
            .rev()
            .take(cpu_bins)
            .copied()
            .collect::<Vec<u64>>()
            .into_iter()
            .rev()
            .collect();
        let cpu_spark = Sparkline::default()
            .block(Block::default().title(Span::styled(
                " CPU History ",
                Style::default()
                    .fg(GRUVBOX_DIM)
                    .add_modifier(Modifier::BOLD),
            )))
            .data(&cpu_spark_data)
            .style(Style::default().fg(GRUVBOX_BLUE));
        frame.render_widget(cpu_spark, tel_layout[1]);

        // ── MEM Gauge ──
        let mem_gauge = Gauge::default()
            .block(Block::default().title(Span::styled(
                format!("Memory: {mem}%"),
                Style::default().fg(GRUVBOX_PURPLE).add_modifier(Modifier::BOLD),
            )))
            .gauge_style(
                Style::default()
                    .fg(GRUVBOX_PURPLE)
                    .bg(GRUVBOX_BG),
            )
            .percent(mem);
        frame.render_widget(mem_gauge, tel_layout[2]);

        // ── MEM Sparkline ──
        let mem_bins = tel_layout[3].width as usize;
        let mem_spark_data: Vec<u64> = memory_history
            .iter()
            .rev()
            .take(mem_bins)
            .copied()
            .collect::<Vec<u64>>()
            .into_iter()
            .rev()
            .collect();
        let mem_spark = Sparkline::default()
            .block(Block::default().title(Span::styled(
                " Memory History ",
                Style::default()
                    .fg(GRUVBOX_DIM)
                    .add_modifier(Modifier::BOLD),
            )))
            .data(&mem_spark_data)
            .style(Style::default().fg(GRUVBOX_PURPLE));
        frame.render_widget(mem_spark, tel_layout[3]);

        // ── NET Receive Gauge ──
        // Scale gauge to 1 MB/s max so it shows useful movement
        const NET_MAX_BYTES: f32 = 1_048_576.0; // 1 MB/s
        let net_rx_pct = ((net_rx_speed / NET_MAX_BYTES) * 100.0).min(100.0) as u16;
        let net_rx_gauge = Gauge::default()
            .block(Block::default().title(Span::styled(
                format!("Receive: {}", format_speed(net_rx_speed)),
                Style::default().fg(GRUVBOX_AQUA).add_modifier(Modifier::BOLD),
            )))
            .gauge_style(
                Style::default()
                    .fg(GRUVBOX_AQUA)
                    .bg(GRUVBOX_BG),
            )
            .percent(net_rx_pct);
        frame.render_widget(net_rx_gauge, tel_layout[4]);

        // ── NET Transmit Gauge ──
        let net_tx_pct = ((net_tx_speed / NET_MAX_BYTES) * 100.0).min(100.0) as u16;
        let net_tx_gauge = Gauge::default()
            .block(Block::default().title(Span::styled(
                format!("Transmit: {}", format_speed(net_tx_speed)),
                Style::default().fg(GRUVBOX_YELLOW).add_modifier(Modifier::BOLD),
            )))
            .gauge_style(
                Style::default()
                    .fg(GRUVBOX_YELLOW)
                    .bg(GRUVBOX_BG),
            )
            .percent(net_tx_pct);
        frame.render_widget(net_tx_gauge, tel_layout[5]);

        // ── TOKEN SPEED Sparkline ──
        let token_bins = tel_layout[6].width as usize;
        let token_data: Vec<u64> = token_speed_history
            .iter()
            .rev()
            .take(token_bins)
            .copied()
            .collect::<Vec<u64>>()
            .into_iter()
            .rev()
            .collect();
        let token_max = *token_data.iter().max().unwrap_or(&1).max(&1);
        let token_spark = Sparkline::default()
            .block(Block::default().title(Span::styled(
                " TOKEN SPEED (tokens/s) ",
                Style::default()
                    .fg(GRUVBOX_DIM)
                    .add_modifier(Modifier::BOLD),
            )))
            .data(&token_data)
            .max(token_max)
            .style(Style::default().fg(GRUVBOX_YELLOW));
        frame.render_widget(token_spark, tel_layout[6]);

        // ── Right Column ──
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Available Tools
                Constraint::Length(4), // MCP Servers
                Constraint::Min(10),   // Available Skills
            ])
            .spacing(1)
            .split(content_chunks[1]);

        // ── Available Tools ──
        let mut tools_lines = Vec::new();
        let tools_data = [
            ("browser:       ", "browser_back, browser_click, browser_close, browser_open"),
            ("browser-cdp:   ", "browser_cdp_call, browser_dialog_accept, browser_dialog_dismiss"),
            ("clarify:       ", "clarify"),
            ("code_execution:", "execute_code"),
        ];
        let tools_colors = [
            (GRUVBOX_FG, GRUVBOX_DIM),
            (GRUVBOX_FG, GRUVBOX_DIM),
            (GRUVBOX_PURPLE, GRUVBOX_DIM),
            (GRUVBOX_YELLOW, GRUVBOX_DIM),
        ];
        for (i, (label, value)) in tools_data.iter().enumerate() {
            tools_lines.push(Line::from(vec![
                Span::styled(*label, Style::default().fg(GRUVBOX_DIM)),
                Span::styled(*value, Style::default().fg(tools_colors[i].0)),
            ]));
        }
        tools_lines.push(Line::from(Span::styled(
            "(and 22 more toolsets...)",
            Style::default()
                .fg(GRUVBOX_DIM)
                .add_modifier(Modifier::ITALIC),
        )));
        frame.render_widget(
            Paragraph::new(tools_lines).block(
                Block::default()
                    .title(Span::styled(
                        " Available Tools ",
                        Style::default()
                            .fg(GRUVBOX_YELLOW)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(GRUVBOX_DIM)),
            ),
            right_chunks[0],
        );

        // ── MCP Servers ──
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"][anim % 10];
        let mcp_lines = vec![Line::from(vec![
            Span::styled(
                "playwright (stdio) — ",
                Style::default().fg(GRUVBOX_DIM),
            ),
            Span::styled(
                format!("connecting {spinner}"),
                Style::default().fg(GRUVBOX_YELLOW),
            ),
        ])];
        frame.render_widget(
            Paragraph::new(mcp_lines).block(
                Block::default()
                    .title(Span::styled(
                        " MCP Servers ",
                        Style::default()
                            .fg(GRUVBOX_YELLOW)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(GRUVBOX_DIM)),
            ),
            right_chunks[1],
        );

        // ── Available Skills ──
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
                Span::styled(format!("{cat:25}"), Style::default().fg(GRUVBOX_DIM)),
                Span::styled(tools, Style::default().fg(GRUVBOX_FG)),
            ]));
        }
        frame.render_widget(
            Paragraph::new(skills_lines).block(
                Block::default().title(Span::styled(
                    " Available Skills ",
                    Style::default()
                        .fg(GRUVBOX_YELLOW)
                        .add_modifier(Modifier::BOLD),
                )),
            ),
            right_chunks[2],
        );
    }
}
