//! Paginated Miller columns with keyboard/mouse navigation and a horizontal viewport.

use ratatui::layout::Rect;
use ratatui::Frame;
use uuid::Uuid;

use crate::core::model::Node;
use crate::core::store::Store;
use crate::tui::theme::Theme;
use crate::tui::widgets::column::Column;

pub struct ColumnNavigator {
    pub store: Store,
    pub columns: Vec<Column>,
    pub last_query: Option<String>,
    areas: Vec<(usize, Rect)>,
}

impl ColumnNavigator {
    pub fn new(store: Store) -> Self {
        let mut nav = Self {
            store,
            columns: Vec::new(),
            last_query: None,
            areas: Vec::new(),
        };
        if let Some(root) = nav.store.root_id {
            nav.columns.push(Column::new(&nav.store, root, 0));
        }
        nav
    }

    pub fn focused(&self) -> Option<&Node> {
        self.columns.last().and_then(|c| c.selected())
    }

    pub fn focused_id(&self) -> Option<Uuid> {
        self.columns.last().and_then(|c| c.selected_id())
    }

    pub fn set_search(&mut self, query: Option<String>) {
        self.last_query = query.clone();
        for c in &mut self.columns {
            c.set_search(query.clone());
        }
    }

    pub fn expand_to(&mut self, node_id: Uuid, initial_select_index: usize) {
        let next_index = self.columns.len();
        let mut col = Column::new(&self.store, node_id, next_index);
        if initial_select_index < col.children.len() {
            col.state.select(Some(initial_select_index));
        }
        self.columns.push(col);
    }

    pub fn move_down(&mut self) {
        if let Some(col) = self.columns.last_mut() {
            col.select_position(&self.store, col.position().saturating_add(1));
        }
    }
    pub fn move_up(&mut self) {
        if let Some(col) = self.columns.last_mut() {
            col.select_position(&self.store, col.position().saturating_sub(1));
        }
    }
    pub fn first(&mut self) {
        if let Some(col) = self.columns.last_mut() {
            col.select_position(&self.store, 0);
        }
    }
    pub fn last(&mut self) {
        if let Some(col) = self.columns.last_mut() {
            col.select_position(&self.store, col.total.saturating_sub(1));
        }
    }
    pub fn drill(&mut self) {
        let focused = match self.columns.last() {
            Some(c) => c.selected().map(|n| (n.id, n.is_container())),
            None => None,
        };
        if let Some((id, is_container)) = focused {
            if is_container {
                self.expand_to(id, 0);
            }
        }
    }

    pub fn step_back(&mut self) {
        if self.columns.len() > 1 {
            self.columns.pop();
        }
    }

    pub fn expand_to_node(&mut self, target_id: Uuid) {
        let lineage = self.lineage(target_id);
        if lineage.is_empty() {
            return;
        }
        // The deepest column should be the *parent* of the target so
        // the target appears as a highlighted child. If the target is
        // the root we still mount one column (the root itself).
        let mount_ids: Vec<Uuid> = if lineage.len() > 1 {
            lineage[..lineage.len() - 1].to_vec()
        } else {
            lineage.clone()
        };

        self.columns.clear();
        for (i, ancestor_id) in mount_ids.iter().enumerate() {
            let mut col = Column::new(&self.store, *ancestor_id, i);
            // The id to highlight in this column is the next one in the
            // lineage, except for the last column (where it is the
            // target itself).
            let highlight_id = if i + 1 < mount_ids.len() {
                mount_ids[i + 1]
            } else {
                target_id
            };
            if let Ok(Some(target)) = self.store.get_node(highlight_id) {
                col.select_position(&self.store, target.rank as usize);
            }
            self.columns.push(col);
        }
    }

