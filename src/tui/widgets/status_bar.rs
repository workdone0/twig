//! Persistent status bar.

use std::path::Path;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::core::model::{DataType, Node};
use crate::tui::theme::Theme;

pub fn render(
    f: &mut Frame,
    area: Rect,
    theme: &Theme,
    file: &Path,
    node: Option<&Node>,
    search_stats: Option<&str>,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Min(10),
            Constraint::Length(12),
            Constraint::Length(20),
        ])
        .split(area);

    let file_str = format!(" FILE: {}", file.display());
    f.render_widget(
        Paragraph::new(Line::from(file_str)).style(Style::default().fg(theme.primary)),
        chunks[0],
    );

    let ctx = node.map(format_node_context).unwrap_or_default();
    f.render_widget(
        Paragraph::new(Line::from(format!(" {ctx}"))).style(theme.base_style()),
        chunks[1],
    );

    f.render_widget(
        Paragraph::new(Line::from(" READ ONLY ")).style(theme.warning_style()),
        chunks[2],
    );

    let stats = search_stats
        .map(|s| format!(" SEARCH: {s}"))
        .unwrap_or_default();
    f.render_widget(
        Paragraph::new(Line::from(stats)).style(Style::default().fg(theme.accent)),
        chunks[3],
    );
}

fn format_node_context(node: &Node) -> String {
    let key = if node.key.is_empty() {
        "root"
    } else {
        &node.key
    };
    let ty = node.ty.as_str();
    format!("{key} : {}", capitalize(ty))
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

#[allow(dead_code)]
pub fn format_type_label(ty: DataType) -> String {
    capitalize(ty.as_str())
}
