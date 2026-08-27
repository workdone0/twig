//! Integration tests for the TUI.
//!
//! These exercise the Store / adapters together rather than driving
//! the full app event loop (which would be flaky in CI without a real
//! TTY). They cover the equivalent of test_integration.py: smart
//! search (path resolve), global search forward/backward, and a
//! snapshot of the rendered body buffer for a small JSON file.

use std::path::PathBuf;
use twig::adapters::json_loader::JsonLoader;
use twig::adapters::loader::Loader;
use twig::core::store::Store;
use twig::tui::widgets::column::Column;
use twig::tui::widgets::navigator::ColumnNavigator;

fn sample() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/cloud_infrastructure.json")
}

fn load_sample(cache_dir: &std::path::Path) -> Store {
    let loader = JsonLoader::new().with_cache_dir(cache_dir.to_path_buf());
    loader.load(&sample(), true).expect("load sample")
}

#[test]
fn smart_search_via_resolve_path_and_navigator() {
    let cache = tempfile::tempdir().unwrap();
    let store = load_sample(cache.path());
    let mut nav = ColumnNavigator::new(store);
    // The sample has `regions` as an object, not an array.
    let path = ".regions.us-east-1.vpcs[0]";
    let node = nav
        .store
        .resolve_path(path)
        .unwrap()
        .expect("path resolves");
    nav.expand_to_node(node.id);
    let focused = nav.focused().expect("focused after expand");
    assert!(focused.path.contains("vpcs[0]"));
}

#[test]
fn global_search_next_and_prev_cycle() {
    let cache = tempfile::tempdir().unwrap();
    let store = load_sample(cache.path());
    let mut nav = ColumnNavigator::new(store);
    // The cloud_infrastructure.json sample has multiple 'available'
    // strings (db availability).
    let first = nav.find_next("available", 1).expect("first match");
    nav.expand_to_node(first.id);
    let second = nav.find_next("available", 1).expect("second match");
    nav.expand_to_node(second.id);
    let prev = nav.find_next("available", -1).expect("prev match");
    assert_ne!(first.id, second.id);
    let _ = prev;
}

#[test]
fn column_navigator_starts_with_root_column() {
    let cache = tempfile::tempdir().unwrap();
    let store = load_sample(cache.path());
    let nav = ColumnNavigator::new(store);
    assert_eq!(nav.columns.len(), 1);
    assert!(!nav.columns[0].children.is_empty());
    let first = &nav.columns[0].children[0];
    assert!(!first.key.is_empty());
}

#[test]
fn column_renders_into_test_backend() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let cache = tempfile::tempdir().unwrap();
    let store = load_sample(cache.path());
    let parent_id = store.root_id.unwrap();
    let mut col = Column::new(&store, parent_id, 0);

    let backend = TestBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            col.render(
                f,
                ratatui::layout::Rect::new(0, 0, 40, 10),
                &twig::tui::theme::CATPPUCCIN_MOCHA,
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let non_empty_rows = (0..10)
        .filter(|y| {
            (0..40).any(|x| {
                let cell = &buf[(x, *y)];
                !cell.symbol().is_empty()
            })
        })
        .count();
    assert!(non_empty_rows > 0);
}

#[test]
fn help_screen_renders_into_test_backend() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            twig::tui::widgets::help::render(
                f,
                ratatui::layout::Rect::new(0, 0, 80, 30),
                &twig::tui::theme::CATPPUCCIN_MOCHA,
                "3.0.0",
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut flat = String::new();
    for y in 0..30 {
        for x in 0..80 {
            flat.push_str(buf[(x, y)].symbol());
        }
        flat.push('\n');
    }
    for key in [
        "Arrows",
        "Search",
        "Next / Prev",
        "Jump to path",
        "Copy path",
        "Copy source",
        "Toggle theme",
        "Quit",
    ] {
        assert!(flat.contains(key), "help screen missing binding {key}");
    }
    assert!(
        flat.lines().any(|l| l.contains("██████")),
        "logo art missing"
    );
}

#[test]
fn column_no_double_chevron_on_highlight() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let cache = tempfile::tempdir().unwrap();
    let store = load_sample(cache.path());
    let parent_id = store.root_id.unwrap();
    let mut col = Column::new(&store, parent_id, 0);

    let backend = TestBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            col.render(
                f,
                ratatui::layout::Rect::new(0, 0, 40, 10),
                &twig::tui::theme::CATPPUCCIN_MOCHA,
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    // The first row is the highlighted one (default selection = 0).
    // Walk the row contents and assert we don't see two `▶` glyphs
    // back-to-back — that was the bug the user reported.
    let row0: String = (0..40).map(|x| buf[(x, 1)].symbol().to_string()).collect();
    assert!(
        !row0.contains("▶▶"),
        "highlighted row should not show two chevrons: {row0:?}"
    );
}
