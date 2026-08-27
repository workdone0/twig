//! Single Miller column showing the children of one parent node.
//!
//! Mirrors `ui/widgets/navigator.py::Column` from the Python version:
//! a vertical list of `▶ {key}: {value_preview}` items, with optional
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
    pub state: ListState,
    pub last_query: Option<String>,
}

impl Column {
    pub const WIDTH: u16 = COLUMN_WIDTH;

    pub fn new(store: &Store, parent_id: Uuid, index: usize) -> Self {
        let children = store.get_children(parent_id).unwrap_or_default();
        let mut state = ListState::default();
        if !children.is_empty() {
            state.select(Some(0));
        }
        Self {
            parent_id,
            index,
            children,
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

    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        let items: Vec<ListItem> = self
            .children
            .iter()
            .map(|c| build_item(c, self.last_query.as_deref(), theme))
            .collect();
        let title = if self.index == 0 {
            " root ".to_string()
        } else {
            format!(" col {} ", self.index)
        };
        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(theme.secondary))
            .title(title);
        let list = List::new(items)
            .block(block)
            .highlight_style(highlighted(theme))
            .highlight_symbol("▶ ");
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
    if raw.len() <= 24 {
        raw
    } else {
        format!("{}…", &raw[..21])
    }
}

fn highlight_spans(text: &str, query: &str, theme: &Theme) -> Vec<Span<'static>> {
    if query.is_empty() {
        return vec![Span::raw(text.to_string())];
    }
    let mut spans: Vec<Span<'static>> = Vec::new();
    let lower = text.to_lowercase();
    let q = query.to_lowercase();
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
