from textual.widgets import Static
from textual.reactive import reactive
from twg.core.model import Node, SQLiteModel

class Breadcrumbs(Static):
    """
    Displays the path to the currently selected node.
    
    Uses jq-style syntax (e.g., .users[0].name) for the path.
    """
    # CSS moved to styles.tcss

    selected_node: reactive[Node | None] = reactive(None)

    def __init__(self, model: SQLiteModel, **kwargs):
        self.model = model
        super().__init__(**kwargs)

    def watch_selected_node(self, node: Node | None) -> None:
        if not node:
            self.update("Path: .")
            return
        
        full_path = self.model.get_path(node.id)
        
        # Smart truncation for long paths
        if len(full_path) > 100:
            parts = full_path.split('.')
            if len(parts) > 6:
                # Keep first 2 and last 3 for better context
                full_path = f"{parts[0]}.{parts[1]} ... {parts[-3]}.{parts[-2]}.{parts[-1]}"
            else:
                # Fallback simple truncation - preserve end
                full_path = "..." + full_path[-97:]

        self.update(f"[b]Path:[/b] {full_path} [dim](jq)[/]")
