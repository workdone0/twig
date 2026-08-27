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
use crate::tui::theme::{Theme, ALL_THEMES, CATPPUCCIN_MOCHA};
use crate::tui::widgets::navigator::ColumnNavigator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Loading,
    Normal,
    Search,
    Jump,
    Help,
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
                        self.navigator = Some(ColumnNavigator::new(store));
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
                KeyCode::Down => {
                    if let Some(n) = self.navigator.as_mut() {
                        n.move_down();
                    }
                }
                KeyCode::Up => {
                    if let Some(n) = self.navigator.as_mut() {
                        n.move_up();
                    }
                }
                KeyCode::Right => {
                    if let Some(n) = self.navigator.as_mut() {
                        n.drill();
                    }
                }
                KeyCode::Left => {
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
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Enter) {
                    self.mode = AppMode::Normal;
                }
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
        let value = if node.is_container() {
            nav.store.reconstruct_value(node.id, 5).unwrap_or(serde_json::Value::Null)
        } else {
            node.value.clone().unwrap_or(serde_json::Value::Null)
        };
        let text = if self.format == "yaml" {
            serde_yml::to_string(&value).unwrap_or_default()
        } else {
            serde_json::to_string_pretty(&value).unwrap_or_default()
        };
        let label = if self.format == "yaml" { "YAML source" } else { "source" };
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
        app.navigator.as_ref().map(|n| &n.store),
    );

    // Body: loading splash or column navigator.
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
                .constraints([
                    Constraint::Percentage(75),
                    Constraint::Percentage(25),
                ])
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
        AppMode::Exiting | AppMode::Search | AppMode::Jump | AppMode::Help => {}
    }

    crate::tui::widgets::status_bar::render(
        f,
        chunks[3],
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