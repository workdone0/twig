//! Diagnostic harness: render every widget into a TestBackend at
//! realistic terminal sizes and print the resulting buffer to stderr
//! so we can eyeball what's actually on screen.
//!
//! Run with: `cargo run --example ui_dump -- /path/to/sample.json`

use std::env;
use std::path::PathBuf;

use ratatui::backend::TestBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;
use twig::tui::widgets::hints;
use twig::tui::widgets::jump;

use twig::adapters::json_loader::JsonLoader;
use twig::adapters::loader::Loader;
use twig::core::store::Store;
use twig::tui::theme::CATPPUCCIN_MOCHA;
use twig::tui::widgets::help;
use twig::tui::widgets::inspector;
use twig::tui::widgets::loading;
use twig::tui::widgets::navigator::ColumnNavigator;
use twig::tui::widgets::search;
use twig::tui::widgets::status_bar;

fn dump(label: &str, terminal: &Terminal<TestBackend>) {
    let buf = terminal.backend().buffer().clone();
    let area = buf.area;
    eprintln!(
        "\n========== {label} ({}x{}) ==========",
        area.width, area.height
    );
    for y in 0..area.height {
        let row: String = (0..area.width)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect();
        eprintln!("{row}");
    }
}

fn split_app(
    area: ratatui::layout::Rect,
) -> (
    ratatui::layout::Rect,
    ratatui::layout::Rect,
    ratatui::layout::Rect,
    ratatui::layout::Rect,
    ratatui::layout::Rect,
) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // header (with bottom border)
            Constraint::Length(1), // breadcrumbs
            Constraint::Min(1),    // body
            Constraint::Length(1), // hints
            Constraint::Length(1), // status
        ])
        .split(area);
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(vertical[2]);
    (vertical[0], vertical[1], body[0], vertical[3], vertical[4])
}

