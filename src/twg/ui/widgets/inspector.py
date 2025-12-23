from textual.app import ComposeResult
from textual.containers import Vertical, Container, Horizontal, VerticalScroll
from textual.widgets import Static, Label
from textual.reactive import reactive
from rich.text import Text
from rich.table import Table
from rich.panel import Panel
from rich.syntax import Syntax
from rich.json import JSON
from datetime import datetime
import json
import re

try:
    import yaml
except ImportError:
    yaml = None

from twg.core.model import Node, DataType, SQLiteModel

class Inspector(Container):
    """
    Displays detailed information about the selected node in a rich format.
    Uses a vertical stack layout for better use of vertical space.
    """
    
    selected_node: reactive[Node | None] = reactive(None)
    format: reactive[str] = reactive("json") # "json" or "yaml"
    
    # Regex patterns for smart insights
    URL_PATTERN = re.compile(r'https?://(?:[-\w.]|(?:%[\da-fA-F]{2}))+')
    HEX_COLOR_PATTERN = re.compile(r'^#(?:[0-9a-fA-F]{3}){1,2}$')
    EMAIL_PATTERN = re.compile(r'[^@]+@[^@]+\.[^@]+')

    SYNTAX_THEME = "monokai"

    def __init__(self, model: SQLiteModel, format: str = "json", **kwargs):
        self.model = model
        super().__init__(**kwargs)
        self.format = format

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Label("Inspector", id="inspector-title")
            yield Label("", id="insp-path")
            
            with VerticalScroll(id="inspector-scroll"):
                yield Container(id="insp-details-grid")
                yield Static(id="insp-insights", classes="insp-section")
                yield Static(id="insp-preview_content", classes="insp-section")
                yield Static(id="insp-raw_content", classes="insp-section")

    def watch_selected_node(self, node: Node | None) -> None:
        path_view = self.query_one("#insp-path", Label)
        details_grid = self.query_one("#insp-details-grid", Container)
        insights = self.query_one("#insp-insights", Static)
        preview = self.query_one("#insp-preview_content", Static)
        raw = self.query_one("#insp-raw_content", Static)
        
        # Clear all
        details_grid.remove_children()
        insights.update("")
        preview.update("")
        raw.update("")
        
        if node is None:
            path_view.update("")
            return

        # Path Header
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

        # Details Grid
        type_str = node.type.value.capitalize()
        size_str = "-"
        
        children_count = 0
        if node.type in (DataType.ARRAY, DataType.OBJECT):
             children_count = self.model.get_children_count(node.id)
             size_str = f"{children_count} items"
        elif node.type == DataType.STRING:
             size_str = f"{len(str(node.value))} chars"

        def add_detail(label: str, value: str):
            details_grid.mount(Label(label, classes="insp-label"))
            details_grid.mount(Label(value, classes="insp-value"))

        add_detail("Type", type_str)
        add_detail("Size", size_str)
        
        # Smart Insights 
        insight_text = Text()
        has_insights = False
        
        if node.type == DataType.STRING:
            val_str = str(node.value)
            
            # URL
            if self.URL_PATTERN.match(val_str):
                add_detail("Format", "URL")
                insight_text.append("\nLink detected:\n", style="bold")
                insight_text.append(val_str, style="blue underline link " + val_str)
                has_insights = True
            
            # Color
            elif self.HEX_COLOR_PATTERN.match(val_str):
                add_detail("Format", "Color")
                insight_text.append("\nColor Preview: ", style="bold")
                insight_text.append("██████", style=val_str)
                insight_text.append(f" {val_str}")
                has_insights = True
                
            # Date
            else:
                try:
                    dt = datetime.fromisoformat(val_str.replace('Z', '+00:00'))
                    add_detail("Format", "ISO8601")
                    add_detail("Time", dt.strftime('%Y-%m-%d %H:%M'))
                except ValueError:
                    pass
        
        if has_insights:
             insights.update(Panel(insight_text, title="Smart Insights", border_style="dim"))
        
        # Preview / Raw
        if node.is_container:
            # Preview: Show first 20 items simply
            preview_str = f""
            
            children = self.model.get_children(node.id)
            
            limit = 30
            for i, child in enumerate(children[:limit]): 
                icon = "[+] " if child.is_container else "- "
                val = "..." if child.is_container else str(child.value)
                if len(val) > 50: val = val[:47] + "..."
                preview_str += f"{icon} {child.key}: {val}\n"
            
            if children_count > limit:
                preview_str += f"\n... and {children_count - limit} more."
                
            preview.update(Panel(preview_str, title="Content Preview"))
            
            # Raw View: Attempt to load if size is reasonable
            # 500 items is a safe upper bound for UI responsiveness
            if children_count < 500:
                try:
                    # Depth 4 is enough for inspection without exploding
                    data = self.model.reconstruct_json(node.id, max_depth=4)
                    
                    if self.format == "yaml" and yaml:
                        # Serialize as YAML
                        source_str = yaml.dump(data, sort_keys=False, default_flow_style=False)
                        lexer = "yaml"
                    else:
                        # Serialize as JSON
                        source_str = json.dumps(data, indent=2)
                        lexer = "json"

                    raw.update(Panel(Syntax(source_str, lexer, theme=self.SYNTAX_THEME, padding=1), title="Source"))
                except Exception as e:
                    raw.update(Panel(Text(f"Error loading raw view: {e}", style="red"), title="Source"))
            else:
                raw.update(Panel(Text("Raw view hidden for performance ( > 500 items)", style="italic dim"), title="Source"))
            
        else:
            # Primitive
            preview.update(Panel(str(node.value), title="Value"))
            
            # Raw: Try to parse as JSON if it looks like it, else just syntax highlight the value
            if self.format == "yaml" and yaml:
                # For primitives, YAML is just the value often, or specialized
                 source_str = yaml.dump(node.value, sort_keys=False)
                 lexer = "yaml"
            else:
                 source_str = json.dumps(node.value, indent=2)
                 lexer = "json"

            raw.update(Panel(Syntax(source_str, lexer, theme=self.SYNTAX_THEME, padding=1), title="Source"))
