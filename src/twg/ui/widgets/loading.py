from textual.app import ComposeResult
from textual.screen import ModalScreen
from textual.widgets import Label, LoadingIndicator
from textual.containers import Container, Center, Middle

class LoadingScreen(ModalScreen):
    """A modal screen that shows a loading indicator."""

    CSS = """
    LoadingScreen {
        align: center middle;
    }

    #loading-container {
        width: 40;
        height: auto;
        background: $surface;
        border: thick $primary;
        padding: 2;
    }
    
    #loading-label {
        margin-top: 1;
        text-align: center;
        text-style: bold;
    }
    """

    def __init__(self, message: str = "Working...", name: str | None = None, id: str | None = None, classes: str | None = None):
        self.message = message
        super().__init__(name, id, classes)

    def compose(self) -> ComposeResult:
        with Container(id="loading-container"):
            yield Center(Middle(LoadingIndicator()))
            yield Label(self.message, id="loading-label")
