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
use uuid::Uuid;

use crate::adapters::loader::Loader;
use crate::core::config::Config;
use crate::core::model::Node;
use crate::core::store::Store;
use crate::tui::theme::{Theme, ALL_THEMES};
use crate::tui::widgets::navigator::ColumnNavigator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Loading,
    Normal,
    Search,
    Jump,
    Help,
    /// Initial load failed. We render the error in-app and wait for
    /// the user to dismiss it before exiting.
    Error,
    Exiting,
}

pub enum LoadEvent {
    Loaded(Store),
    Error(String),
}

pub struct App {
    pub file: std::path::PathBuf,
    pub force_rebuild: bool,
    pub navigator: Option<ColumnNavigator>,
    pub mode: AppMode,
    pub theme: Theme,
    pub status_message: Option<String>,
    pub error: Option<String>,
    pub focused: Option<Node>,
    pub search_stats: Option<String>,
    pub frame: usize,
    pub modal_input: String,
    pub last_search_query: Option<String>,
    pub format: String,
    config: Config,
    config_path: Option<std::path::PathBuf>,
    pub load_options: crate::adapters::loader::LoadOptions,
}

impl App {
    pub fn new(file: &Path, force_rebuild: bool) -> Self {
        Self::with_config(
            file,
            force_rebuild,
            Config::load(),
            Some(crate::core::config::default_path()),
        )
    }

