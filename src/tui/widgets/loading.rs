//! Loading splash shown while ingestion is in progress.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
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

const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Render the splash centered inside `area`. Animates the spinner via
/// the caller's `frame` counter.
pub fn render(f: &mut Frame, area: Rect, theme: &Theme, file: &str, frame: usize) {
    let block = Block::default()
        .title(" Loading ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(LOGO.lines().count() as u16),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(block.inner(area));

    f.render_widget(Clear, area);
    f.render_widget(block, area);

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