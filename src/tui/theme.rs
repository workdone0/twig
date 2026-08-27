//! Theme palettes used by the TUI.
//!
//! **Catppuccin Mocha is the default theme.** That guarantee is locked
//! in three places:
//!
//!   1. `ALL_THEMES[0]` is `CATPPUCCIN_MOCHA` (compile-time const), so
//!      `App::cycle_theme` advances *from* Catppuccin on first press
//!      of `t`.
//!   2. `Config::DEFAULT_THEME = "catppuccin-mocha"` is what fresh
//!      installs write into `config.json`.
//!   3. `App::new` falls back to `CATPPUCCIN_MOCHA` whenever the
//!      config file is missing, unreadable, or references an unknown
//!      theme name. A regression test in this module asserts that
//!      `ALL_THEMES[0]` stays Catppuccin Mocha.
//!
//! Solarized Dark is the second entry for the `t` cycle.

use ratatui::style::{Color, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub fg: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub surface: Color,
    pub panel: Color,
    pub is_dark: bool,
}

pub const CATPPUCCIN_MOCHA: Theme = Theme {
    name: "catppuccin-mocha",
    bg: Color::Rgb(0x1e, 0x1e, 0x2e),
    fg: Color::Rgb(0xcd, 0xd6, 0xf4),
    primary: Color::Rgb(0xcb, 0xa6, 0xf7),
    secondary: Color::Rgb(0x89, 0xb4, 0xfa),
    accent: Color::Rgb(0xf3, 0x8b, 0xa8),
    success: Color::Rgb(0xa6, 0xe3, 0xa1),
    warning: Color::Rgb(0xf9, 0xe2, 0xaf),
    error: Color::Rgb(0xf3, 0x8b, 0xa8),
    surface: Color::Rgb(0x31, 0x32, 0x44),
    panel: Color::Rgb(0x31, 0x32, 0x44),
    is_dark: true,
};

pub const SOLARIZED_DARK: Theme = Theme {
    name: "solarized-dark",
    bg: Color::Rgb(0x00, 0x2b, 0x36),
    fg: Color::Rgb(0x83, 0x94, 0x96),
    primary: Color::Rgb(0x26, 0x8b, 0xd2),
    secondary: Color::Rgb(0x2a, 0xa1, 0x98),
    accent: Color::Rgb(0xdc, 0x32, 0x2f),
    success: Color::Rgb(0x85, 0x99, 0x00),
    warning: Color::Rgb(0xb5, 0x89, 0x00),
    error: Color::Rgb(0xdc, 0x32, 0x2f),
    surface: Color::Rgb(0x07, 0x36, 0x42),
    panel: Color::Rgb(0x07, 0x36, 0x42),
    is_dark: true,
};

/// Order matters: `[0]` is the default. Don't shuffle.
pub const ALL_THEMES: &[&Theme] = &[&CATPPUCCIN_MOCHA, &SOLARIZED_DARK];

/// Name of the default theme. Used by `Config::default()` and asserted
/// against by tests so a reorder of `ALL_THEMES` can't silently change
/// the default.
pub const DEFAULT_THEME_NAME: &str = CATPPUCCIN_MOCHA.name;

impl Theme {
    pub fn base_style(&self) -> Style {
        Style::default().bg(self.bg).fg(self.fg)
    }

    pub fn surface_style(&self) -> Style {
        Style::default().bg(self.surface).fg(self.fg)
    }

    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.primary)
    }

    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catppuccin_mocha_has_expected_name() {
        assert_eq!(CATPPUCCIN_MOCHA.name, "catppuccin-mocha");
        const _: () = assert!(CATPPUCCIN_MOCHA.is_dark);
    }

    #[test]
    fn solarized_dark_has_expected_name() {
        assert_eq!(SOLARIZED_DARK.name, "solarized-dark");
        const _: () = assert!(SOLARIZED_DARK.is_dark);
    }

    #[test]
    fn all_themes_lists_both_themes() {
        let names: Vec<_> = ALL_THEMES.iter().map(|t| t.name).collect();
        assert!(names.contains(&"catppuccin-mocha"));
        assert!(names.contains(&"solarized-dark"));
    }

    /// Catppuccin must be the *first* entry in ALL_THEMES so a fresh
    /// install always boots into it. Reordering the slice silently
    /// would change the default — this test refuses that.
    #[test]
    fn catppuccin_is_default_theme() {
        assert_eq!(
            ALL_THEMES[0].name, CATPPUCCIN_MOCHA.name,
            "ALL_THEMES[0] must be Catppuccin Mocha; reorder the slice carefully"
        );
        assert_eq!(DEFAULT_THEME_NAME, "catppuccin-mocha");
    }
}
