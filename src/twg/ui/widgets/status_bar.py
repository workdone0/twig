from textual.app import ComposeResult
from textual.containers import Horizontal
from textual.widgets import Static, Label
from textual.reactive import reactive
import os
from twg.core.model import Node, DataType

class StatusBar(Horizontal):
    """
    A persistent status bar displaying file info and selection context.
    """
    
    selected_node: reactive[Node | None] = reactive(None)

    def __init__(self, file_path: str, **kwargs):
        self.file_path = file_path
        super().__init__(**kwargs)

    def compose(self) -> ComposeResult:
        # File Info (Left)
        try:
            size_bytes = os.path.getsize(self.file_path)
            if size_bytes < 1024:
                size_str = f"{size_bytes} B"
            elif size_bytes < 1024 * 1024:
                size_str = f"{size_bytes / 1024:.1f} KB"
            else:
                size_str = f"{size_bytes / (1024 * 1024):.1f} MB"
        except OSError:
            size_str = "?"

        filename = os.path.basename(self.file_path)
        yield Static(f"📄 {filename} ({size_str})", id="sb-file")
        
        # Context (Center/Remaining)
        yield Static("", id="sb-context")
        
        # Mode (Right)
        yield Static("READ ONLY", id="sb-mode")

    def watch_selected_node(self, node: Node | None) -> None:
        context = self.query_one("#sb-context", Static)
        
        if node is None:
            context.update("")
            return
            
        # Format: "Key (Type)"
        key = node.key if node.key else "root"
        type_str = node.type.value.capitalize()
        
        info = f"{key} : {type_str}"
        
        if node.is_container:
            if isinstance(node.raw_value, (list, dict)):
                count = len(node.raw_value)
                info += f" [{count} items]"
            elif node.children:
                info += f" [{len(node.children)} items]"
                
        context.update(info)
