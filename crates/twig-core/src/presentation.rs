//! Canonical palettes and normal-mode keys consumed by both interfaces.
use serde::Deserialize;
use std::{collections::BTreeMap, sync::LazyLock};

pub const DEFAULT_THEME: &str = "dark";
pub fn canonical_theme(name: &str) -> &'static str {
    match name {
        "light" => "light",
        // Both legacy themes were dark; preserve that preference.
        _ => "dark",
    }
}
pub static PALETTES: LazyLock<BTreeMap<String, BTreeMap<String, String>>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../themes.json")).expect("valid bundled palettes")
});

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    MoveDown,
    MoveUp,
    Open,
    Back,
    First,
    Last,
    Search,
    Jump,
    NextMatch,
    PreviousMatch,
    CopyPath,
    CopyValue,
    ToggleTheme,
    Help,
    Quit,
}
#[derive(Deserialize)]
pub struct Binding {
    pub action: Action,
    pub keys: Vec<String>,
    pub label: String,
}
pub static BINDINGS: LazyLock<Vec<Binding>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../bindings.json")).expect("valid bundled bindings")
});
pub fn action_for(key: &str) -> Option<Action> {
    BINDINGS
        .iter()
        .find(|binding| binding.keys.iter().any(|k| k == key))
        .map(|binding| binding.action)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn palettes_and_bindings_have_one_unambiguous_contract() {
        assert_eq!(
            PALETTES.keys().map(String::as_str).collect::<Vec<_>>(),
            ["dark", "light"]
        );
        for palette in PALETTES.values() {
            for value in palette.values() {
                assert_eq!(value.len(), 7);
                assert!(u32::from_str_radix(&value[1..], 16).is_ok());
            }
        }
        let mut keys = std::collections::HashSet::new();
        for binding in BINDINGS.iter() {
            for key in &binding.keys {
                assert!(keys.insert(key));
            }
        }
        assert_eq!(action_for("t"), Some(Action::ToggleTheme));
        assert_eq!(action_for("Enter"), Some(Action::Open));
        assert_eq!(canonical_theme("solarized-dark"), "dark");
        assert_eq!(canonical_theme("catppuccin-mocha"), "dark");
        assert_eq!(canonical_theme("light"), "light");
    }
}
