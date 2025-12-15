import importlib.metadata
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
        border: tall $primary; 
        padding: 1 4;
        align: center middle;
    }

    #help-logo {
        width: 100%;
        text-align: center;
        text-style: bold;
        color: $success;
        margin-top: 1;
        margin-bottom: 1;
    }

    .help-tagline {
        width: 100%;
        text-align: center;
        color: $accent;
        margin-bottom: 1;
    }

    .help-version {
        width: 100%;
        text-align: center;
        color: $warning;
        margin-bottom: 2;
    }

    #help-link {
        width: 100%;
        text-align: center;
        color: $primary;
        text-style: underline;
        margin-bottom: 3;
    }

    #help-title {
        width: 100%;
        text-align: center;
        text-style: bold underline;
        color: $text;
        margin-bottom: 1;
        background: $surface;
    }

    #help-grid {
        layout: grid;
        grid-size: 2;
        grid-columns: auto auto;
        grid-gutter: 0 4;
        width: auto;
        height: auto;
        padding: 0 0;
        margin-bottom: 2;
        align: center middle;
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
        ("/", "Search "),
        ("n / N", "Next / Prev match"),
        (":", "Jump to path"),
        ("c", "Copy path"),
        ("t", "Toggle theme"),
        ("q", "Quit"),
    ]

    def compose(self) -> ComposeResult:
        try:
            version = importlib.metadata.version("twg")
        except importlib.metadata.PackageNotFoundError:
            version = "unknown"

        with Container(id="help-container"):
            # Branding Section
            yield Label("Twig", id="help-logo")
            yield Label("Inspect. Navigate. Understand.", classes="help-tagline")
            yield Label(f"v{version}", classes="help-version")
            yield Label("[@click=app.open_url('https://github.com/workdone0/twig')]github.com/workdone0/twig[/]", id="help-link")
            
            # Keybindings Section
            yield Label("Keyboard Shortcuts", id="help-title")
            
            with Grid(id="help-grid"):
                for key, desc in self.KEYBINDINGS:
                    yield Label(key, classes="key")
                    yield Label(desc, classes="desc")

            yield Label("(Press Esc to close)", id="help-exit")

    def key_escape(self) -> None:
        self.dismiss(None)
    