    fn lineage(&self, target: Uuid) -> Vec<Uuid> {
        let mut path = Vec::new();
        let mut current = Some(target);
        while let Some(id) = current {
            path.push(id);
            let node = match self.store.get_node(id).ok().flatten() {
                Some(n) => n,
                None => break,
            };
            current = node.parent;
        }
        path.reverse();
        path
    }

    pub fn find_next(&self, query: &str, direction: i32) -> Option<Node> {
        let start = self.focused_id();
        self.store
            .find_next_node(query, start, direction)
            .ok()
            .flatten()
    }

    pub fn on_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseButton, MouseEventKind};
        match mouse.kind {
            MouseEventKind::ScrollDown => self.move_down(),
            MouseEventKind::ScrollUp => self.move_up(),
            MouseEventKind::Down(MouseButton::Right) => self.step_back(),
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some((index, area)) = self
                    .areas
                    .iter()
                    .find(|(_, a)| a.contains((mouse.column, mouse.row).into()))
                    .copied()
                {
                    if mouse.row <= area.y {
                        return;
                    }
                    self.columns.truncate(index + 1);
                    if let Some(col) = self.columns.last_mut() {
                        let row = (mouse.row - area.y - 1) as usize + col.state.offset();
                        let was_selected = col.state.selected() == Some(row);
                        col.select_index(row);
                        if was_selected && row < col.children.len() {
                            self.drill();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        let visible = (area.width as usize / Column::WIDTH as usize).max(1);
        let start = self.columns.len().saturating_sub(visible);
        let n = self.columns.len() - start;
        if n == 0 {
            return;
        }
        // Each column gets `WIDTH` columns of horizontal space, except
        // the last one which expands to fill whatever remains. This
        // way a single-column navigator fills the available area
        // (instead of leaving a wide empty region to its right), and
        // multi-column navigators still get the full Miller-column
        // scrolling affordance.
        let mut widths: Vec<ratatui::layout::Constraint> = Vec::with_capacity(n);
        for i in 0..n {
            if i + 1 == n {
                widths.push(ratatui::layout::Constraint::Min(Column::WIDTH));
            } else {
                widths.push(ratatui::layout::Constraint::Length(Column::WIDTH));
            }
        }
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(widths)
            .split(area);
        self.areas.clear();
        for (i, col) in self.columns.iter_mut().skip(start).enumerate() {
            if let Some(area) = chunks.get(i) {
                self.areas.push((start + i, *area));
                col.render(f, *area, theme);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::loader::Loader;

    #[test]
    fn navigator_starts_with_root_column() {
        let tmp = tempfile::tempdir().unwrap();
        let json = tmp.path().join("data.json");
        std::fs::write(&json, r#"{"a":1,"b":2}"#).unwrap();
        let loader = crate::adapters::json_loader::JsonLoader::new()
            .with_cache_dir(tmp.path().to_path_buf());
        let store = loader.load(&json, true).unwrap();
        let nav = ColumnNavigator::new(store);
        assert_eq!(nav.columns.len(), 1);
        assert_eq!(nav.columns[0].children.len(), 2);
        assert_eq!(nav.columns[0].children[0].key, "a");
    }

    #[test]
    fn navigator_expands_to_target_node() {
        let tmp = tempfile::tempdir().unwrap();
        let json = tmp.path().join("data.json");
        std::fs::write(&json, r#"{"a":{"n":"nested"},"b":"banana"}"#).unwrap();
        let loader = crate::adapters::json_loader::JsonLoader::new()
            .with_cache_dir(tmp.path().to_path_buf());
        let store = loader.load(&json, true).unwrap();
        let mut nav = ColumnNavigator::new(store);
        let target = nav.store.resolve_path(".a.n").unwrap().unwrap();
        nav.expand_to_node(target.id);
        assert_eq!(nav.columns.len(), 2); // root -> a
        let deepest = nav.columns.last().unwrap();
        assert_eq!(deepest.children[0].key, "n");
        assert_eq!(deepest.state.selected(), Some(0));
    }
}
