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
            Static(id="insp-header"),
            Static(id="insp-path"),
            Static(id="insp-details"),
            Static(id="insp-content"),
            id="inspector-scroll"
        )

    def watch_selected_node(self, node: Node | None) -> None:
        header = self.query_one("#insp-header", Static)
        path_view = self.query_one("#insp-path", Static)
        details = self.query_one("#insp-details", Static)
        content = self.query_one("#insp-content", Static)
        
        if node is None:
            header.update(Text("No Selection", style="dim"))
            path_view.update("")
            details.update("")
            content.update("")
            return
        
        # 1. Header
        key_text = Text(node.key if node.key else "root", style="bold cyan")
        header.update(key_text)

        # 2. Human Readable Path
        # Traverse up to build path: root > users > 0 > name
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
        path_view.update(Text(human_path, style="dim"))

        # 3. Details Table
        # Type | Size | ID?
        type_str = node.type.value.capitalize()
        size_str = "-"
        
        if node.type == DataType.ARRAY:
             # If exact bucket count logic is complex, just show len(children) or raw len
             if isinstance(node.raw_value, list):
                 size_str = f"{len(node.raw_value)} items"
        elif node.type == DataType.OBJECT:
             if isinstance(node.raw_value, dict):
                 size_str = f"{len(node.raw_value)} keys"
        elif node.type == DataType.STRING:
             size_str = f"{len(str(node.value))} chars"

        table = Table(show_header=False, box=None, padding=(0, 2))
        table.add_row(Text("Type", style="bold"), Text(type_str, style="green"))
        table.add_row(Text("Size", style="bold"), size_str)
        
        # Smart Insight: Date
        if node.type == DataType.STRING:
            try:
                # Basic ISO parsing
                dt = datetime.fromisoformat(node.value.replace('Z', '+00:00'))
                table.add_row(Text("Parsed", style="bold"), Text(dt.strftime('%Y-%m-%d %H:%M'), style="yellow"))
            except ValueError:
                pass
        
        details.update(table)

        # 4. Content / Value
        if node.is_container:
            # Maybe show a preview of first few items?
            # For now just a description
            content.update(Panel(f"Container with {size_str}", title="Value"))
        else:
            # Pretty print value
            val = node.value
            if isinstance(val, (dict, list)):
                # Should not happen for leaf, but just in case
                formatted = json.dumps(val, indent=2)
                syntax = Syntax(formatted, "json", theme="monokai", word_wrap=True)
                content.update(Panel(syntax, title="Value"))
            else:
                content.update(Panel(str(val), title="Value"))
