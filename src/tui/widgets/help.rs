//! Centered help modal.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::theme::Theme;

pub const LOGO: &str = r"
████████╗██╗    ██╗██╗ ██████╗
╚══██╔══╝██║    ██║██║██╔════╝
   ██║   ██║ █╗ ██║██║██║  ███╗
   ██║   ██║███╗██║██║██║   ██║
   ██║   ╚███╔███╔╝██║╚██████╔╝
   ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝
";

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
    let modal_w = 60u16.min(area.width.saturating_sub(4));
    let modal_h = 22u16.min(area.height.saturating_sub(2));
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

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(LOGO.lines().count() as u16),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(LOGO)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.warning)),
        rows[0],
    );
    f.render_widget(
        Paragraph::new(Line::from("Inspect. Navigate. Understand."))
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
        rows[2],
    );
    f.render_widget(
        Paragraph::new(Line::from(format!("v{version}")))
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::DIM)),
        rows[3],
    );
    f.render_widget(
        Paragraph::new(Line::from("https://twig.wtf"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.primary))
            .wrap(Wrap { trim: false }),
        rows[4],
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )))
        .alignment(Alignment::Center),
        rows[5],
    );

    let mut key_lines: Vec<Line> = Vec::new();
    for (key, desc) in KEYBINDINGS {
        key_lines.push(Line::from(vec![
            Span::styled(format!("  {key:<10}"), Style::default().fg(theme.accent)),
            Span::raw("  "),
            Span::styled(*desc, Style::default().fg(theme.fg)),
        ]));
    }
    f.render_widget(Paragraph::new(key_lines), rows[6]);

    f.render_widget(
        Paragraph::new(Line::from("(Press Esc to close)"))
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::DIM)),
        rows[7],
    );
}