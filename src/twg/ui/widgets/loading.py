from textual.app import ComposeResult
from textual.screen import ModalScreen
from textual.widgets import Label, LoadingIndicator
from textual.containers import Container, Center, Middle, Vertical


class LoadingScreen(ModalScreen):
    """A modal screen that shows a loading indicator."""

    LOGO = r"""
████████╗██╗    ██╗██╗ ██████╗ 
╚══██╔══╝██║    ██║██║██╔════╝ 
   ██║   ██║ █╗ ██║██║██║  ███╗
   ██║   ██║███╗██║██║██║   ██║
   ██║   ╚███╔███╔╝██║╚██████╔╝
   ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ 
"""

    CSS = """
    LoadingScreen {
        align: center middle;
    }

    #loading-container {
        width: 60;
        height: auto;
        background: $surface;
        border: thick $primary;
        padding: 1 2;
    }
    
    #loading-logo {
        width: 100%;
        text-align: center;
        color: $warning;
        margin-bottom: 1;
    }

    #loading-bar {
        width: 100%;
        margin-bottom: 1;
    }

    #loading-label {
        width: 100%;
        text-align: center;
        text-style: bold;
    }

    .loading-link {
        width: 100%;
        text-align: center;
        color: $text-muted;
        margin-top: 1;
    }
    """

    def __init__(self, message: str = "Working...", name: str | None = None, id: str | None = None, classes: str | None = None):
        self.message = message
        super().__init__(name, id, classes)

    def compose(self) -> ComposeResult:
        with Container(id="loading-container"):
            yield Label(self.LOGO, id="loading-logo")
            yield Center(LoadingIndicator())
            yield Label(self.message, id="loading-label")
            yield Label("https://twig.wtf", classes="loading-link")
            yield Label("☕ buymeacoffee.com/workdone0", classes="loading-link")
