//! The interactive TUI application.
//!
//! Holds the active [`crate::core::store::Store`], the focused node,
//! the current theme, and the modal-mode state machine. The actual
//! rendering is delegated to widgets; this module owns the event loop
//! and key dispatch.

use std::path::Path;
use std::sync::mpsc;

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Terminal;

use crate::adapters::loader::Loader;
use crate::core::config::Config;
use crate::core::model::Node;
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
    pub focused: Option<Node>,
    pub search_stats: Option<String>,
    pub frame: usize,
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
            focused: None,
            search_stats: None,
            frame: 0,
            config,
        }
    }

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
            self.frame = self.frame.wrapping_add(1);
        }
    }

    fn on_key(&mut self, key: KeyEvent) {
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

    f.render_widget(Block::default().style(theme.base_style()), area);

    // Top-level vertical split: header / body / status.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // header
            Constraint::Length(1),  // breadcrumbs
            Constraint::Min(1),     // body
            Constraint::Length(1),  // status
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

    crate::tui::widgets::breadcrumbs::render(
        f,
        chunks[1],
        theme,
        app.focused.as_ref(),
        app.store.as_ref(),
    );

    // Body: loading splash or placeholder main view.
    match app.mode {
        AppMode::Loading => {
            crate::tui::widgets::loading::render(
                f,
                chunks[2],
                theme,
                &app.file.display().to_string(),
                app.frame,
            );
        }
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
            f.render_widget(
                Paragraph::new(Line::from(text)).block(Block::default()),
                chunks[2],
            );
        }
        AppMode::Exiting => {}
    }

    crate::tui::widgets::status_bar::render(
        f,
        chunks[3],
        theme,
        &app.file,
        app.focused.as_ref(),
        app.search_stats.as_deref(),
    );

    // Status / message text on top of the body for transient messages.
    match (&app.error, &app.status_message) {
        (Some(_), _) => {} // Errors shown in status bar; nothing extra.
        (None, Some(s)) => {
            // Lightweight on-body hint.
            let hint = Paragraph::new(Line::from(format!(" {s}")))
                .style(theme.primary_style())
                .block(Block::default());
            f.render_widget(
                hint,
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(1), Constraint::Length(1)])
                    .split(chunks[2])[1],
            );
        }
        (None, None) => {}
    }
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
        terminal.draw(|f| render(f, &mut { app })).unwrap();
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