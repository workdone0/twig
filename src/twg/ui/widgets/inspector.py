from textual.app import ComposeResult
from textual.containers import Vertical, Container
from textual.widgets import Static, Label
from textual.reactive import reactive
from twg.core.model import Node, DataType, TwigModel

class Inspector(Container):
    """
    Displays detailed information about the selected node.
    
    Shows the key, type, value, and any smart-detected information (e.g., ISO dates).
    """
    # CSS moved to styles.tcss
    
    selected_node: reactive[Node | None] = reactive(None)

    def __init__(self, model: TwigModel, **kwargs):
        self.model = model
        super().__init__(**kwargs)

    def compose(self) -> ComposeResult:
        yield Label("Inspector", id="inspector-title")
        yield Static("Select a node to view details", id="inspector-content")

    def watch_selected_node(self, node: Node | None) -> None:
        content_widget = self.query_one("#inspector-content", Static)
        
        if node is None:
            content_widget.update("Select a node to view details")
            return
        
        # 2. Smart Type Detection & Formatting
        key_display = node.key if node.key else "(root)"
        type_display = node.type.value
        value_display = str(node.value)
        
        extra_info = ""

        if node.is_container:
            value_display = f"<{len(node.children)} items>"
        elif node.type == DataType.STRING:
            # Check for Date
            try:
                # Basic ISO check (can be improved)
                from datetime import datetime
                dt = datetime.fromisoformat(node.value.replace('Z', '+00:00'))
                type_display = "Date (ISO8601)"
                extra_info = f"\n[b]Parsed:[/b] {dt.strftime('%Y-%m-%d %H:%M:%S')}"
            except ValueError:
                pass
            
            # Check for URL
            if node.value.startswith("http://") or node.value.startswith("https://"):
                type_display = "URL"

        content = f"""
[b]Key:[/b] {key_display}
[b]Type:[/b] {type_display}

[b]Value:[/b]
{value_display}
{extra_info}
        """
        content_widget.update(content)
