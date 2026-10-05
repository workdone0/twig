//! Centered search input modal.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::tui::theme::Theme;

/// Render the search modal; App owns editing and key dispatch.
pub fn render(f: &mut Frame, area: Rect, theme: &Theme, query: &str) {
    let modal_w = 60u16.min(area.width);
    let modal_h = area.height.min(5);
    let modal = Rect {
        x: area.x + (area.width.saturating_sub(modal_w)) / 2,
        y: area.y + (area.height.saturating_sub(modal_h)) / 2,
        width: modal_w,
        height: modal_h,
    };

    f.render_widget(Clear, modal);
    let block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));
    let inner = block.inner(modal);
    f.render_widget(block, modal);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(Line::from("Search keys & values:"))
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::BOLD)),
        rows[0],
    );
    let cursor = Span::styled("▏", Style::default().fg(theme.accent));
    let input_line = Line::from(vec![
        Span::styled("> ", Style::default().fg(theme.primary)),
        Span::styled(query.to_string(), theme.primary_style()),
        cursor,
    ]);
    f.render_widget(
        Paragraph::new(input_line).style(theme.base_style()),
        rows[1],
    );
    f.render_widget(
        Paragraph::new(Line::from("(Enter to search · Esc to cancel)"))
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::DIM)),
        rows[2],
    );
}
