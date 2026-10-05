use serde_json::{json, Value};
use std::{
    path::Path,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use twig::adapters::{
    json_loader::JsonLoader,
    loader::{LoadOptions, Loader},
    yaml_loader::YamlLoader,
};

fn roundtrip(value: &Value) -> Value {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("input.json");
    std::fs::write(&file, value.to_string()).unwrap();
    let store = JsonLoader::new()
        .with_cache_dir(tmp.path().join("cache"))
        .load(&file, true)
        .unwrap();
    store
        .reconstruct_value(store.root_id.unwrap(), 130)
        .unwrap()
}
#[test]
fn preserves_order_types_and_special_keys() {
    for v in [
        json!([1,2,{},3,[],{"x":[]}]),
        json!({"n":u64::MAX,"negative":i64::MIN,"f":1.5,"s":"😀"}),
        json!({"a.b":1,"a":{"b":2},"":3,"[0]":4}),
    ] {
        assert_eq!(roundtrip(&v), v);
    }
}
#[test]
fn caches_validate_content_and_reject_partial_and_missing_input() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("input.json");
    let loader = JsonLoader::new().with_cache_dir(tmp.path().join("cache"));
    std::fs::write(&file, "{\"old\":1}").unwrap();
    let original = loader.load(&file, false).unwrap();
    std::fs::write(&file, "{\"new\":2}").unwrap();
    let changed = loader.load(&file, false).unwrap();
    assert!(changed.resolve_path(".new").unwrap().is_some());
    assert!(original.resolve_path(".old").unwrap().is_some());
    let invalid = json!((0..10001).collect::<Vec<_>>()).to_string() + "\n{";
    std::fs::write(&file, invalid).unwrap();
    assert!(loader.load(&file, true).is_err());
    assert!(loader.load(&file, false).is_err());
    std::fs::remove_file(&file).unwrap();
    assert!(loader.load(&file, false).is_err());
}
#[test]
fn concurrent_builders_and_corrupt_cache_recover() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("input.json");
    let cache = tmp.path().join("cache");
    std::fs::write(&file, "[1,2,3]").unwrap();
    std::fs::create_dir(&cache).unwrap();
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let file = file.clone();
            let cache = cache.clone();
            std::thread::spawn(move || {
                JsonLoader::new()
                    .with_cache_dir(cache)
                    .load(&file, false)
                    .unwrap()
                    .node_count()
                    .unwrap()
            })
        })
        .collect();
    for h in handles {
        assert_eq!(h.join().unwrap(), 4);
    }
    for e in std::fs::read_dir(&cache).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|e| e == "db") {
            std::fs::write(p, "garbage").unwrap();
        }
    }
    assert_eq!(
        JsonLoader::new()
            .with_cache_dir(cache)
            .load(&file, false)
            .unwrap()
            .node_count()
            .unwrap(),
        4
    );
}
#[test]
fn cancellation_and_no_cache() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("a.json");
    std::fs::write(&file, "{}").unwrap();
    let flag = Arc::new(AtomicBool::new(true));
    let cache = tmp.path().join("unused");
    let loader = JsonLoader::with_options(LoadOptions {
        cancelled: flag.clone(),
        no_cache: true,
        cache_dir: Some(cache.clone()),
        ..Default::default()
    });
    assert!(loader.load(&file, false).is_err());
    flag.store(false, Ordering::Relaxed);
    let store = loader.load(&file, false).unwrap();
    assert_eq!(store.node_count().unwrap(), 1);
    assert!(!cache.exists());
}
#[test]
fn yaml_document_order_and_quoted_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("a.yaml");
    std::fs::write(&file, "a.b: 1\na:\n  b: 2\n---\n- 1\n- {}\n- 2\n").unwrap();
    let store = YamlLoader::new()
        .with_cache_dir(tmp.path().join("cache"))
        .load(&file, true)
        .unwrap();
    assert_eq!(
        store.resolve_path(".[0][\"a.b\"]").unwrap().unwrap().value,
        Some(json!(1))
    );
    assert_eq!(
        store
            .reconstruct_value(store.root_id.unwrap(), 130)
            .unwrap(),
        json!([{"a.b":1,"a":{"b":2}},[1,{},2]])
    );
}
#[test]
fn multi_root_and_duplicate_keys_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("a.json");
    let loader = JsonLoader::new().with_cache_dir(tmp.path().join("cache"));
    for input in ["{} {}", "{\"a\":1,\"a\":2}"] {
        std::fs::write(&file, input).unwrap();
        assert!(loader.load(&file, true).is_err());
    }
}
#[test]
fn search_literals_document_order_and_paginated_navigation() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("a.json");
    let items: Vec<_> = (0..1000).map(|i| format!("item {i} %_")).collect();
    std::fs::write(&file, json!(items).to_string()).unwrap();
    let store = JsonLoader::new()
        .with_cache_dir(tmp.path().join("cache"))
        .load(&file, true)
        .unwrap();
    assert_eq!(store.get_search_stats("%_", None).unwrap().1, 1000);
    assert_eq!(store.get_search_stats("missing%", None).unwrap().1, 0);
    let a = store.resolve_path(".[9]").unwrap().unwrap();
    let b = store
        .find_next_node("item", Some(a.id), 1)
        .unwrap()
        .unwrap();
    assert_eq!(b.path, ".[10]");
    let mut nav = twig::tui::widgets::navigator::ColumnNavigator::new(store);
    assert!(nav.columns[0].children.len() <= 256);
    for _ in 0..300 {
        nav.move_down();
    }
    assert_eq!(nav.focused().unwrap().key, "300");
    nav.last();
    assert_eq!(nav.focused().unwrap().key, "999");
    nav.first();
    assert_eq!(nav.focused().unwrap().key, "0");
    let id = nav.store.resolve_path(".[700]").unwrap().unwrap().id;
    nav.expand_to_node(id);
    assert_eq!(nav.focused().unwrap().key, "700");
}
#[test]
fn unicode_rendering_and_scalar_roots() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("a.json");
    for input in [
        json!({"İ😀":"a😀😀😀😀😀😀😀","long":"界".repeat(100)}),
        json!(42),
    ] {
        std::fs::write(&file, input.to_string()).unwrap();
        let store = JsonLoader::new()
            .with_cache_dir(tmp.path().join("cache"))
            .load(&file, true)
            .unwrap();
        let mut nav = twig::tui::widgets::navigator::ColumnNavigator::new(store);
        nav.set_search(Some("İ😀".into()));
        assert!(nav.focused().is_some());
        for (w, h) in [(80, 24), (1, 1), (0, 0), (20, 5)] {
            let mut terminal =
                ratatui::Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
            terminal
                .draw(|f| nav.render(f, f.area(), &twig::tui::theme::CATPPUCCIN_MOCHA))
                .unwrap();
        }
    }
}
fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_twig"))
}
#[test]
fn cli_formats_repairs_and_writes_consistently() {
    let tmp = tempfile::tempdir().unwrap();
    let input = tmp.path().join("bad.json");
    let out = tmp.path().join("out.json");
    std::fs::write(&input, "{'a': {'b':1,},}").unwrap();
    let result = binary()
        .args(["--fix", "--print", "--indent", "4"])
        .arg(&input)
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stdout).contains("    \"a\""));
    let saved = binary()
        .args(["--fix", "--print", "--indent", "4", "-o"])
        .arg(&out)
        .arg(&input)
        .output()
        .unwrap();
    assert!(saved.status.success());
    assert_eq!(std::fs::read(out).unwrap(), result.stdout);
    assert!(
        binary()
            .args(["--check", "--print"])
            .arg(&input)
            .output()
            .unwrap()
            .status
            .code()
            == Some(2)
    );
    assert!(binary().arg("-v").output().unwrap().status.success());
}
#[test]
fn cli_yaml_stream_case_and_process_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let input = tmp.path().join("a.YAML");
    std::fs::write(&input, "a: 1\n---\nb: 2\n").unwrap();
    let out = binary().arg("--print").arg(&input).output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("---"));
    assert!(!binary()
        .arg("--fix")
        .arg(&input)
        .output()
        .unwrap()
        .status
        .success());
    assert!(!binary()
        .args(["--check", "--no-cache"])
        .arg(Path::new("/nonexistent/twig-input.json"))
        .output()
        .unwrap()
        .status
        .success());
}

