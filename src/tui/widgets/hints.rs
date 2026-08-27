//! Bottom-of-screen hints bar.
//!
//! Renders a single line listing the most useful keybindings so users
//! don't have to remember (or open the help modal) to find the basic
//! navigation keys. The accent color highlights the key portion; the
//! description stays in the foreground color.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::theme::Theme;

/// Each entry: (key display string, description).
const HINTS: &[(&str, &str)] = &[
    ("/", "search"),
    (":", "jump"),
    ("c", "path"),
    ("y", "src"),
    ("t", "theme"),
    ("?", "help"),
    ("q", "quit"),
];

pub fn render(f: &mut Frame, area: Rect, theme: &Theme) {
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::raw(" "));
    for (i, (key, desc)) in HINTS.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(
                "  ",
                Style::default().add_modifier(ratatui::style::Modifier::DIM),
            ));
        }
        spans.push(Span::styled(
            format!("{key} "),
            Style::default()
                .fg(theme.accent)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ));
        spans.push(Span::styled(*desc, Style::default().fg(theme.fg)));
    }
    f.render_widget(
        Paragraph::new(Line::from(spans)).style(theme.surface_style()),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hints_render_without_panic() {
        // Sanity: the renderer doesn't access any state beyond what
        // we pass in.
        let backend = ratatui::backend::TestBackend::new(80, 1);
        let mut term = ratatui::Terminal::new(backend).unwrap();
        term.draw(|f| {
            render(
                f,
                ratatui::layout::Rect::new(0, 0, 80, 1),
                &crate::tui::theme::CATPPUCCIN_MOCHA,
            );
        })
        .unwrap();
    }
}
