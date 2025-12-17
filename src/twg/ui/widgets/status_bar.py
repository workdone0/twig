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
    search_stats: reactive[str | None] = reactive(None)

    def __init__(self, file_path: str, model=None, **kwargs):
        self.file_path = file_path
        self.model = model
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
        yield Static(f"FILE: {filename} ({size_str})", id="sb-file")
        
        yield Static("", id="sb-context")
        
        yield Static("READ ONLY", id="sb-mode")
        
        yield Static("", id="sb-search-stats")

    def watch_selected_node(self, node: Node | None) -> None:
        context = self.query_one("#sb-context", Static)
        
        if node is None:
            context.update("")
            return
            
        # Format: "Key (Type)"
        key = node.key if node.key else "root"
        type_str = node.type.value.capitalize()
        
        info = f"{key} : {type_str}"
        
        if node.is_container and self.model:
            count = self.model.get_children_count(node.id)
            info += f" [{count} items]"
                
        context.update(info)
        
    def watch_search_stats(self, stats: str | None) -> None:
        stats_widget = self.query_one("#sb-search-stats", Static)
        if stats:
            stats_widget.update(f"SEARCH: {stats}")
        else:
            stats_widget.update("")
