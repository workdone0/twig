//! Breadcrumbs widget showing the jq-style path to the focused node.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::core::model::Node;
use crate::tui::theme::Theme;

pub fn render(
    f: &mut Frame,
    area: Rect,
    theme: &Theme,
    node: Option<&Node>,
    store: Option<&crate::core::store::Store>,
) {
    let full_path = node
        .zip(store)
        .and_then(|(n, s)| s.get_path(n.id).ok())
        .unwrap_or_default();
    let display = truncate(&full_path, 100);
    let line = Line::from(vec![
        Span::styled("Path: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(display, theme.primary_style()),
        Span::styled("  (jq)", Style::default().add_modifier(Modifier::DIM)),
    ]);
    f.render_widget(Paragraph::new(line).style(theme.base_style()), area);
}

fn truncate(path: &str, max: usize) -> String {
    if path.len() <= max {
        return path.to_string();
    }
    let parts: Vec<&str> = path.split('.').collect();
    if parts.len() > 6 {
        let head = format!("{}.{}", parts[0], parts[1]);
        let tail = format!(
            "{}.{}.{}",
            parts[parts.len() - 3],
            parts[parts.len() - 2],
            parts[parts.len() - 1]
        );
        format!("{head} … {tail}")
    } else {
        format!("…{}", &path[path.len() - (max - 1)..])
    }
}
