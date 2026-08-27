//! The interactive TUI application.
//!
//! Holds the active [`crate::core::store::Store`], the focused node,
//! the current theme, and the modal-mode state machine. The actual
//! rendering is delegated to widgets; this module owns the event loop
//! and key dispatch.
//!
//! Wiring of background ingestion, breadcrumbs, inspector, status bar,
//! modals, and clipboard all lands in subsequent commits; this commit
//! only lands the shell so later ones fill in the rest.

use std::path::Path;
use std::sync::mpsc;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

use crate::adapters::loader::Loader;
use crate::core::config::Config;
use crate::core::store::Store;
use crate::tui::theme::{Theme, ALL_THEMES, CATPPUCCIN_MOCHA};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Loading,
    Normal,
    Exiting,
}

pub enum LoadEvent {
    Loaded(Store),
    Error(String),
}

pub struct App {
    pub file: std::path::PathBuf,
    pub force_rebuild: bool,
    pub store: Option<Store>,
    pub mode: AppMode,
    pub theme: Theme,
    pub status_message: Option<String>,
    pub error: Option<String>,
    config: Config,
}

impl App {
    pub fn new(file: &Path, force_rebuild: bool) -> Self {
        let config = Config::load();
        let theme_name = config.get_string("theme").unwrap_or(CATPPUCCIN_MOCHA.name);
        let theme = ALL_THEMES
            .iter()
            .find(|t| t.name == theme_name)
            .copied()
            .unwrap_or(&CATPPUCCIN_MOCHA)
            .clone();
        Self {
            file: file.to_path_buf(),
            force_rebuild,
            store: None,
            mode: AppMode::Loading,
            theme,
            status_message: None,
            error: None,
            config,
        }
    }

    /// Pick the appropriate loader based on file extension.
    pub fn loader_for(file: &Path) -> Box<dyn Loader> {
        match file
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .as_deref()
        {
            Some("yaml") | Some("yml") => Box::new(crate::adapters::yaml_loader::YamlLoader::new()),
            _ => Box::new(crate::adapters::json_loader::JsonLoader::new()),
        }
    }

    pub fn cycle_theme(&mut self) {
        let names: Vec<&'static str> = ALL_THEMES.iter().map(|t| t.name).collect();
        let current = names
            .iter()
            .position(|n| *n == self.theme.name)
            .unwrap_or(0);
        let next = (current + 1) % names.len();
        let next_name = names[next];
        if let Some(t) = ALL_THEMES.iter().find(|t| t.name == next_name) {
            self.theme = (*t).clone();
            let _ = self.config.set("theme", serde_json::Value::from(next_name));
            self.status_message = Some(format!("Theme: {next_name}"));
        }
    }

    /// Drive the event loop until the user quits. Spawns the loader
    /// on a background thread, pumps events from crossterm, and
    /// dispatches keys based on the current [`AppMode`].
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()>
    where
        B::Error: std::fmt::Debug + Send + Sync + 'static,
    {
        let (tx, rx) = mpsc::channel::<LoadEvent>();
        let file = self.file.clone();
        let force_rebuild = self.force_rebuild;
        std::thread::spawn(move || {
            let loader = App::loader_for(&file);
            match loader.load(&file, force_rebuild) {
                Ok(store) => {
                    let _ = tx.send(LoadEvent::Loaded(store));
                }
                Err(e) => {
                    let _ = tx.send(LoadEvent::Error(format!("{e:#}")));
                }
            }
        });

        let tick = std::time::Duration::from_millis(50);
        loop {
            // Drain loader events.
            while let Ok(ev) = rx.try_recv() {
                match ev {
                    LoadEvent::Loaded(store) => {
                        self.store = Some(store);
                        self.mode = AppMode::Normal;
                    }
                    LoadEvent::Error(msg) => {
                        self.error = Some(msg);
                        self.mode = AppMode::Exiting;
                    }
                }
            }

            if let Err(e) = terminal.draw(|f| render(f, self)) {
                eprintln!("twig: draw error: {e:?}");
                self.mode = AppMode::Exiting;
            }

            if self.mode == AppMode::Exiting {
                return Ok(());
            }

            if crossterm::event::poll(tick)? {
                if let Event::Key(key) = crossterm::event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.on_key(key);
                    }
                }
            }
        }
    }

    fn on_key(&mut self, key: KeyEvent) {
        // Ctrl-C always exits, regardless of mode.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.mode = AppMode::Exiting;
            return;
        }
        match self.mode {
            AppMode::Loading => {
                if key.code == KeyCode::Char('q') {
                    self.mode = AppMode::Exiting;
                }
            }
            AppMode::Normal => match key.code {
                KeyCode::Char('q') => self.mode = AppMode::Exiting,
                KeyCode::Char('t') => self.cycle_theme(),
                KeyCode::Char('?') | KeyCode::Char('h') => {
                    self.status_message = Some(
                        "Twig: ←/→ columns • ↑/↓ row • / search • : jump • t theme • ? help • q quit"
                            .to_string(),
                    );
                }
                _ => {}
            },
            AppMode::Exiting => {}
        }
    }
}

fn render(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();
    let theme = &app.theme;
    let bg = theme.base_style();

    f.render_widget(Block::default().style(bg), area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(1),    // body
            Constraint::Length(1), // status
        ])
        .split(area);

    let header = Paragraph::new(Line::from(format!(
        " twig · {} · theme: {} ",
        app.file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?"),
        app.theme.name
    )))
    .style(theme.surface_style())
    .block(Block::default());
    f.render_widget(header, chunks[0]);

    let body = match app.mode {
        AppMode::Loading => Paragraph::new(Line::from(format!(
            "Loading {}…",
            app.file.display()
        )))
        .style(theme.primary_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title("Loading"),
        ),
        AppMode::Normal => {
            let text = if let Some(store) = &app.store {
                format!(
                    "Loaded {} nodes from {}",
                    store.node_count().unwrap_or(0),
                    app.file.display()
                )
            } else {
                "No data loaded.".to_string()
            };
            Paragraph::new(Line::from(text))
                .style(theme.base_style())
                .block(Block::default().borders(Borders::ALL))
        }
        AppMode::Exiting => Paragraph::new(Line::from("Exiting…")),
    };
    f.render_widget(body, chunks[1]);

    let status = match (&app.error, &app.status_message) {
        (Some(e), _) => format!(" ERROR: {e}"),
        (None, Some(s)) => format!(" {s}"),
        (None, None) => " Press ? for help, q to quit ".to_string(),
    };
    let status_widget = Paragraph::new(Line::from(status))
        .style(theme.surface_style())
        .block(Block::default());
    f.render_widget(status_widget, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn app_renders_initial_loading_state() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App::new(std::path::Path::new("does-not-exist.json"), false);
        terminal
            .draw(|f| render(f, &mut { app }))
            .unwrap();
    }

    #[test]
    fn cycle_theme_advances() {
        let mut app = App::new(std::path::Path::new("x.json"), false);
        let first = app.theme.name.to_string();
        app.cycle_theme();
        let second = app.theme.name.to_string();
        assert_ne!(first, second);
    }
}