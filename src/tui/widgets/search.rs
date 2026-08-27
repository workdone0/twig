//! Centered search input modal.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::tui::theme::Theme;

/// Render the modal and return the inner input area so the caller can
/// place a `tui-textarea` or similar widget inside. For the initial
/// cut we just draw the frame; the actual keypress capture happens in
/// the App's modal state machine (a follow-up commit).
pub fn render(f: &mut Frame, area: Rect, theme: &Theme, query: &str) {
    let modal_w = 60u16.min(area.width.saturating_sub(4));
    let modal_h = 5u16;
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
    f.render_widget(
        Paragraph::new(Line::from(format!("> {query}"))).style(theme.primary_style()),
        rows[1],
    );
    f.render_widget(
        Paragraph::new(Line::from("(Enter to search · Esc to cancel)"))
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::DIM)),
        rows[2],
    );
}
