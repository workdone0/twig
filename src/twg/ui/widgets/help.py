import importlib.metadata
from textual.app import ComposeResult
from textual.screen import ModalScreen
from textual.widgets import Label
from textual.containers import Container, Grid

class HelpModal(ModalScreen[None]):
    """Modal screen giving help/keybindings."""

    LOGO = r"""
████████╗██╗    ██╗██╗ ██████╗ 
╚══██╔══╝██║    ██║██║██╔════╝ 
   ██║   ██║ █╗ ██║██║██║  ███╗
   ██║   ██║███╗██║██║██║   ██║
   ██║   ╚███╔███╔╝██║╚██████╔╝
   ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ 
"""

    CSS = """
    HelpModal {
        align: center middle;
    }

    #help-container {
        width: 70;
        height: auto;
        background: $surface;
        border: tall $primary; 
        padding: 1 2;
        align: center middle;
    }

    #help-logo {
        width: 100%;
        text-align: center;
        color: $warning;
        margin-top: 1;
        margin-bottom: 1;
    }

    .help-tagline {
        width: 100%;
        text-align: center;
        color: $success;
        text-style: bold;
        margin-bottom: 1;
    }

    .help-version {
        width: 100%;
        text-align: center;
        color: $text-muted;
        margin-bottom: 2;
    }

    #help-link {
        width: 100%;
        text-align: center;
        color: $primary;
        text-style: underline;
        margin-bottom: 1;
    }

    #coffee-link {
        width: 100%;
        text-align: center;
        color: $accent;
        margin-bottom: 2;
    }

    #help-title {
        width: 100%;
        text-align: center;
        text-style: bold underline;
        color: $text;
        margin-bottom: 1;
        margin-top: 1;
        background: $surface;
    }

    #help-grid {
        layout: grid;
        grid-size: 2;
        grid-columns: 1fr 1fr;
        grid-gutter: 0 2;
        width: 100%;
        height: auto;
        margin-bottom: 2;
        align: center middle;
    }

    .key {
        text-align: left;
        text-style: bold;
        color: $accent;
    }

    .desc {
        text-align: right;
        color: $text;
    }

    #help-exit {
        width: 100%;
        text-align: center;
        color: $text-muted;
        margin-bottom: 1;
    }
    """

    KEYBINDINGS = [
        ("Arrows", "Navigate tree"),
        ("/", "Search "),
        ("n / N", "Next / Prev match"),
        (":", "Jump to path"),
        ("c", "Copy path"),
        ("y", "Copy source"),
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
            yield Label(self.LOGO, id="help-logo")
            yield Label("Inspect. Navigate. Understand.", classes="help-tagline")
            yield Label(f"v{version}", classes="help-version")
            yield Label(f"v{version}", classes="help-version")
            yield Label("https://twig.wtf", id="help-link")
            yield Label("☕ buymeacoffee.com/workdone0", classes="help-tagline", id="coffee-link")
            
            # Keybindings Section
            yield Label("Keyboard Shortcuts", id="help-title")
            
            with Grid(id="help-grid"):
                for key, desc in self.KEYBINDINGS:
                    yield Label(key, classes="key")
                    yield Label(desc, classes="desc")

            yield Label("(Press Esc to close)", id="help-exit")

    def key_escape(self) -> None:
        self.dismiss(None)
    
