from textual.app import App, ComposeResult
from textual.widgets import Footer, Header, LoadingIndicator, Label, Static
from textual.containers import Container, Horizontal, Vertical, Center, Middle
import sys
import os
import asyncio

from twg.core.model import TwigModel
from twg.adapters.json_adapter import JsonAdapter
from twg.ui.widgets.column_navigator import ColumnNavigator
from twg.ui.widgets.inspector import Inspector

from twg.ui.widgets.breadcrumbs import Breadcrumbs
from twg.core.model import Node

MAX_FILE_SIZE_WARNING = 100 * 1024 * 1024 # 100 MB

class TwigApp(App):
    """
    The main Textual application class for Twig.
    
    Manages the application lifecycle, layout composition, and global actions
    like theme toggling and path copying.
    """
    CSS_PATH = "styles.tcss"

    BINDINGS = [
        ("q", "quit", "Quit"),
        ("c", "copy_path", "Copy Path"),
        ("t", "toggle_theme", "Toggle Theme"),
    ]

    def on_mount(self) -> None:
        self.theme = "catppuccin-mocha"
        self.title = "Twig"
        self.run_worker(self.load_file, thread=True)

    def load_file(self) -> None:
        """Worker method to load the file in a background thread."""
        try:
            # check file size
            try:
                size = os.path.getsize(self.file_path)
                if size > MAX_FILE_SIZE_WARNING:
                    self.call_from_thread(
                        self.notify, 
                        f"Large file detected ({size / (1024*1024):.1f} MB). Performance may be degraded.", 
                        severity="warning",
                        timeout=5
                    )
            except OSError:
                pass # safely ignore, adapter will handle file errors

            adapter = JsonAdapter()
            model = adapter.load_into_model(self.file_path)
            self.call_from_thread(self.on_file_loaded, model)
        except Exception as e:
            self.call_from_thread(self.on_file_load_error, str(e))
    
    def on_file_loaded(self, model: TwigModel) -> None:
        """Called when the file is successfully loaded."""
        self.model = model
        
        # Remove loading indicator
        content = self.query_one("#main-content")
        content.remove_children()
        
        # Mount the actual UI
        content.mount(
            Vertical(
                Breadcrumbs(self.model, id="breadcrumbs"),
                Horizontal(
                    ColumnNavigator(self.model),
                    Inspector(self.model, id="inspector"),
                    id="main-container"
                ),
                id="content-view"
            )
        )

    def on_file_load_error(self, error_message: str) -> None:
        """Called when file loading fails."""
        # Exit the app and pass the error message back to the caller
        self.exit(error_message)

    def action_toggle_theme(self) -> None:
        """Cycles through all available themes and notifies the user."""
        themes = list(self.available_themes.keys())
        try:
            current_index = themes.index(self.theme)
        except ValueError:
            current_index = 0
            
        next_index = (current_index + 1) % len(themes)
        next_theme = themes[next_index]
        
        self.theme = next_theme
        self.notify(f"Theme: {next_theme}")

    def __init__(self, file_path: str):
        self.file_path = file_path
        self.model: TwigModel | None = None
        self.current_node: Node | None = None
        super().__init__()

    def compose(self) -> ComposeResult:
        yield Header(show_clock=False)
        
        # Initial state: Loading
        with Container(id="main-content"):
            yield Center(Middle(LoadingIndicator()))
            yield Center(Middle(Label(f"Loading {os.path.basename(self.file_path)}...")))
        
        yield Footer()

    def on_column_navigator_node_selected(self, message: ColumnNavigator.NodeSelected) -> None:
        """
        Handler for node selection events from the ColumnNavigator.
        
        Updates the Inspector and Breadcrumbs with the selected node details.
        """
        node = self.model.get_node(message.node_id)
        self.current_node = node
        
        inspector = self.query_one(Inspector)
        inspector.selected_node = node
        
        breadcrumbs = self.query_one(Breadcrumbs)
        breadcrumbs.selected_node = node

    def action_copy_path(self) -> None:
        """Copies the jq-style path of the currently selected node to the clipboard."""
        if not self.current_node:
            return
            
        full_path = self.model.get_path(self.current_node.id)
        if full_path:
            import pyperclip
            try:
                pyperclip.copy(full_path)
                self.notify(f"Copied path: {full_path}")
            except Exception as e:
                self.notify(f"Failed to copy: {e}", severity="error")
        else:
             self.notify("Root path copied")

import argparse

def run():
    parser = argparse.ArgumentParser(
        description="Twig: Inspect. Navigate. Understand. A terminal-based data explorer."
    )
    parser.add_argument(
        "file",
        help="The JSON/data file to explore."
    )
    parser.add_argument(
        "-v", "--version",
        action="version",
        version="%(prog)s 1.0.0"
    )
    
    args = parser.parse_args()
    
    app = TwigApp(args.file)
    result = app.run()
    
    if result is not None and isinstance(result, str):
        print(f"Error: {result}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    run()
