//! Loading splash shown while ingestion is in progress.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::theme::Theme;

pub const LOGO: &str = "████████╗██╗    ██╗██╗ ██████╗
╚══██╔══╝██║    ██║██║██╔════╝
   ██║   ██║ █╗ ██║██║██║  ███╗
   ██║   ██║███╗██║██║██║   ██║
   ██║   ╚███╔███╔╝██║╚██████╔╝
   ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ ";

const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Render the splash centered inside `area`. Animates the spinner via
/// the caller's `frame` counter.
pub fn render(f: &mut Frame, area: Rect, theme: &Theme, file: &str, frame: usize) {
    let logo_lines = LOGO.lines().count() as u16;
    let block = Block::default()
        .title(" Loading ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .padding(ratatui::widgets::Padding::new(2, 2, 1, 1));
    let modal_h = (logo_lines + 4 + 2)
        .min(area.height.saturating_sub(2))
        .max(6);
    let modal_w = 60u16.min(area.width.saturating_sub(4)).max(20);
    let modal = Rect {
        x: area.x + (area.width.saturating_sub(modal_w)) / 2,
        y: area.y + (area.height.saturating_sub(modal_h)) / 2,
        width: modal_w,
        height: modal_h,
    };

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(logo_lines),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(block.inner(modal));

    f.render_widget(Clear, modal);
    f.render_widget(block, modal);

    let spin = SPINNER[frame % SPINNER.len()];
    f.render_widget(
        Paragraph::new(LOGO)
            .style(Style::default().fg(theme.warning))
            .alignment(Alignment::Center),
        inner[0],
    );
    f.render_widget(
        Paragraph::new(Line::from(format!("{spin}  ingesting {file}…")))
            .style(theme.primary_style())
            .alignment(Alignment::Center),
        inner[2],
    );
    f.render_widget(
        Paragraph::new(Line::from("https://twig.wtf"))
            .style(Style::default().fg(theme.fg))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false }),
        inner[4],
    );
}
