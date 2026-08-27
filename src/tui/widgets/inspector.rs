//! Right-side details pane.
//!
//! Mirrors `ui/widgets/inspector.py` from the Python version:
//! - Title bar
//! - Human-readable path
//! - Details grid (Type / Size / Format / Time)
//! - Smart Insights panel (URL → blue underlined; hex color → 6-cell
//!   background preview; ISO8601 → parsed via chrono)
//! - Content Preview (first 30 children with +/- icons)
//! - Source panel: depth-limited reconstruction serialized as JSON
//!   or YAML, rendered with a tiny manual syntax highlighter

use chrono::DateTime;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;
use regex::Regex;

use crate::core::model::{DataType, Node};
use crate::core::store::Store;
use crate::tui::theme::Theme;

pub fn render(
    f: &mut Frame,
    area: Rect,
    theme: &Theme,
    node: Option<&Node>,
    store: Option<&Store>,
    format: &str,
) {
    let block = Block::default()
        .title(" Inspector ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.warning));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if node.is_none() {
        f.render_widget(
            Paragraph::new(Line::from("Nothing selected.")).style(theme.base_style()),
            inner,
        );
        return;
    }

    let node = node.unwrap();
    let store = store.unwrap();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // path
            Constraint::Length(5), // details grid
            Constraint::Length(4), // insights
            Constraint::Min(3),    // preview
            Constraint::Min(3),    // source
        ])
        .split(inner);

    // Path header (truncate to column width so it doesn't bleed).
    let chain = build_path_chain(node, store);
    let path_text = chain.join(" › ");
    let path_text = truncate_path(&path_text, inner.width as usize);
    f.render_widget(
        Paragraph::new(Line::from(path_text))
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::DIM)),
        rows[0],
    );

    // Details grid.
    let mut details_lines: Vec<Line> = Vec::new();
    details_lines.push(detail_line("Type", capitalize(node.ty.as_str()), theme));
    if node.is_container() {
        let count = store.get_children_count(node.id).unwrap_or(0);
        details_lines.push(detail_line("Size", format!("{count} items"), theme));
    } else {
        let len = node
            .value
            .as_ref()
            .map(|v| v.to_string().len())
            .unwrap_or(0);
        details_lines.push(detail_line("Size", format!("{len} chars"), theme));
    }
    if node.ty == DataType::String {
        if let Some(s) = node.value.as_ref().and_then(|v| v.as_str()) {
            if let Some(detail) = detect_string_format(s) {
                details_lines.push(detail_line("Format", detail.label, theme));
                if let Some(extra) = detail.extra {
                    details_lines.push(detail_line("Time", extra, theme));
                }
            }
        }
    }
    let detail_items: Vec<ListItem> = details_lines.into_iter().map(ListItem::new).collect();
    f.render_widget(List::new(detail_items).style(theme.base_style()), rows[1]);

    // Insights (only draw the box if there's content to show).
    let insight_lines = build_insights(node, theme);
    if !insight_lines.is_empty() {
        let insights_block = Block::default()
            .title(" Smart Insights ")
            .borders(Borders::ALL)
            .border_style(Style::default().add_modifier(Modifier::DIM));
        let insights_inner = insights_block.inner(rows[2]);
        f.render_widget(insights_block, rows[2]);
        f.render_widget(
            Paragraph::new(insight_lines).wrap(Wrap { trim: false }),
            insights_inner,
        );
    }

    // Content Preview.
    let preview_block = Block::default()
        .title(" Preview ")
        .borders(Borders::ALL)
        .border_style(Style::default().add_modifier(Modifier::DIM));
    let preview_inner = preview_block.inner(rows[3]);
    f.render_widget(preview_block, rows[3]);
    let preview_lines = build_preview(node, store);
    f.render_widget(
        Paragraph::new(preview_lines).wrap(Wrap { trim: false }),
        preview_inner,
    );

    // Source.
    let source_block = Block::default()
        .title(" Source ")
        .borders(Borders::ALL)
        .border_style(Style::default().add_modifier(Modifier::DIM));
    let source_inner = source_block.inner(rows[4]);
    f.render_widget(source_block, rows[4]);
    let source_lines = build_source(node, store, format);
    f.render_widget(
        Paragraph::new(source_lines).wrap(Wrap { trim: false }),
        source_inner,
    );
}

fn truncate_path(path: &str, max_cols: usize) -> String {
    if max_cols == 0 || path.chars().count() <= max_cols {
        return path.to_string();
    }
    if max_cols <= 4 {
        return "…".to_string();
    }
    let keep = max_cols - 1; // 1 char for the leading ellipsis
    let mut iter = path.chars();
    let tail: String = iter
        .by_ref()
        .rev()
        .take(keep)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("…{tail}")
}

fn detail_line(label: &str, value: impl Into<String>, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label}: "),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(value.into(), theme.base_style()),
    ])
}

fn build_path_chain(node: &Node, store: &Store) -> Vec<String> {
    let mut chain = Vec::new();
    let mut current = node.clone();
    let mut seen = std::collections::HashSet::new();
    seen.insert(current.id);
    loop {
        let label = if current.key.is_empty() {
            "root".to_string()
        } else {
            current.key.clone()
        };
        chain.push(label);
        let Some(parent_id) = current.parent else {
            break;
        };
        if !seen.insert(parent_id) {
            break;
        }
        let Ok(Some(parent)) = store.get_node(parent_id) else {
            break;
        };
        current = parent;
    }
    chain.reverse();
    chain
}

struct FormatDetail {
    label: String,
    extra: Option<String>,
}

