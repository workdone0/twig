//! The native and browser adapters must expose the same document semantics.
use twig::{
    adapters::{json_loader::JsonLoader, loader::Loader, yaml_loader::YamlLoader},
    core::store::Store,
};
use twig_core::{
    memory::MemoryStore,
    parser,
    storage::{self, NodeStore},
};

fn compare(native: &Store, memory: &MemoryStore, paths: &[&str]) {
    for path in paths {
        let n = storage::resolve_path(native, path).unwrap().unwrap();
        let m = storage::resolve_path(memory, path).unwrap().unwrap();
        assert_eq!(n.path, m.path);
        assert_eq!(
            storage::export_value(native, n.id).unwrap(),
            storage::export_value(memory, m.id).unwrap()
        );
        assert_eq!(
            storage::preview_value(native, n.id, 4, &mut 200).unwrap(),
            storage::preview_value(memory, m.id, 4, &mut 200).unwrap()
        );
        for offset in [0, 1, 256] {
            assert_eq!(
                native
                    .children(n.id, offset, 256)
                    .unwrap()
                    .iter()
                    .map(|n| &n.key)
                    .collect::<Vec<_>>(),
                memory
                    .children(m.id, offset, 256)
                    .unwrap()
                    .iter()
                    .map(|n| &n.key)
                    .collect::<Vec<_>>()
            );
        }
    }
    for query in [
        "",
        "a",
        "%",
        "_",
        "APPLE",
        "é",
        "É",
        "18446744073709551615",
        "no match",
    ] {
        for direction in [-1, 1] {
            let mut a = None;
            let mut b = None;
            for _ in 0..8 {
                let n = native.search(query, a, direction).unwrap();
                let m = memory.search(query, b, direction).unwrap();
                assert_eq!(n.as_ref().map(|n| &n.path), m.as_ref().map(|n| &n.path));
                a = n.map(|n| n.id);
                b = m.map(|n| n.id);
            }
        }
    }
}
#[test]
fn stores_agree_on_json_and_yaml() {
    for (format, text, paths) in [
        (
            "json",
            r#"{"a.b":[1,{"apple":"%_é"},false,null],"big":18446744073709551615,"":"empty","é":"É","a":{"b":2}}"#,
            vec![".", ".[\"a.b\"][1]", ".big", ".[\"\"]"],
        ),
        (
            "yaml",
            "kind: apple\nn: 18446744073709551615\n---\nlist: [1, true, null]\n",
            vec![".", ".kind", ".[1].list"],
        ),
        ("json", "18446744073709551615", vec!["."]),
        ("json", "{}", vec!["."]),
    ] {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join(format!("data.{format}"));
        std::fs::write(&file, text).unwrap();
        let native = if format == "yaml" {
            YamlLoader::new()
                .with_cache_dir(tmp.path().into())
                .load(&file, true)
                .unwrap()
        } else {
            JsonLoader::new()
                .with_cache_dir(tmp.path().into())
                .load(&file, true)
                .unwrap()
        };
        let mut memory = MemoryStore::default();
        let mut input = text.as_bytes();
        if format == "yaml" {
            parser::parse_yaml(&mut input, &mut memory, &()).unwrap();
        } else {
            parser::parse_json(&mut input, &mut memory, &()).unwrap();
        }
        compare(&native, &memory, &paths);
    }
}
#[test]
fn invalid_inputs_are_rejected_by_both() {
    for text in ["", "{} {}", "{\"a\":1,\"a\":2}", "[1,", &"[".repeat(130)] {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("bad.json");
        std::fs::write(&file, text).unwrap();
        assert!(JsonLoader::new()
            .with_cache_dir(tmp.path().into())
            .load(&file, true)
            .is_err());
        assert!(
            parser::parse_json(&mut text.as_bytes(), &mut MemoryStore::default(), &()).is_err()
        );
    }
}
#[test]
fn pagination_and_export_limits_agree() {
    let text = format!("[{}]", vec!["0"; 10_001].join(","));
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("wide.json");
    std::fs::write(&file, &text).unwrap();
    let native = JsonLoader::new()
        .with_cache_dir(tmp.path().into())
        .load(&file, true)
        .unwrap();
    let mut memory = MemoryStore::default();
    parser::parse_json(&mut text.as_bytes(), &mut memory, &()).unwrap();
    for offset in [0, 256, 9984] {
        assert_eq!(
            native
                .children(native.root_id.unwrap(), offset, 256)
                .unwrap()
                .iter()
                .map(|n| n.rank)
                .collect::<Vec<_>>(),
            memory
                .children(memory.root().unwrap(), offset, 256)
                .unwrap()
                .iter()
                .map(|n| n.rank)
                .collect::<Vec<_>>()
        );
    }
    assert!(storage::export_value(&native, native.root_id.unwrap()).is_err());
    assert!(storage::export_value(&memory, memory.root().unwrap()).is_err());
}
#[test]
fn browser_budgets_reject_dense_input() {
    let text = format!("[{}]", vec!["0"; twig_core::memory::MAX_NODES].join(","));
    let error =
        parser::parse_json(&mut text.as_bytes(), &mut MemoryStore::default(), &()).unwrap_err();
    assert!(error.to_string().contains("resource limit"));
}
