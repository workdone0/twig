//! In-app error screen shown when the file failed to load.
//!
//! Renders the error message centered, with a short hint about how
//! to escape (any key) and, when the error looks like a JSON parse
//! failure, a reminder that `twig --fix <file>` can repair it.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, theme: &Theme, error: Option<&str>) {
    let message = error.unwrap_or("(no detail)");
    let modal_w = (area.width.saturating_sub(4)).clamp(20, 80);
    // Account for: 2 borders + 2 padding rows (top + bottom) + the 6
    // content rows below. Anything below 11 leaves at least one row
    // collapsed, which is fine on small terminals but means we
    // intentionally drop the dismiss hint.
    let modal_h = 11u16.min(area.height).max(7);
    let modal = Rect {
        x: area.x + (area.width.saturating_sub(modal_w)) / 2,
        y: area.y + (area.height.saturating_sub(modal_h)) / 2,
        width: modal_w,
        height: modal_h,
    };

    f.render_widget(Clear, modal);
    let block = Block::default()
        .title(" Load Failed ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.error))
        .padding(Padding::new(2, 2, 1, 1));
    let inner = block.inner(modal);
    f.render_widget(block, modal);

    let lower = message.to_lowercase();
    let is_parse_error = lower.contains("json")
        || lower.contains("parse")
        || lower.contains("expected")
        || lower.contains("trailing");

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header label
            Constraint::Length(1), // spacer
            Constraint::Min(2),    // error message body
            Constraint::Length(1), // spacer
            Constraint::Length(1), // hint
            Constraint::Length(1), // dismiss hint
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Could not load this file:",
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ))),
        rows[0],
    );

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            message,
            Style::default().fg(theme.error),
        )))
        .wrap(Wrap { trim: false }),
        rows[2],
    );

    let hint_text = if is_parse_error {
        "Hint: run `twig --fix <file>` to attempt automatic repair."
    } else {
        "Hint: check the file path, permissions, and format."
    };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            hint_text,
            Style::default().fg(theme.warning),
        ))),
        rows[4],
    );

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Press any key to exit",
            Style::default().add_modifier(Modifier::DIM),
        )))
        .alignment(Alignment::Center),
        rows[5],
    );
}

#[allow(dead_code)]
fn _ensure_unused(_a: Constraint) {}
