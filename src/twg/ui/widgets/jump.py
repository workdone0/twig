from textual.app import ComposeResult
from textual.screen import ModalScreen
from textual.widgets import Input, Label
from textual.containers import Container

class JumpModal(ModalScreen[str]):
    """Modal screen for entering jq path to jump to."""

    CSS = """
    JumpModal {
        align: center middle;
    }

    #jump-container {
        width: 60;
        height: auto;
        background: $surface;
        border: thick $primary;
        padding: 1 2;
        align: center middle;
    }

    #jump-label {
        width: 100%;
        text-align: center;
        margin-bottom: 1;
        text-style: bold;
    }

    #jump-input {
        margin-bottom: 1;
    }

    #jump-exit {
        width: 100%;
        text-align: center;
        color: $text-muted;
        margin-top: 1;
    }
    """

    def compose(self) -> ComposeResult:
        with Container(id="jump-container"):
            yield Label("Jump to path (jq style):", id="jump-label")
            yield Input(placeholder=".items[0].id", id="jump-input")
            yield Label("(Press Esc to cancel)", id="jump-exit")

    def on_input_submitted(self, message: Input.Submitted) -> None:
        if message.value:
            self.dismiss(message.value)
        else:
            self.dismiss(None)
            
    def key_escape(self) -> None:
        self.dismiss(None)