    pub fn with_config(
        file: &Path,
        force_rebuild: bool,
        config: Config,
        config_path: Option<std::path::PathBuf>,
    ) -> Self {
        // Pick the theme from config; fall back to the registered
        // default (Catppuccin Mocha) if config has none or refers to
        // an unknown theme. The `ALL_THEMES[0]` lookup ensures the
        // registered default and the fallback can't drift.
        let configured = config.get_string("theme");
        let theme = configured
            .and_then(|name| ALL_THEMES.iter().find(|t| t.name == name).copied())
            .unwrap_or(ALL_THEMES[0])
            .clone();
        Self {
            file: file.to_path_buf(),
            force_rebuild,
            navigator: None,
            mode: AppMode::Loading,
            theme,
            status_message: None,
            error: None,
            focused: None,
            search_stats: None,
            frame: 0,
            modal_input: String::new(),
            last_search_query: None,
            format: Self::format_for(file),
            config,
            config_path,
            load_options: Default::default(),
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

    fn format_for(file: &Path) -> String {
        match file
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .as_deref()
        {
            Some("yaml") | Some("yml") => "yaml".to_string(),
            _ => "json".to_string(),
        }
    }

    pub fn cycle_theme(&mut self) {
        // Cycle through ALL_THEMES in declaration order. Default
        // (Catppuccin Mocha, at index 0) is reachable from any other
        // theme by cycling enough times.
        let names: Vec<&'static str> = ALL_THEMES.iter().map(|t| t.name).collect();
        let current = names
            .iter()
            .position(|n| *n == self.theme.name)
            .unwrap_or(0);
        let next = (current + 1) % names.len();
        let next_name = names[next];
        if let Some(t) = ALL_THEMES.iter().find(|t| t.name == next_name) {
            self.theme = (*t).clone();
            self.config
                .set_memory("theme", serde_json::Value::from(next_name));
            self.status_message = Some(format!("Theme: {next_name}"));
            if let Some(path) = &self.config_path {
                if let Err(e) = self.config.save_to(path) {
                    self.status_message = Some(format!("Theme changed, but could not save: {e}"));
                }
            }
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()>
    where
        B::Error: std::fmt::Debug + Send + Sync + 'static,
    {
        let (tx, rx) = mpsc::channel::<LoadEvent>();
        let file = self.file.clone();
        let force_rebuild = self.force_rebuild;
        let options = self.load_options.clone();
        let worker = std::thread::spawn(move || {
            let loader: Box<dyn Loader> = if crate::cli::is_yaml(&file) {
                Box::new(crate::adapters::yaml_loader::YamlLoader::with_options(
                    options,
                ))
            } else {
                Box::new(crate::adapters::json_loader::JsonLoader::with_options(
                    options,
                ))
            };
            match loader.load(&file, force_rebuild) {
                Ok(store) => {
                    let _ = tx.send(LoadEvent::Loaded(store));
                }
                Err(e) => {
                    let _ = tx.send(LoadEvent::Error(format!("{e:#}")));
                }
            }
        });

        struct WorkerGuard {
            cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
            worker: Option<std::thread::JoinHandle<()>>,
        }
        impl Drop for WorkerGuard {
            fn drop(&mut self) {
                self.cancelled
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                if let Some(worker) = self.worker.take() {
                    let _ = worker.join();
                }
            }
        }
        let _worker = WorkerGuard {
            cancelled: self.load_options.cancelled.clone(),
            worker: Some(worker),
        };
        let tick = std::time::Duration::from_millis(50);
        loop {
            loop {
                let ev = match rx.try_recv() {
                    Ok(ev) => ev,
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        if self.mode == AppMode::Loading {
                            self.error = Some("Loader stopped unexpectedly".into());
                            self.mode = AppMode::Error;
                        }
                        break;
                    }
                };
                match ev {
                    LoadEvent::Loaded(store) => {
                        self.navigator = Some(ColumnNavigator::new(store));
                        self.focused = self.navigator.as_ref().and_then(|n| n.focused()).cloned();
                        self.mode = AppMode::Normal;
                    }
                    LoadEvent::Error(msg) => {
                        // Stay in the TUI and render the error
                        // message in-app instead of immediately
                        // closing the terminal. The user reads it
                        // and presses a key to exit; main.rs also
                        // prints the same message to stderr on
                        // exit for shells / logs.
                        self.error = Some(msg);
                        self.mode = AppMode::Error;
                    }
                }
            }

            if let Err(e) = terminal.draw(|f| render(f, self)) {
                self.error = Some(format!("Terminal draw error: {e:?}"));
                self.mode = AppMode::Exiting;
            }

            if self.mode == AppMode::Exiting {
                break;
            }

            if crossterm::event::poll(tick)? {
                match crossterm::event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key(key),
                    Event::Mouse(mouse) if self.mode == AppMode::Normal => {
                        if let Some(nav) = &mut self.navigator {
                            nav.on_mouse(mouse);
                            self.focused = nav.focused().cloned();
                        }
                    }
                    _ => {}
                }
            }
            self.frame = self.frame.wrapping_add(1);
        }

        // The terminal guard in main restores state before errors are printed.
        if let Some(msg) = self.error.take() {
            Err(anyhow::anyhow!("{msg}"))
        } else {
            Ok(())
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
                KeyCode::Char('?') => {
                    self.mode = AppMode::Help;
                }
                KeyCode::Char('/') => {
                    self.modal_input.clear();
                    self.mode = AppMode::Search;
                }
                KeyCode::Char(':') => {
                    self.modal_input.clear();
                    self.mode = AppMode::Jump;
                }
                KeyCode::Char('n') => self.next_match(1),
                KeyCode::Char('N') => self.next_match(-1),
                KeyCode::Char('c') => self.copy_path(),
                KeyCode::Char('y') => self.copy_source(),
                KeyCode::Char('g') | KeyCode::Home => {
                    if let Some(n) = &mut self.navigator {
                        n.first();
                    }
                }
                KeyCode::Char('G') | KeyCode::End => {
                    if let Some(n) = &mut self.navigator {
                        n.last();
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if let Some(n) = self.navigator.as_mut() {
                        n.move_down();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(n) = self.navigator.as_mut() {
                        n.move_up();
                    }
                }
                KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => {
                    if let Some(n) = self.navigator.as_mut() {
                        n.drill();
                    }
                }
                KeyCode::Left | KeyCode::Esc | KeyCode::Char('h') => {
                    if let Some(n) = self.navigator.as_mut() {
                        n.step_back();
                    }
                }
                _ => {}
            },
            AppMode::Search => match key.code {
                KeyCode::Esc => self.mode = AppMode::Normal,
                KeyCode::Enter => self.run_search(),
                KeyCode::Backspace => {
                    self.modal_input.pop();
                }
                KeyCode::Char(c) => self.modal_input.push(c),
                _ => {}
            },
            AppMode::Jump => match key.code {
                KeyCode::Esc => self.mode = AppMode::Normal,
                KeyCode::Enter => self.run_jump(),
                KeyCode::Backspace => {
                    self.modal_input.pop();
                }
                KeyCode::Char(c) => self.modal_input.push(c),
                _ => {}
            },
            AppMode::Help => {
                if matches!(
                    key.code,
                    KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Enter
                ) {
                    self.mode = AppMode::Normal;
                }
            }
            AppMode::Error => {
                // Any key dismisses the error screen.
                self.mode = AppMode::Exiting;
            }
            AppMode::Exiting => {}
        }
        // Refresh focused node.
        if let Some(nav) = &self.navigator {
            self.focused = nav.focused().cloned();
        }
    }

    fn next_match(&mut self, direction: i32) {
        let query = match &self.last_search_query {
            Some(q) => q.clone(),
            None => {
                self.status_message = Some("No active search query.".to_string());
                return;
            }
        };
        if let Some(nav) = self.navigator.as_mut() {
            if let Some(node) = nav.find_next(&query, direction) {
                nav.expand_to_node(node.id);
                nav.set_search(self.last_search_query.clone());
                self.focused = Some(node.clone());
                self.update_search_stats(&node.id, &query);
            } else {
                self.status_message = Some(format!("Not found: '{query}'"));
            }
        }
    }

    fn run_search(&mut self) {
        let query = self.modal_input.trim().to_string();
        self.mode = AppMode::Normal;
        if query.is_empty() {
            return;
        }
        self.last_search_query = Some(query.clone());
        self.next_match(1);
        if let Some(nav) = self.navigator.as_mut() {
            nav.set_search(Some(query));
        }
    }

    fn run_jump(&mut self) {
        let path = self.modal_input.trim().to_string();
        self.mode = AppMode::Normal;
        if path.is_empty() {
            return;
        }
        if let Some(nav) = self.navigator.as_mut() {
            match nav.store.resolve_path(&path) {
                Ok(Some(node)) => {
                    nav.expand_to_node(node.id);
                    self.focused = Some(node);
                }
                Ok(None) => {
                    self.status_message = Some(format!("Path not found: {path}"));
                }
                Err(e) => {
                    self.status_message = Some(format!("{e}"));
                }
            }
        }
    }

    fn update_search_stats(&mut self, current_id: &Uuid, query: &str) {
        if let Some(nav) = &self.navigator {
            if let Ok((current, total)) = nav.store.get_search_stats(query, Some(*current_id)) {
                if total > 0 {
                    self.search_stats = Some(format!("{current}/{total}"));
                }
            }
        }
    }

    fn copy_path(&mut self) {
        let Some(node) = self.focused.clone() else {
            return;
        };
        let Some(nav) = &self.navigator else {
            return;
        };
        let path = match nav.store.get_path(node.id) {
            Ok(p) => p,
            Err(_) => ".".to_string(),
        };
        match crate::tui::widgets::clipboard::Clipboard::copy(&path) {
            Ok(()) => self.status_message = Some(format!("Copied path: {path}")),
            Err(e) => self.status_message = Some(format!("Clipboard: {e}")),
        }
    }

    fn copy_source(&mut self) {
        let Some(node) = self.focused.clone() else {
            return;
        };
        let Some(nav) = &self.navigator else {
            return;
        };
        let value = match nav.store.export_value(node.id) {
            Ok(value) => value,
            Err(e) => {
                self.status_message = Some(format!("Copy failed: {e}"));
                return;
            }
        };
        let text = if self.format == "yaml" {
            serde_norway::to_string(&value).unwrap_or_default()
        } else {
            serde_json::to_string_pretty(&value).unwrap_or_default()
        };
        let label = if self.format == "yaml" {
            "YAML source"
        } else {
            "source"
        };
        match crate::tui::widgets::clipboard::Clipboard::copy(&text) {
            Ok(()) => {
                let preview: String = text.chars().take(50).collect();
                let preview = preview.replace('\n', " ");
                let suffix = if text.len() > 50 { "..." } else { "" };
                self.status_message = Some(format!("Copied {label}: {preview}{suffix}"));
            }
            Err(e) => self.status_message = Some(format!("Clipboard: {e}")),
        }
    }
}

fn render(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();
    let theme = &app.theme;

    f.render_widget(Block::default().style(theme.base_style()), area);

    // Top-level vertical split: header / breadcrumbs / body / hints / status.
    // The header gets 2 rows so the title line and the bottom border don't
    // touch the breadcrumbs directly.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // header (with bottom border)
            Constraint::Length(1), // breadcrumbs
            Constraint::Min(1),    // body
            Constraint::Length(1), // hints / keybinding bar
            Constraint::Length(1), // status
        ])
        .split(area);

    let header = Paragraph::new(Line::from(format!(
        " twig · {} · theme: {} ",
        app.file.file_name().and_then(|s| s.to_str()).unwrap_or("?"),
        app.theme.name
    )))
    .style(theme.surface_style())
    .block(
        Block::default()
            .borders(ratatui::widgets::Borders::BOTTOM)
            .border_style(theme.surface_style()),
    );
    f.render_widget(header, chunks[0]);

    crate::tui::widgets::breadcrumbs::render(
        f,
        chunks[1],
        theme,
        app.focused.as_ref(),
        app.navigator.as_ref().map(|n| &n.store),
    );

    // Body: loading splash or column navigator.
    match app.mode {
        AppMode::Loading => {
            crate::tui::widgets::loading::render(
                f,
                chunks[2],
                theme,
                &format!(
                    "{} · read {:.1}/{:.1} MiB · {} nodes",
                    app.file.display(),
                    app.load_options
                        .progress
                        .bytes
                        .load(std::sync::atomic::Ordering::Relaxed) as f64
                        / 1_048_576.0,
                    app.load_options
                        .progress
                        .total
                        .load(std::sync::atomic::Ordering::Relaxed) as f64
                        / 1_048_576.0,
                    app.load_options
                        .progress
                        .nodes
                        .load(std::sync::atomic::Ordering::Relaxed)
                ),
                app.frame,
            );
        }
        AppMode::Error => {
            crate::tui::widgets::error::render(f, chunks[2], theme, app.error.as_deref());
        }
        AppMode::Normal | AppMode::Search | AppMode::Jump | AppMode::Help => {
            // Refresh focused node from navigator so the inspector
            // tracks the user's selection.
            if app.focused.is_none() {
                if let Some(nav) = app.navigator.as_ref() {
                    app.focused = nav.focused().cloned();
                }
            }
            // Body: 75% navigator, 25% inspector.
            let body_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
                .split(chunks[2]);
            if let Some(nav) = app.navigator.as_mut() {
                nav.render(f, body_chunks[0], theme);
            } else {
                let text = format!("Loaded {}", app.file.display());
                f.render_widget(
                    Paragraph::new(Line::from(text)).block(Block::default()),
                    body_chunks[0],
                );
            }
            crate::tui::widgets::inspector::render(
                f,
                body_chunks[1],
                theme,
                app.focused.as_ref(),
                app.navigator.as_ref().map(|n| &n.store),
                &app.format,
            );
        }
        AppMode::Exiting => {}
    }

    crate::tui::widgets::hints::render(f, chunks[3], theme);

    crate::tui::widgets::status_bar::render(
        f,
        chunks[4],
        theme,
        &app.file,
        app.focused.as_ref(),
        app.search_stats.as_deref(),
    );

    // Modal overlays.
    match app.mode {
        AppMode::Search => {
            crate::tui::widgets::search::render(f, area, theme, &app.modal_input);
        }
        AppMode::Jump => {
            crate::tui::widgets::jump::render(f, area, theme, &app.modal_input);
        }
        AppMode::Help => {
            crate::tui::widgets::help::render(f, area, theme, env!("CARGO_PKG_VERSION"));
        }
        _ => {}
    }

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
        let app = App::with_config(
            std::path::Path::new("does-not-exist.json"),
            false,
            Config::default(),
            None,
        );
        terminal.draw(|f| render(f, &mut { app })).unwrap();
    }