#[test]
fn export_is_complete_or_explicitly_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("data.json");
    let loader = JsonLoader::new().with_cache_dir(tmp.path().join("cache"));
    let value = json!({"a":{"b":{"c":{"d":{"e":{"f":{"g":42}}}}}}});
    std::fs::write(&file, value.to_string()).unwrap();
    let store = loader.load(&file, true).unwrap();
    assert_eq!(store.export_value(store.root_id.unwrap()).unwrap(), value);
    drop(store);
    std::fs::write(&file, json!((0..10001).collect::<Vec<_>>()).to_string()).unwrap();
    let store = loader.load(&file, true).unwrap();
    assert!(store
        .export_value(store.root_id.unwrap())
        .unwrap_err()
        .to_string()
        .contains("10,000"));
    let preview = store
        .preview_value(store.root_id.unwrap(), 4, &mut 200)
        .unwrap();
    assert!(preview.as_array().unwrap().len() <= 30);
}

#[test]
fn yaml_complex_keys_and_non_finite_values_fail_explicitly() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("data.yaml");
    let loader = YamlLoader::new().with_cache_dir(tmp.path().join("cache"));
    for input in ["? [a,b]\n: 1", "1: x", "x: .inf", "x: !custom value"] {
        std::fs::write(&file, input).unwrap();
        assert!(loader.load(&file, true).is_err(), "accepted {input}");
    }
}

#[test]
fn cli_stdin_and_atomic_failure_preserve_destination() {
    use std::io::Write;
    let mut child = binary()
        .args(["--print", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"{\"n\":1}").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&out.stdout).unwrap(),
        json!({"n":1})
    );
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("bad.json");
    let out = tmp.path().join("out.json");
    std::fs::write(&file, "{").unwrap();
    std::fs::write(&out, "preserve me").unwrap();
    assert!(!binary()
        .arg("--print")
        .arg(&file)
        .arg("-o")
        .arg(&out)
        .output()
        .unwrap()
        .status
        .success());
    assert_eq!(std::fs::read_to_string(out).unwrap(), "preserve me");
}
