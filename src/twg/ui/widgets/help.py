from textual.app import ComposeResult
from textual.screen import ModalScreen
from textual.widgets import Label
from textual.containers import Container, Grid

class HelpModal(ModalScreen[None]):
    """Modal screen giving help/keybindings."""

    CSS = """
    HelpModal {
        align: center middle;
    }

    #help-container {
        width: 60;
        height: auto;
        background: $surface;
        border: thick $primary;
        padding: 1 2;
        align: center middle; 
    }

    #help-title {
        width: 100%;
        text-align: center;
        text-style: bold;
        margin-bottom: 2;
    }

    #help-grid {
        layout: grid;
        grid-size: 2;
        grid-columns: auto 1fr;
        grid-gutter: 1 2;
        width: 100%;
        height: auto;
        margin-bottom: 2;
    }

    .key {
        text-align: right;
        text-style: bold;
        color: $accent;
    }

    .desc {
        text-align: left;
        color: $text;
    }

    #help-exit {
        width: 100%;
        text-align: center;
        color: $text-muted;
    }
    """

    KEYBINDINGS = [
        ("Arrows", "Navigate tree"),
        ("/", "Search keys/values"),
        ("n / N", "Next / Prev match"),
        (":", "Jump to path"),
        ("c", "Copy path to clipboard"),
        ("t", "Toggle theme"),
        ("q", "Quit"),
    ]

    def compose(self) -> ComposeResult:
        with Container(id="help-container"):
            yield Label("Keyboard Shortcuts", id="help-title")
            
            with Grid(id="help-grid"):
                for key, desc in self.KEYBINDINGS:
                    yield Label(key, classes="key")
                    yield Label(desc, classes="desc")
            
            yield Label("(Press Esc to close)", id="help-exit")

    def key_escape(self) -> None:
        self.dismiss(None)
    
    def on_click(self) -> None:
        self.dismiss(None)