    // Injected config keeps theme persistence independent of the user profile.
    #[test]
    fn default_theme_is_catppuccin_and_cycle_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let mut app = App::with_config(
            std::path::Path::new("x.json"),
            false,
            Config::default(),
            Some(dir.path().join("config.json")),
        );
        assert_eq!(app.theme.name, "catppuccin-mocha");
        app.cycle_theme();
        assert_ne!(app.theme.name, "catppuccin-mocha");
        app.cycle_theme();
        assert_eq!(app.theme.name, "catppuccin-mocha");
    }
}

#[cfg(test)]
mod interaction_tests {
    use super::*;
    fn loaded_app() -> (tempfile::TempDir, App) {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("data.json");
        std::fs::write(&file, r#"{"first":{"nested":42},"last":"😀"}"#).unwrap();
        let store = crate::adapters::json_loader::JsonLoader::new()
            .with_cache_dir(dir.path().join("cache"))
            .load(&file, true)
            .unwrap();
        let mut app = App::with_config(&file, false, Config::default(), None);
        app.navigator = Some(ColumnNavigator::new(store));
        app.mode = AppMode::Normal;
        app.focused = app.navigator.as_ref().and_then(|n| n.focused()).cloned();
        (dir, app)
    }
    #[test]
    fn keys_navigation_modals_and_copy_selection() {
        let (_dir, mut app) = loaded_app();
        let press = |app: &mut App, c| app.on_key(KeyEvent::new(c, KeyModifiers::NONE));
        assert_eq!(app.focused.as_ref().unwrap().key, "first");
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.focused.as_ref().unwrap().key, "nested");
        press(&mut app, KeyCode::Char('h'));
        press(&mut app, KeyCode::Char('G'));
        assert_eq!(app.focused.as_ref().unwrap().key, "last");
        press(&mut app, KeyCode::Char('g'));
        assert_eq!(app.focused.as_ref().unwrap().key, "first");
        press(&mut app, KeyCode::Char('/'));
        press(&mut app, KeyCode::Char('4'));
        press(&mut app, KeyCode::Char('2'));
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.focused.as_ref().unwrap().key, "nested");
        press(&mut app, KeyCode::Char('?'));
        assert_eq!(app.mode, AppMode::Help);
        press(&mut app, KeyCode::Esc);
        assert_eq!(app.mode, AppMode::Normal);
        app.on_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert_eq!(app.mode, AppMode::Exiting);
    }
    #[test]
    fn every_modal_renders_at_tiny_sizes_and_config_errors_are_visible() {
        let (dir, mut app) = loaded_app();
        app.config_path = Some(dir.path().to_path_buf());
        app.cycle_theme();
        assert!(app
            .status_message
            .as_ref()
            .unwrap()
            .contains("could not save"));
        for mode in [
            AppMode::Normal,
            AppMode::Loading,
            AppMode::Help,
            AppMode::Search,
            AppMode::Jump,
            AppMode::Error,
        ] {
            app.mode = mode;
            for (w, h) in [(0, 0), (1, 1), (10, 3), (80, 24)] {
                let mut terminal = Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
                terminal.draw(|f| render(f, &mut app)).unwrap();
            }
        }
    }
}
