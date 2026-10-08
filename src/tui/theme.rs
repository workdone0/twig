//! Twig's two shared palettes, adapted to terminal colors.
use ratatui::style::{Color, Style};
use std::sync::LazyLock;
pub use twig_core::presentation::canonical_theme;
use twig_core::presentation::PALETTES;
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

fn palette(name: &'static str) -> Theme {
    let colors = &PALETTES[name];
    let color = |key: &str| {
        let rgb = u32::from_str_radix(&colors[key][1..], 16).expect("valid palette color");
        Color::Rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
    };
    Theme {
        name,
        bg: color("bg"),
        fg: color("fg"),
        primary: color("primary"),
        secondary: color("secondary"),
        accent: color("accent"),
        success: color("success"),
        warning: color("warning"),
        error: color("error"),
        surface: color("surface"),
        panel: color("panel"),
        is_dark: name == "dark",
    }
}
pub static DARK: LazyLock<Theme> = LazyLock::new(|| palette("dark"));
pub static LIGHT: LazyLock<Theme> = LazyLock::new(|| palette("light"));
pub static ALL_THEMES: LazyLock<[&Theme; 2]> = LazyLock::new(|| [&DARK, &LIGHT]);
pub const DEFAULT_THEME_NAME: &str = twig_core::presentation::DEFAULT_THEME;
// Compatibility aliases for callers; only Dark and Light are selectable.
pub use DARK as CATPPUCCIN_MOCHA;
pub use DARK as SOLARIZED_DARK;

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
    fn exactly_dark_and_light_with_dark_default() {
        assert_eq!(
            ALL_THEMES.iter().map(|t| t.name).collect::<Vec<_>>(),
            ["dark", "light"]
        );
        assert_eq!(DEFAULT_THEME_NAME, "dark");
        assert!(DARK.is_dark);
        assert!(!LIGHT.is_dark);
        assert_ne!(DARK.bg, LIGHT.bg);
    }
}
