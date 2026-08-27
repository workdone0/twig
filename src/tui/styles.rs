//! Small style helpers used across widgets.
//!
//! Style composition in ratatui happens one trait at a time, so we
//! centralize the common building blocks here: muted foregrounds,
//! highlight backgrounds, search-match foregrounds, etc.

use ratatui::style::{Modifier, Style};

use crate::tui::theme::Theme;

pub fn muted(theme: &Theme) -> Style {
    Style::default().fg(theme.fg).add_modifier(Modifier::DIM)
}

pub fn highlighted(theme: &Theme) -> Style {
    Style::default()
        .bg(theme.surface)
        .fg(theme.fg)
        .add_modifier(Modifier::BOLD)
}

pub fn search_match(theme: &Theme) -> Style {
    Style::default()
        .fg(theme.warning)
        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
}