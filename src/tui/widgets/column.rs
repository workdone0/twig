//! Single Miller column showing the children of one parent node.
//!
//! A paginated vertical list of `▶ {key}: {value_preview}` items, with optional
//! highlighted search-match substrings.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;
use uuid::Uuid;

use crate::core::model::{DataType, Node};
use crate::core::store::Store;
use crate::tui::styles::{highlighted, search_match};
use crate::tui::theme::Theme;

pub const COLUMN_WIDTH: u16 = 32;

#[derive(Debug)]
pub struct Column {
    pub parent_id: Uuid,
    pub index: usize,
    pub children: Vec<Node>,
    pub total: usize,
    pub offset: usize,
    pub state: ListState,
    pub last_query: Option<String>,
}

impl Column {
    pub const WIDTH: u16 = COLUMN_WIDTH;

    pub fn new(store: &Store, parent_id: Uuid, index: usize) -> Self {
        let mut children = store
            .get_children_page(parent_id, 0, 256)
            .unwrap_or_default();
        let mut total = store.get_children_count(parent_id).unwrap_or(0) as usize;
        if let Ok(Some(node)) = store.get_node(parent_id) {
            if !node.is_container() {
                children = vec![node];
                total = 1;
            }
        }
        let mut state = ListState::default();
        if !children.is_empty() {
            state.select(Some(0));
        }
        Self {
            parent_id,
            index,
            children,
            total,
            offset: 0,
            state,
            last_query: None,
        }
    }

    pub fn set_search(&mut self, query: Option<String>) {
        self.last_query = query;
    }

    pub fn selected(&self) -> Option<&Node> {
        self.state.selected().and_then(|i| self.children.get(i))
    }

    pub fn selected_id(&self) -> Option<Uuid> {
        self.selected().map(|n| n.id)
    }

    pub fn select_index(&mut self, idx: usize) {
        if idx < self.children.len() {
            self.state.select(Some(idx));
        }
    }

    pub fn position(&self) -> usize {
        self.offset + self.state.selected().unwrap_or(0)
    }

    pub fn select_position(&mut self, store: &Store, position: usize) {
        if position >= self.total {
            return;
        }
        if position < self.offset || position >= self.offset + self.children.len() {
            self.offset = (position / 256) * 256;
            self.children = store
                .get_children_page(self.parent_id, self.offset, 256)
                .unwrap_or_default();
            self.state = ListState::default();
        }
        self.state.select(Some(position - self.offset));
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        let items: Vec<ListItem> = self
            .children
            .iter()
            .map(|c| build_item(c, self.last_query.as_deref(), theme))
            .collect();
        // Pad the column one row from the top so the first row isn't
        // pinned against the breadcrumbs bar above (which is what
        // the user reported as "cramped"). The right border stays
        // to visually separate adjacent columns.
        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(theme.secondary))
            .padding(ratatui::widgets::Padding::new(0, 0, 1, 0));
        // Selection is communicated purely through the highlight style
        // (bg + bold). No `highlight_symbol` here — using one together
        // with the `▶` icon in `build_item` produced a double chevron
        // on highlighted containers.
        let list = List::new(items)
            .block(block)
            .highlight_style(highlighted(theme));
        f.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn parent_id(&self) -> Uuid {
        self.parent_id
    }
}

fn build_item(node: &Node, query: Option<&str>, theme: &Theme) -> ListItem<'static> {
    let icon = match node.ty {
        DataType::Object | DataType::Array => "▶",
        _ => " ",
    };
    let mut preview = String::new();
    if !node.is_container() {
        preview = format!(": {}", value_preview(node));
    }
    let raw = format!("{icon} {}{preview}", node.key);

    let line = if let Some(q) = query {
        Line::from(highlight_spans(&raw, q, theme))
    } else {
        Line::from(raw)
    };
    ListItem::new(line)
}

fn value_preview(node: &Node) -> String {
    let raw = match &node.value {
        Some(v) => v.to_string(),
        None => String::new(),
    };
    super::text::truncate(&raw, 24)
}

fn highlight_spans(text: &str, query: &str, theme: &Theme) -> Vec<Span<'static>> {
    if query.is_empty() {
        return vec![Span::raw(text.to_string())];
    }
    let mut spans: Vec<Span<'static>> = Vec::new();
    let lower = text.to_ascii_lowercase();
    let q = query.to_ascii_lowercase();
    let mut start = 0;
    while let Some(pos) = lower[start..].find(&q) {
        let abs = start + pos;
        if abs > start {
            spans.push(Span::raw(text[start..abs].to_string()));
        }
        let end = abs + q.len();
        spans.push(Span::styled(
            text[abs..end].to_string(),
            search_match(theme),
        ));
        start = end;
    }
    if start < text.len() {
        spans.push(Span::raw(text[start..].to_string()));
    }
    spans.push(Span::styled(
        String::new(),
        Style::default().add_modifier(Modifier::DIM),
    ));
    spans
}
