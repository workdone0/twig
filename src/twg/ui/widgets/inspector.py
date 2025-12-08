from textual.app import ComposeResult
from textual.containers import Vertical, Container, Horizontal
from textual.widgets import Static, Label
from textual.reactive import reactive
from rich.text import Text
from rich.table import Table
from rich.panel import Panel
from rich.syntax import Syntax
from datetime import datetime
import json

from twg.core.model import Node, DataType, TwigModel

class Inspector(Container):
    """
    Displays detailed information about the selected node in a rich format.
    """
    
    selected_node: reactive[Node | None] = reactive(None)

    def __init__(self, model: TwigModel, **kwargs):
        self.model = model
        super().__init__(**kwargs)

    def compose(self) -> ComposeResult:
        yield Label("Inspector", id="inspector-title")
        yield Vertical(
            Label("", id="insp-header"),
            Label("", id="insp-path"),
            Container(id="insp-details-grid"),
            Static(id="insp-content"),
            id="inspector-scroll"
        )

    def watch_selected_node(self, node: Node | None) -> None:
        header = self.query_one("#insp-header", Label)
        path_view = self.query_one("#insp-path", Label)
        details_grid = self.query_one("#insp-details-grid", Container)
        content = self.query_one("#insp-content", Static)
        
        # Clear details grid
        details_grid.remove_children()
        
        if node is None:
            header.update("No Selection")
            header.add_class("dim")
            path_view.update("")
            content.update("")
            return
        
        header.remove_class("dim")

        # 1. Header
        key_display = node.key if node.key else "root"
        header.update(key_display)

        # 2. Human Readable Path
        chain = []
        curr = node
        while curr:
            label = curr.key if curr.key else "root"
            chain.append(label)
            if curr.parent:
                curr = self.model.get_node(curr.parent)
            else:
                curr = None
        
        human_path = " › ".join(reversed(chain))
        path_view.update(human_path)

        # 3. Details Grid (Native Widgets)
        type_str = node.type.value.capitalize()
        size_str = "-"
        
        if node.type == DataType.ARRAY:
             if isinstance(node.raw_value, list):
                 size_str = f"{len(node.raw_value)} items"
        elif node.type == DataType.OBJECT:
             if isinstance(node.raw_value, dict):
                 size_str = f"{len(node.raw_value)} keys"
        elif node.type == DataType.STRING:
             size_str = f"{len(str(node.value))} chars"

        # Helper to add row
        def add_detail(label: str, value: str):
            details_grid.mount(Label(label, classes="insp-label"))
            details_grid.mount(Label(value, classes="insp-value"))

        add_detail("Type", type_str)
        add_detail("Size", size_str)
        
        # Smart Insight: Date
        if node.type == DataType.STRING:
            try:
                dt = datetime.fromisoformat(node.value.replace('Z', '+00:00'))
                add_detail("Parsed", dt.strftime('%Y-%m-%d %H:%M'))
            except ValueError:
                pass

        # 4. Content / Value
        if node.is_container:
            content.update(Panel(f"Container with {size_str}", title="Value"))
        else:
            val = node.value
            if isinstance(val, (dict, list)):
                formatted = json.dumps(val, indent=2)
                syntax = Syntax(formatted, "json", theme="monokai", word_wrap=True)
                content.update(Panel(syntax, title="Value"))
            else:
                content.update(Panel(str(val), title="Value"))
