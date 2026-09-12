//! Markdown rendering utilities for chat messages.
//!
//! Converts a subset of Markdown (headers, emphasis, lists, code, links) into
//! ratatui `Line`/`Span` structures so messages look good in the terminal.

use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
};

use crate::state::config::ChatColorsRgb;
use crate::utils::ansi::AnsiParser;

/// Render markdown content into ratatui lines.
///
/// Code blocks are returned as a single fenced block marker so the caller can
/// route them to the syntax highlighter. Inline code is styled inline.
#[must_use]
pub fn render_markdown<'a>(content: &'a str, colors: &ChatColorsRgb, width: u16) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let parser = Parser::new(content);

    let mut current_line = Vec::new();
    let mut current_style = base_style(colors);
    let mut in_code_block = false;
    let mut code_buffer = String::new();
    let mut code_lang = String::new();
    let mut list_stack: Vec<u64> = Vec::new();
    let mut pending_list_prefix = None::<String>;
    let mut ansi_parser = AnsiParser::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    // Paragraphs are implicit; nothing to do.
                }
                Tag::Heading { level, .. } => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_line)));
                    }
                    let prefix = "#".repeat(level as usize);
                    current_line.push(Span::styled(
                        format!("{prefix} "),
                        Style::new().fg(colors.border).add_modifier(Modifier::BOLD),
                    ));
                    current_style = base_style(colors)
                        .add_modifier(Modifier::BOLD)
                        .fg(heading_color(colors, level));
                }
                Tag::Strong => {
                    current_style = current_style.add_modifier(Modifier::BOLD);
                }
                Tag::Emphasis => {
                    current_style = current_style.add_modifier(Modifier::ITALIC);
                }
                Tag::Strikethrough => {
                    current_style = current_style.add_modifier(Modifier::CROSSED_OUT);
                }
                Tag::CodeBlock(lang) => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_line)));
                    }
                    in_code_block = true;
                    code_lang = match lang {
                        pulldown_cmark::CodeBlockKind::Fenced(lang_str) => lang_str.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                    code_buffer.clear();
                }
                Tag::List(start) => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_line)));
                    }
                    list_stack.push(start.unwrap_or(0));
                }
                Tag::Item => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_line)));
                    }
                    let indent = list_stack.len().saturating_sub(1) * 2;
                    let prefix = format!("{}• ", " ".repeat(indent));
                    pending_list_prefix = Some(prefix);
                }
                Tag::BlockQuote(_) => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_line)));
                    }
                    current_line.push(Span::styled("│ ", Style::new().fg(colors.border).dim()));
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    current_style = base_style(colors);
                    if !current_line.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_line)));
                    }
                }
                TagEnd::Strong | TagEnd::Emphasis | TagEnd::Strikethrough => {
                    current_style = base_style(colors);
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    if !code_buffer.is_empty() {
                        lines.push(Line::from(Span::styled(
                            format!("```{code_lang}"),
                            Style::new().fg(colors.timestamp),
                        )));
                        for line in code_buffer.lines() {
                            lines.push(Line::from(Span::styled(
                                line.to_string(),
                                Style::new().fg(colors.code_text).bg(colors.code_bg),
                            )));
                        }
                        lines.push(Line::from(Span::styled(
                            "```",
                            Style::new().fg(colors.timestamp),
                        )));
                    }
                    code_buffer.clear();
                    code_lang.clear();
                }
                TagEnd::Paragraph | TagEnd::BlockQuote(_) => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_line)));
                    }
                }
                TagEnd::List(_) => {
                    list_stack.pop();
                    if !current_line.is_empty() {
                        lines.push(Line::from(std::mem::take(&mut current_line)));
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    code_buffer.push_str(&text);
                } else {
                    if let Some(prefix) = pending_list_prefix.take() {
                        current_line.push(Span::styled(prefix, base_style(colors)));
                    }

                    // Parse ANSI codes in the text
                    let parsed = ansi_parser.parse(&text);
                    for (segment, ansi_style) in parsed {
                        let mut final_style = current_style;

                        // Merge ANSI style with markdown style
                        let r_style = ansi_style.to_ratatui_style();
                        if let Some(fg) = r_style.fg {
                            final_style = final_style.fg(fg);
                        }
                        if let Some(bg) = r_style.bg {
                            final_style = final_style.bg(bg);
                        }
                        final_style.add_modifier |= r_style.add_modifier;
                        final_style.sub_modifier |= r_style.sub_modifier;

                        current_line.push(Span::styled(segment, final_style));
                    }
                }
            }
            Event::Code(code) => {
                current_line.push(Span::styled(
                    code.to_string(),
                    Style::new().fg(colors.code_text).bg(colors.code_bg),
                ));
            }
            Event::Html(html) => {
                current_line.push(Span::styled(html.to_string(), base_style(colors)));
            }
            Event::SoftBreak | Event::HardBreak => {
                if !current_line.is_empty() {
                    lines.push(Line::from(std::mem::take(&mut current_line)));
                }
            }
            Event::Rule => {
                if !current_line.is_empty() {
                    lines.push(Line::from(std::mem::take(&mut current_line)));
                }
                let rule = "─".repeat(width as usize);
                lines.push(Line::from(Span::styled(
                    rule,
                    Style::new().fg(colors.border).dim(),
                )));
            }
            _ => {}
        }
    }

    if !current_line.is_empty() {
        lines.push(Line::from(std::mem::take(&mut current_line)));
    }

    // Strip trailing empty lines for cleanliness.
    while lines
        .last()
        .is_some_and(|l| l.to_string().trim().is_empty())
    {
        lines.pop();
    }

    lines
}

fn base_style(colors: &ChatColorsRgb) -> Style {
    Style::new().fg(colors.assistant_text)
}

fn heading_color(colors: &ChatColorsRgb, level: pulldown_cmark::HeadingLevel) -> Color {
    match level {
        pulldown_cmark::HeadingLevel::H1 => colors.border,
        pulldown_cmark::HeadingLevel::H2 => colors.border,
        pulldown_cmark::HeadingLevel::H3 => colors.timestamp,
        _ => colors.timestamp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_colors() -> ChatColorsRgb {
        ChatColorsRgb::default()
    }

    #[test]
    fn test_render_bold() {
        let colors = test_colors();
        let lines = render_markdown("**hello** world", &colors, 80);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].spans.len(), 2);
    }

    #[test]
    fn test_render_heading() {
        let colors = test_colors();
        let lines = render_markdown("# Title\nbody", &colors, 80);
        assert!(lines.len() >= 2);
        assert!(lines[0].to_string().contains("Title"));
    }

    #[test]
    fn test_render_code_block() {
        let colors = test_colors();
        let lines = render_markdown("```rust\nlet x = 1;\n```", &colors, 80);
        assert!(lines.iter().any(|l| l.to_string().contains("let x")));
    }
}
