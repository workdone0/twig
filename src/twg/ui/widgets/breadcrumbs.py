from textual.widgets import Static
from textual.reactive import reactive
from twg.core.model import Node, TwigModel

class Breadcrumbs(Static):
    """
    Displays the path to the currently selected node.
    
    Uses jq-style syntax (e.g., .users[0].name) for the path.
    """
    # CSS moved to styles.tcss

    selected_node: reactive[Node | None] = reactive(None)

    def __init__(self, model: TwigModel, **kwargs):
        self.model = model
        super().__init__(**kwargs)

    def watch_selected_node(self, node: Node | None) -> None:
        if not node:
            self.update("Path: .")
            return
        
        full_path = self.model.get_path(node.id)
        self.update(f"[b]Path:[/b] {full_path} [dim](jq)[/]")
