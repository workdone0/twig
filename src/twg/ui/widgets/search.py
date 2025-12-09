from textual.app import ComposeResult
from textual.screen import ModalScreen
from textual.widgets import Input, Label
from textual.containers import Vertical, Container


class SearchModal(ModalScreen[str]):
    """Modal screen for entering search query."""

    CSS = """
    SearchModal {
        align: center middle;
    }

    #search-container {
        width: 60;
        height: auto;
        background: $surface;
        border: thick $primary;
        padding: 1 2;
        align: center middle;
    }

    #search-label {
        width: 100%;
        text-align: center;
        margin-bottom: 1;
        text-style: bold;
    }

    #search-input {
        margin-bottom: 1;
    }

    #search-help {
        width: 100%;
        text-align: center;
        color: $text-muted;
    }
    """

    def compose(self) -> ComposeResult:
        with Container(id="search-container"):
            yield Label("Search keys & values:", id="search-label")
            yield Input(placeholder="Type to search...", id="search-input")
            yield Label("(Press Esc to cancel)", id="search-help")

    def on_input_submitted(self, message: Input.Submitted) -> None:
        if message.value:
            self.dismiss(message.value)
        else:
            self.dismiss(None)
            
    def key_escape(self) -> None:
        self.dismiss(None)