fn load(path: &std::path::Path) -> Store {
    let cache = std::env::temp_dir().join("twig-ui-dump");
    let _ = std::fs::create_dir_all(&cache);
    JsonLoader::new()
        .with_cache_dir(cache)
        .load(path, true)
        .expect("load")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args
        .get(1)
        .map(PathBuf::from)
        .expect("usage: ui_dump <file>");

    // 1) Header / breadcrumbs / status bar (80x24).
    {
        let backend = TestBackend::new(80, 24);
        let mut term = Terminal::new(backend).unwrap();
        let store = load(&path);
        let nav = ColumnNavigator::new(store);
        let focused = nav.focused().cloned();
        term.draw(|f| {
            let (header, breadcrumb, _body, hints_area, status) = split_app(f.area());
            let _ = nav;
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
            f.render_widget(
                Paragraph::new(Line::from(format!(
                    " twig · {} · theme: {} ",
                    filename, CATPPUCCIN_MOCHA.name
                )))
                .style(CATPPUCCIN_MOCHA.surface_style()),
                header,
            );
            twig::tui::widgets::breadcrumbs::render(
                f,
                breadcrumb,
                &CATPPUCCIN_MOCHA,
                focused.as_ref(),
                Some(&nav.store),
            );
            status_bar::render(f, status, &CATPPUCCIN_MOCHA, &path, focused.as_ref(), None);
            hints::render(f, hints_area, &CATPPUCCIN_MOCHA);
            hints::render(f, hints_area, &CATPPUCCIN_MOCHA);
        })
        .unwrap();
        dump("shell-only 80x24", &term);
    }

    // 2) Loading splash at 80x24.
    {
        let backend = TestBackend::new(80, 24);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| {
            loading::render(
                f,
                f.area(),
                &CATPPUCCIN_MOCHA,
                &path.display().to_string(),
                0,
            );
        })
        .unwrap();
        dump("loading 80x24", &term);
    }

    // 3) Help modal at 80x24.
    {
        let backend = TestBackend::new(80, 24);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| {
            help::render(f, f.area(), &CATPPUCCIN_MOCHA, "3.0.0");
        })
        .unwrap();
        eprintln!("\n[debug] help rect: {:?}", term.backend().buffer().area());
        dump("help 80x24", &term);
    }

    // 3b) Help modal at 120x40.
    {
        let backend = TestBackend::new(120, 40);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| {
            help::render(f, f.area(), &CATPPUCCIN_MOCHA, "3.0.0");
        })
        .unwrap();
        dump("help 120x40", &term);
    }

    // 4) Search / Jump modal.
    {
        let backend = TestBackend::new(80, 24);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| {
            search::render(f, f.area(), &CATPPUCCIN_MOCHA, "available");
        })
        .unwrap();
        dump("search 80x24", &term);
    }

    // 4b) Jump modal.
    {
        let backend = TestBackend::new(80, 24);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| {
            jump::render(f, f.area(), &CATPPUCCIN_MOCHA, ".regions.us-east-1");
        })
        .unwrap();
        dump("jump 80x24", &term);
    }

    // 5) Full app: navigator + inspector at 120x40.
    {
        let backend = TestBackend::new(120, 40);
        let mut term = Terminal::new(backend).unwrap();
        let store = load(&path);
        let mut nav = ColumnNavigator::new(store);
        // Walk into a container so we actually render multiple columns.
        let first = nav.columns[0]
            .children
            .iter()
            .find(|c| c.is_container())
            .cloned();
        if let Some(first) = first {
            nav.expand_to(first.id, 0);
        }
        term.draw(|f| {
            let (header, breadcrumb, nav_area, hints_area, status) = split_app(f.area());
            nav.render(f, nav_area, &CATPPUCCIN_MOCHA);
            let focused = nav.focused();
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
            f.render_widget(
                Paragraph::new(Line::from(format!(
                    " twig · {} · theme: {} ",
                    filename, CATPPUCCIN_MOCHA.name
                )))
                .style(CATPPUCCIN_MOCHA.surface_style()),
                header,
            );
            twig::tui::widgets::breadcrumbs::render(
                f,
                breadcrumb,
                &CATPPUCCIN_MOCHA,
                focused,
                Some(&nav.store),
            );
            let inspector_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
                .split(
                    Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(2),
                            Constraint::Length(1),
                            Constraint::Min(1),
                            Constraint::Length(1),
                        ])
                        .split(f.area())[2],
                )[1];
            inspector::render(
                f,
                inspector_area,
                &CATPPUCCIN_MOCHA,
                focused,
                Some(&nav.store),
                "json",
            );
            status_bar::render(f, status, &CATPPUCCIN_MOCHA, &path, focused, None);
            hints::render(f, hints_area, &CATPPUCCIN_MOCHA);
        })
        .unwrap();
        dump("full-app 120x40", &term);
    }

    // 5b) Full app with deep navigation (jump to a leaf string).
    {
        let backend = TestBackend::new(120, 40);
        let mut term = Terminal::new(backend).unwrap();
        let store = load(&path);
        let mut nav = ColumnNavigator::new(store);
        if let Ok(Some(target)) = nav.store.resolve_path(".metadata.homepage") {
            nav.expand_to_node(target.id);
        }
        term.draw(|f| {
            let (header, breadcrumb, nav_area, hints_area, status) = split_app(f.area());
            nav.render(f, nav_area, &CATPPUCCIN_MOCHA);
            let focused = nav.focused();
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
            f.render_widget(
                Paragraph::new(Line::from(format!(
                    " twig · {} · theme: {} ",
                    filename, CATPPUCCIN_MOCHA.name
                )))
                .style(CATPPUCCIN_MOCHA.surface_style()),
                header,
            );
            twig::tui::widgets::breadcrumbs::render(
                f,
                breadcrumb,
                &CATPPUCCIN_MOCHA,
                focused,
                Some(&nav.store),
            );
            let inspector_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
                .split(
                    Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(2),
                            Constraint::Length(1),
                            Constraint::Min(1),
                            Constraint::Length(1),
                        ])
                        .split(f.area())[2],
                )[1];
            inspector::render(
                f,
                inspector_area,
                &CATPPUCCIN_MOCHA,
                focused,
                Some(&nav.store),
                "json",
            );
            status_bar::render(f, status, &CATPPUCCIN_MOCHA, &path, focused, None);
            hints::render(f, hints_area, &CATPPUCCIN_MOCHA);
        })
        .unwrap();
        dump("full-app deep 120x40", &term);
    }

    // 6) Full app at 80x24 (small terminal).
    {
        let backend = TestBackend::new(80, 24);
        let mut term = Terminal::new(backend).unwrap();
        let store = load(&path);
        let mut nav = ColumnNavigator::new(store);
        let first = nav.columns[0]
            .children
            .iter()
            .find(|c| c.is_container())
            .cloned();
        if let Some(first) = first {
            nav.expand_to(first.id, 0);
        }
        term.draw(|f| {
            let (header, breadcrumb, nav_area, hints_area, status) = split_app(f.area());
            nav.render(f, nav_area, &CATPPUCCIN_MOCHA);
            let focused = nav.focused();
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
            f.render_widget(
                Paragraph::new(Line::from(format!(
                    " twig · {} · theme: {} ",
                    filename, CATPPUCCIN_MOCHA.name
                )))
                .style(CATPPUCCIN_MOCHA.surface_style()),
                header,
            );
            twig::tui::widgets::breadcrumbs::render(
                f,
                breadcrumb,
                &CATPPUCCIN_MOCHA,
                focused,
                Some(&nav.store),
            );
            let inspector_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
                .split(
                    Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(2),
                            Constraint::Length(1),
                            Constraint::Min(1),
                            Constraint::Length(1),
                        ])
                        .split(f.area())[2],
                )[1];
            inspector::render(
                f,
                inspector_area,
                &CATPPUCCIN_MOCHA,
                focused,
                Some(&nav.store),
                "json",
            );
            status_bar::render(f, status, &CATPPUCCIN_MOCHA, &path, focused, None);
            hints::render(f, hints_area, &CATPPUCCIN_MOCHA);
        })
        .unwrap();
        dump("full-app 80x24", &term);
    }
}
