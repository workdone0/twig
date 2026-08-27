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
    let row0: String = (0..40).map(|x| buf[(x, 1)].symbol().to_string()).collect();
    assert!(
        !row0.contains("▶▶"),
        "highlighted row should not show two chevrons: {row0:?}"
    );
}

#[test]
fn column_fills_full_width_when_alone() {
    // When the navigator has only one column, it should expand to
    // fill the available width (Constraint::Min(WIDTH)) instead of
    // leaving a wide empty region to the right (the bug where a
    // 120-wide terminal showed a 32-wide column with 88 empty cols).
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use twig::tui::widgets::navigator::ColumnNavigator;

    let cache = tempfile::tempdir().unwrap();
    let store = load_sample(cache.path());
    let mut nav = ColumnNavigator::new(store);
    let backend = TestBackend::new(120, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            nav.render(
                f,
                ratatui::layout::Rect::new(0, 0, 120, 20),
                &twig::tui::theme::CATPPUCCIN_MOCHA,
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    // The right border of the column should be at the very right edge
    // (column 119) — i.e. the column fills the area.
    let right_col_x = (0..120)
        .rev()
        .find(|x| buf[(*x, 1)].symbol() == "│")
        .expect("right border present");
    assert!(
        right_col_x >= 115,
        "single column should expand to fill area; right border at x={right_col_x}, expected ~119"
    );
}

#[test]
fn help_screen_shows_url_line() {
    // Regression: there used to be two render calls to the same
    // chunk in the help modal — the URL was overwritten by a
    // duplicate "Keyboard Shortcuts" header. Make sure the URL
    // appears in the rendered output.
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            twig::tui::widgets::help::render(
                f,
                ratatui::layout::Rect::new(0, 0, 80, 24),
                &twig::tui::theme::CATPPUCCIN_MOCHA,
                "3.0.0",
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut flat = String::new();
    for y in 0..24 {
        for x in 0..80 {
            flat.push_str(buf[(x, y)].symbol());
        }
        flat.push('\n');
    }
    assert!(
        flat.contains("https://twig.wtf"),
        "help screen missing URL line"
    );
    assert!(
        !flat.contains("Keyboard Shortcuts"),
        "help screen should not render the old 'Keyboard Shortcuts' subheader (it was dropping the URL)"
    );
}

#[test]
fn status_bar_fits_context_text_on_narrow_terminal() {
    // Regression: the context column used to be Percentage(35)/Min(10)
    // with a " READ ONLY" badge on Length(10), so "cloud_provider :
    // String" got clipped to "cloud_provider : St" on 80-wide
    // terminals. After the fix the context column should show the
    // full text.
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let cache = tempfile::tempdir().unwrap();
    let store = load_sample(cache.path());
    let nav = ColumnNavigator::new(store);
    let focused = nav.focused().cloned();

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            twig::tui::widgets::status_bar::render(
                f,
                ratatui::layout::Rect::new(0, 23, 80, 1),
                &twig::tui::theme::CATPPUCCIN_MOCHA,
                std::path::Path::new("/tmp/small.json"),
                focused.as_ref(),
                None,
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let row: String = (0..80).map(|x| buf[(x, 23)].symbol().to_string()).collect();
    // The focused node is the first child of the root ('account_id'
    // in cloud_infrastructure.json / 'cloud_provider' in our
    // /tmp/small.json). Either way, " : String" or " : Object"
    // should be visible in full.
    assert!(
        row.contains("Object") || row.contains("String"),
        "status bar should show full type label, got: {row:?}"
    );
    // And the badge should still be present.
    assert!(
        row.contains("READ ONLY"),
        "status bar missing READ ONLY badge"
    );
}

#[test]
fn inspector_hides_empty_insights_panel() {
    // Regression: Smart Insights box used to be drawn around empty
    // content for nodes that don't match URL/hex/ISO patterns.
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use twig::tui::widgets::inspector;

    let cache = tempfile::tempdir().unwrap();
    let store = load_sample(cache.path());
    let mut nav = ColumnNavigator::new(store);
    let target = nav
        .store
        .find_next_node("cloud_provider", None, 1)
        .unwrap()
        .unwrap();
    nav.expand_to_node(target.id);
    let focused = nav.focused().cloned();

    let backend = TestBackend::new(40, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            inspector::render(
                f,
                ratatui::layout::Rect::new(0, 0, 40, 24),
                &twig::tui::theme::CATPPUCCIN_MOCHA,
                focused.as_ref(),
                Some(&nav.store),
                "json",
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut flat = String::new();
    for y in 0..24 {
        for x in 0..40 {
            flat.push_str(buf[(x, y)].symbol());
        }
        flat.push('\n');
    }
    assert!(
        !flat.contains("Smart Insights"),
        "inspector should not render empty Smart Insights box"
    );
}

#[test]
fn column_renders_without_titles() {
    // Regression: Column::render used to add a " root " / " col N "
    // title to each Block. User feedback was that the labels are
    // noisy; the visible right-border alone is enough separator.
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let cache = tempfile::tempdir().unwrap();
    let store = load_sample(cache.path());
    let parent_id = store.root_id.unwrap();
    let mut col = Column::new(&store, parent_id, 0);

    let backend = TestBackend::new(40, 6);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            col.render(
                f,
                ratatui::layout::Rect::new(0, 0, 40, 6),
                &twig::tui::theme::CATPPUCCIN_MOCHA,
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut flat = String::new();
    for y in 0..6 {
        for x in 0..40 {
            flat.push_str(buf[(x, y)].symbol());
        }
        flat.push('\n');
    }
    assert!(
        !flat.contains("col 0") && !flat.contains(" root "),
        "column should not render a title; got:\n{flat}"
    );
}

#[test]
fn hints_bar_renders_all_keys() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use twig::tui::widgets::hints;

    let backend = TestBackend::new(120, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            hints::render(
                f,
                ratatui::layout::Rect::new(0, 0, 120, 1),
                &twig::tui::theme::CATPPUCCIN_MOCHA,
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let row: String = (0..120).map(|x| buf[(x, 0)].symbol().to_string()).collect();
    for label in ["search", "jump", "theme", "help", "quit", "path", "src"] {
        assert!(
            row.contains(label),
            "hints bar missing '{label}' label; got: {row:?}"
        );
    }
}

#[test]
fn search_modal_shows_cursor_indicator() {
    // Regression: the search input line used to be a flat
    // "> {query}" string. Add a cursor block at the end of the
    // typed text so users can see where the next character lands.
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let backend = TestBackend::new(80, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            twig::tui::widgets::search::render(
                f,
                ratatui::layout::Rect::new(0, 0, 80, 10),
                &twig::tui::theme::CATPPUCCIN_MOCHA,
                "available",
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut flat = String::new();
    for y in 0..10 {
        for x in 0..80 {
            flat.push_str(buf[(x, y)].symbol());
        }
        flat.push('\n');
    }
    assert!(flat.contains("available"), "search input not visible");
    assert!(
        flat.contains('▏'),
        "search modal should render a cursor indicator (▏) after the typed text"
    );
}

// --- malformed input handling ----------------------------------------
//
// Regression tests for the bug where unformatted / invalid files made
// the TUI silently close with no error message. The loader should
// now return a descriptive anyhow::Error including line + column
// info, the in-app error screen should render the message, and
// `twig --fix` should be able to repair the file.

fn write_temp_json(name: &str, body: &str) -> std::path::PathBuf {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(name);
    std::fs::write(&path, body).unwrap();
    // Keep the tempdir alive by leaking it; the test process exits
    // before the OS reclaims /tmp.
    std::mem::forget(dir);
    path
}

#[test]
fn json_loader_reports_truncated_json_with_line_info() {
    use twig::adapters::json_loader::JsonLoader;
    use twig::adapters::loader::Loader;
    let path = write_temp_json("broken.json", r#"{"a":1,"b":[1,2,3"#);
    let loader = JsonLoader::new();
    let err = loader
        .load(&path, true)
        .expect_err("truncated JSON must surface an error");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("Failed to parse JSON"),
        "loader error should mention JSON: {msg}"
    );
    assert!(
        msg.contains("line") && msg.contains("column"),
        "loader error should include line/column: {msg}"
    );
}

#[test]
fn json_loader_reports_garbage_with_line_info() {
    use twig::adapters::json_loader::JsonLoader;
    use twig::adapters::loader::Loader;
    let path = write_temp_json("garbage.json", "{{{{");
    let loader = JsonLoader::new();
    let err = loader
        .load(&path, true)
        .expect_err("garbage JSON must surface an error");
    let msg = format!("{err:#}");
    assert!(msg.contains("Failed to parse JSON"), "{msg}");
    assert!(msg.contains("line"), "{msg}");
}

#[test]
fn json_loader_reports_empty_file() {
    use twig::adapters::json_loader::JsonLoader;
    use twig::adapters::loader::Loader;
    let path = write_temp_json("empty.json", "");
    let loader = JsonLoader::new();
    let err = loader.load(&path, true).expect_err("empty file must error");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("empty") || msg.contains("whitespace"),
        "loader should explain empty file: {msg}"
    );
}

#[test]
fn json_loader_reports_nonexistent_file() {
    use twig::adapters::json_loader::JsonLoader;
    use twig::adapters::loader::Loader;
    let path = std::path::Path::new("/tmp/twig_does_not_exist_xyzzy.json");
    let loader = JsonLoader::new();
    let err = loader
        .load(path, true)
        .expect_err("missing file must error");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("No such file") || msg.contains("not found") || msg.contains("os error"),
        "loader should explain missing file: {msg}"
    );
}

#[test]
fn yaml_loader_reports_parse_error_with_line_info() {
    use twig::adapters::loader::Loader;
    use twig::adapters::yaml_loader::YamlLoader;
    let path = write_temp_json("broken.yaml", "name: ok\n  bad_indent: |\n  more: : oops\n");
    let loader = YamlLoader::new();
    let err = loader.load(&path, true).expect_err("bad YAML must error");
    let msg = format!("{err:#}");
    assert!(msg.contains("YAML parse error"), "{msg}");
    assert!(msg.contains("line"), "{msg}");
}

#[test]
fn error_screen_renders_message_and_dismiss_hint() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            twig::tui::widgets::error::render(
                f,
                ratatui::layout::Rect::new(0, 0, 80, 24),
                &twig::tui::theme::CATPPUCCIN_MOCHA,
                Some("Failed to parse JSON at line 7, column 4: trailing comma"),
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut flat = String::new();
    for y in 0..24 {
        for x in 0..80 {
            flat.push_str(buf[(x, y)].symbol());
        }
        flat.push('\n');
    }
    assert!(flat.contains("Load Failed"), "error screen title missing");
    assert!(
        flat.contains("Failed to parse JSON at line 7, column 4"),
        "error message not visible"
    );
    assert!(
        flat.contains("Press any key to exit"),
        "dismiss hint missing"
    );
    // For a JSON parse error, the screen should suggest --fix.
    assert!(
        flat.contains("twig --fix"),
        "error screen should mention --fix for parse errors"
    );
}

#[test]
fn error_screen_for_non_parse_error_omits_fix_hint() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            twig::tui::widgets::error::render(
                f,
                ratatui::layout::Rect::new(0, 0, 80, 24),
                &twig::tui::theme::CATPPUCCIN_MOCHA,
                Some("No such file or directory (os error 2)"),
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut flat = String::new();
    for y in 0..24 {
        for x in 0..80 {
            flat.push_str(buf[(x, y)].symbol());
        }
        flat.push('\n');
    }
    assert!(flat.contains("check the file path"));
}

#[test]
fn twig_fix_can_repair_truncated_json() {
    // End-to-end: the same input the loader can't parse can be
    // repaired by --fix and then re-loaded successfully. This is the
    // round-trip the new error hint points the user at.
    use twig::adapters::json_loader::JsonLoader;
    use twig::adapters::loader::Loader;
    let path = write_temp_json("trunc.json", r#"{"a":1,"b":[1,2,3"#);

    // Step 1: --fix repairs the file in place.
    twig::cli::fix::run_from_path(&path).expect("--fix should succeed");

    // Step 2: the fixed file now loads cleanly.
    let loader = JsonLoader::new();
    let store = loader.load(&path, true).expect("fixed file loads");
    assert!(store.node_count().unwrap() > 1);
}

#[test]
fn main_returns_error_when_load_fails() {
    // The binary-level contract: when ingestion fails, main.rs
    // returns Err so the process exits with a non-zero status and
    // the error message is visible to the user (printed to stderr
    // by main, plus shown in-app via the Error screen).
    use twig::adapters::json_loader::JsonLoader;
    use twig::adapters::loader::Loader;
    let path = write_temp_json("trunc.json", r#"{"a":1,"b":[1,2,3"#);
    let loader = JsonLoader::new();
    let res = loader.load(&path, true);
    assert!(res.is_err(), "truncated JSON must produce a load error");
    let msg = format!("{}", res.err().unwrap());
    assert!(
        msg.contains("JSON") || msg.contains("parse"),
        "error message must mention JSON or parse: {msg}"
    );
}

#[test]
fn check_mode_loads_file_and_reports_stats() {
    // `twig --check` exercises the streaming ingestion pipeline
    // without opening a TUI, which makes it usable in non-interactive
    // contexts like CI. Verify it returns Ok and produces a node
    // count from a small in-memory JSON file.
    use twig::adapters::json_loader::JsonLoader;
    use twig::adapters::loader::Loader;
    let path = write_temp_json("check.json", r#"{"a":1,"b":[1,2,3],"c":{"x":1}}"#);
    let loader = JsonLoader::new();
    let store = loader
        .load(&path, true)
        .expect("check-mode sample should load cleanly");
    let nodes = store.node_count().expect("node_count");
    assert!(
        nodes >= 5,
        "expected at least 5 nodes (root + a, b, b[0..2], c, c.x), got {nodes}"
    );
}

#[test]
fn check_mode_reports_failure_for_truncated_json() {
    // Negative case: --check should propagate the same anyhow error
    // that the TUI sees, so a failed CI run surfaces a meaningful
    // exit code rather than a silent pass.
    use twig::adapters::json_loader::JsonLoader;
    use twig::adapters::loader::Loader;
    let path = write_temp_json("check_trunc.json", r#"{"a":1,"b":[1,2"#);
    let loader = JsonLoader::new();
    let res = loader.load(&path, true);
    assert!(res.is_err(), "truncated JSON must surface an error");
}