fn detect_string_format(s: &str) -> Option<FormatDetail> {
    let url_pat = Regex::new(r"^https?://[^\s]+$").unwrap();
    let hex_pat = Regex::new(r"^#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6})$").unwrap();
    if url_pat.is_match(s) {
        return Some(FormatDetail {
            label: "URL".into(),
            extra: None,
        });
    }
    if hex_pat.is_match(s) {
        return Some(FormatDetail {
            label: "Color".into(),
            extra: None,
        });
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(FormatDetail {
            label: "ISO8601".into(),
            extra: Some(dt.format("%Y-%m-%d %H:%M").to_string()),
        });
    }
    None
}

fn build_insights(node: &Node, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    if node.ty != DataType::String {
        return lines;
    }
    let s = match node.value.as_ref().and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return lines,
    };

    let url_pat = Regex::new(r"^https?://[^\s]+$").unwrap();
    let hex_pat = Regex::new(r"^#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6})$").unwrap();

    if url_pat.is_match(s) {
        lines.push(Line::from(Span::styled(
            "Link: ",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            s.to_string(),
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::UNDERLINED),
        )));
    } else if hex_pat.is_match(s) {
        lines.push(Line::from(vec![
            Span::styled(
                "Color preview: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled("██████", Style::default().bg(parse_hex(s))),
            Span::raw(format!(" {s}")),
        ]));
    } else if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        lines.push(Line::from(format!(
            "ISO8601 → {}",
            dt.format("%Y-%m-%d %H:%M:%S %z")
        )));
    }
    lines
}

fn parse_hex(s: &str) -> ratatui::style::Color {
    let hex = s.trim_start_matches('#');
    if hex.len() == 3 {
        let mut buf = String::with_capacity(6);
        for c in hex.chars() {
            buf.push(c);
            buf.push(c);
        }
        return parse_hex(&format!("#{buf}"));
    }
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
        return ratatui::style::Color::Rgb(r, g, b);
    }
    ratatui::style::Color::White
}

fn build_preview(node: &Node, store: &Store) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    if !node.is_container() {
        let value = node
            .value
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_default();
        lines.push(Line::from(value));
        return lines;
    }
    let children = store.get_children(node.id).unwrap_or_default();
    let limit = 30;
    for child in children.iter().take(limit) {
        let icon = if child.is_container() { "[+]" } else { "-" };
        let val = if child.is_container() {
            "...".to_string()
        } else {
            let v = child
                .value
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default();
            if v.len() > 50 {
                format!("{}...", &v[..47])
            } else {
                v
            }
        };
        lines.push(Line::from(format!("{icon} {}: {val}", child.key)));
    }
    if children.len() > limit {
        lines.push(Line::from(format!(
            "\n... and {} more",
            children.len() - limit
        )));
    }
    lines
}

fn build_source(node: &Node, store: &Store, format: &str) -> Vec<Line<'static>> {
    let cap = 500;
    let children_count = if node.is_container() {
        store.get_children_count(node.id).unwrap_or(0)
    } else {
        0
    };
    if node.is_container() && children_count > cap {
        return vec![Line::from(Span::styled(
            format!("Raw view hidden for performance (> {cap} items)"),
            Style::default().add_modifier(Modifier::ITALIC | Modifier::DIM),
        ))];
    }
    let value = match store.reconstruct_value(node.id, 4) {
        Ok(v) => v,
        Err(_) => {
            return vec![Line::from(Span::styled(
                "Failed to load source.",
                Style::default().fg(theme_fg_error()),
            ))]
        }
    };
    let serialized = if format == "yaml" {
        serde_yml::to_string(&value).unwrap_or_default()
    } else {
        serde_json::to_string_pretty(&value).unwrap_or_default()
    };
    colorize_source(&serialized, format)
}

fn theme_fg_error() -> ratatui::style::Color {
    ratatui::style::Color::Red
}

fn colorize_source(text: &str, format: &str) -> Vec<Line<'static>> {
    if format == "yaml" {
        return text.lines().map(|l| Line::from(l.to_string())).collect();
    }
    let mut lines: Vec<Line<'static>> = Vec::new();
    for line in text.lines() {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '"' => {
                    let mut s = String::from(c);
                    while let Some(&next) = chars.peek() {
                        s.push(chars.next().unwrap());
                        if next == '"' && !s.ends_with("\\\"") {
                            break;
                        }
                    }
                    spans.push(Span::styled(
                        s,
                        Style::default().fg(ratatui::style::Color::Green),
                    ));
                }
                '-' | '0'..='9' => {
                    let mut s = String::from(c);
                    while let Some(&n) = chars.peek() {
                        if n.is_ascii_digit()
                            || n == '.'
                            || n == 'e'
                            || n == 'E'
                            || n == '+'
                            || n == '-'
                        {
                            s.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    spans.push(Span::styled(
                        s,
                        Style::default().fg(ratatui::style::Color::Yellow),
                    ));
                }
                't' | 'f' | 'n' => {
                    let mut s = String::from(c);
                    while let Some(&n) = chars.peek() {
                        if n.is_ascii_alphabetic() {
                            s.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if s == "true" || s == "false" || s == "null" {
                        spans.push(Span::styled(
                            s,
                            Style::default().fg(ratatui::style::Color::Magenta),
                        ));
                    } else {
                        spans.push(Span::raw(s));
                    }
                }
                '{' | '}' | '[' | ']' | ',' | ':' => {
                    spans.push(Span::styled(
                        c.to_string(),
                        Style::default().add_modifier(Modifier::DIM),
                    ));
                }
                other => spans.push(Span::raw(other.to_string())),
            }
        }
        lines.push(Line::from(spans));
    }
    lines
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}
