//! Centered help modal.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::theme::Theme;

pub const LOGO: &str = "████████╗██╗    ██╗██╗ ██████╗
╚══██╔══╝██║    ██║██║██╔════╝
   ██║   ██║ █╗ ██║██║██║  ███╗
   ██║   ██║███╗██║██║██║   ██║
   ██║   ╚███╔███╔╝██║╚██████╔╝
   ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ ";

const KEYBINDINGS: &[(&str, &str)] = &[
    ("Arrows", "Navigate tree"),
    ("/", "Search"),
    ("n / N", "Next / Prev match"),
    (":", "Jump to path"),
    ("c", "Copy path"),
    ("y", "Copy source"),
    ("t", "Toggle theme"),
    ("q", "Quit"),
];

pub fn render(f: &mut Frame, area: Rect, theme: &Theme, version: &str) {
    let logo_lines = LOGO.lines().count() as u16;
    let key_count = KEYBINDINGS.len() as u16;
    // 1 (tagline) + 1 (version) + 1 (url) + key_count + 1 (close hint)
    // + 1 (footer spacer) = 5 + key_count. Plus the logo.
    let content_h = logo_lines + 5 + key_count;
    // Account for top + bottom border.
    let needed_h = content_h + 2;
    let modal_h = needed_h.min(area.height.saturating_sub(2)).max(8);
    let modal_w = 60u16.min(area.width.saturating_sub(4)).max(20);
    let modal = Rect {
        x: area.x + (area.width.saturating_sub(modal_w)) / 2,
        y: area.y + (area.height.saturating_sub(modal_h)) / 2,
        width: modal_w,
        height: modal_h,
    };

    f.render_widget(Clear, modal);
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));
    let inner = block.inner(modal);
    f.render_widget(block, modal);

    let mut rows = vec![Constraint::Length(logo_lines)];
    rows.push(Constraint::Length(1)); // tagline
    rows.push(Constraint::Length(1)); // version
    rows.push(Constraint::Length(1)); // url
    rows.push(Constraint::Length(key_count)); // keybindings
    rows.push(Constraint::Length(1)); // close hint
    rows.push(Constraint::Min(0)); // bottom padding (can shrink to 0)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(rows)
        .split(inner);

    f.render_widget(
        Paragraph::new(LOGO)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.warning)),
        chunks[0],
    );
    f.render_widget(
        Paragraph::new(Line::from("Inspect. Navigate. Understand."))
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
        chunks[1],
    );
    f.render_widget(
        Paragraph::new(Line::from(format!("v{version}")))
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::DIM)),
        chunks[2],
    );
    f.render_widget(
        Paragraph::new(Line::from("https://twig.wtf"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.primary))
            .wrap(Wrap { trim: false }),
        chunks[3],
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )))
        .alignment(Alignment::Center),
        chunks[3],
    );

    let key_lines: Vec<Line> = KEYBINDINGS
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(format!("  {key:<10}"), Style::default().fg(theme.accent)),
                Span::raw("  "),
                Span::styled(*desc, Style::default().fg(theme.fg)),
            ])
        })
        .collect();
    f.render_widget(Paragraph::new(key_lines), chunks[4]);

    f.render_widget(
        Paragraph::new(Line::from("(Press Esc or ? to close)"))
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::DIM)),
        chunks[5],
    );
}
