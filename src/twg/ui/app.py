from textual.app import App, ComposeResult
from textual.widgets import Footer, Header, LoadingIndicator, Label, Static
from textual.containers import Container, Horizontal, Vertical, Center, Middle
import sys
import os
import argparse
import pyperclip
from rich.console import Console
from rich.json import JSON
from rich.panel import Panel
from rich.text import Text
from rich import box
from rich.syntax import Syntax

from textual.binding import Binding

try:
    import yaml
except ImportError:
    yaml = None

from twg.core.model import SQLiteModel
from twg.core.config import Config
from twg.adapters.sqlite_loader import SQLiteLoader
from twg.adapters.yaml_loader import YamlLoader
from twg.ui.widgets.navigator import ColumnNavigator
from twg.ui.widgets.inspector import Inspector
from twg.ui.widgets.status_bar import StatusBar
from twg.ui.widgets.search import SearchModal
from twg.ui.widgets.jump import JumpModal
from twg.ui.widgets.help import HelpModal
from twg.ui.widgets.loading import LoadingScreen
from twg.ui.widgets.breadcrumbs import Breadcrumbs
from twg.core.model import Node
from twg.core.cleaner import repair_json

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
        ("y", "copy_source", "Copy Source"),
        ("t", "toggle_theme", "Toggle Theme"),
        ("/", "search", "Search"),
        ("n", "next_match", "Next Match"),
        ("N", "prev_match", "Prev Match"),
        (":", "jump", "Jump to Path"),
        ("?", "help", "Help"),
        Binding("h", "help", "Help", show=False),
    ]

    def on_mount(self) -> None:
        from twg.ui.themes import SOLARIZED_DARK, CATPPUCCIN_MOCHA
        self.register_theme(SOLARIZED_DARK)
        self.register_theme(CATPPUCCIN_MOCHA)
        
        self.config = Config()
        self.theme = self.config.get("theme", "catppuccin-mocha")
        self.title = "Twig"
        self.run_worker(self.load_file, thread=True)

    def load_file(self) -> None:
        """Worker method to load the file in a background thread."""
        try:
            if self.file_path.lower().endswith(('.yaml', '.yml')):
                self.format = "yaml"
                loader = YamlLoader()
            else:
                self.format = "json"
                loader = SQLiteLoader()
            
            model = loader.load_into_model(
                self.file_path, 
                force_rebuild=self.force_rebuild
            )
            self.call_from_thread(self.on_file_loaded, model)
        except Exception as e:
            self.call_from_thread(self.on_file_load_error, str(e))
    
    def on_file_loaded(self, model: SQLiteModel) -> None:
        """Called when the file is successfully loaded."""
        self.model = model
        
        # Dismiss loading screen if it exists
        try:
            self.pop_screen()
        except:
            pass
            
        content = self.query_one("#main-content")
        content.remove_children()
        
        content.mount(
            Vertical(
                Breadcrumbs(self.model, id="breadcrumbs"),
                Horizontal(
                    ColumnNavigator(self.model),
                    Inspector(self.model, format=self.format, id="inspector"),
                    id="main-container"
                ),
                id="content-view"
            )
        )
        
        status_bar = self.query_one(StatusBar)
        status_bar.model = self.model

    def on_file_load_error(self, error_message: str) -> None:
        """Called when file loading fails."""
        # Exit the app and pass the error message back to the caller
        
        # Colorized hint using Rich markup
        
        hint = (
            f"\n\n[bold yellow]Tip:[/bold yellow] If the file contains invalid JSON, try running:\n"
            f"  [green]twg --fix {self.file_path} -o {self.file_path}[/green]\n"
            f"[dim](This will repair and overwrite the file in place)[/dim]"
        )
        self.exit(f"[bold red]Error:[/bold red] {error_message}{hint}")

    def action_toggle_theme(self) -> None:
        """Cycles through all available themes and notifies the user."""
        themes = [t for t in self.available_themes.keys() if t != "textual-ansi"]
        try:
            current_index = themes.index(self.theme)
        except ValueError:
            current_index = 0
            
        next_index = (current_index + 1) % len(themes)
        next_theme = themes[next_index]
        
        self.theme = next_theme
        self.config.set("theme", next_theme)
        self.notify(f"Theme: {next_theme}")

    def __init__(self, file_path: str, force_rebuild: bool = False):
        self.file_path = file_path
        self.force_rebuild = force_rebuild
        self.model: SQLiteModel | None = None
        self.current_node: Node | None = None
        self.last_search_query: str | None = None
        self.config: Config | None = None
        self.format: str = "json" # Default
        super().__init__()

    def compose(self) -> ComposeResult:
        yield Header(show_clock=False)
        
        with Container(id="main-content"):
            yield Label("Initializing...", id="init-label")
            
        yield StatusBar(self.file_path, id="status-bar")
        yield Footer()

    def on_ready(self) -> None:
        """Push loading screen once app is ready."""
        self.push_screen(LoadingScreen(message=f"Loading {os.path.basename(self.file_path)}..."))

    def on_column_navigator_node_selected(self, message: ColumnNavigator.NodeSelected) -> None:
        """
        Handler for node selection events from the ColumnNavigator.
        
        Updates the Inspector, Breadcrumbs, and Status Bar with the selected node details.
        """
        node = self.model.get_node(message.node_id)
        self.current_node = node
        
        inspector = self.query_one(Inspector)
        inspector.selected_node = node
        
        breadcrumbs = self.query_one(Breadcrumbs)
        breadcrumbs.selected_node = node
        
        status_bar = self.query_one(StatusBar)
        status_bar.selected_node = node

    def action_search(self) -> None:
        """Open the search modal."""
        async def check_search(query: str | None) -> None:
            if query:
                self.last_search_query = query
                await self.action_next_match()
        
        self.push_screen(SearchModal(), check_search)
            
    def action_help(self) -> None:
        """Open the help modal."""
        self.push_screen(HelpModal())

    async def action_jump(self) -> None:
        """Open the jump modal."""
        async def check_jump(path: str | None) -> None:
            if path:
                if not self.model: return
                
                try:
                    node = self.model.resolve_path(path)
                    if node:
                        navigator = self.query_one(ColumnNavigator)
                        await navigator.expand_to_node(node.id)
                    else:
                        self.notify(f"Path not found: {path}", severity="error")
                except ValueError as e:
                    self.notify(str(e), severity="error")

        await self.push_screen(JumpModal(), check_jump)

    async def action_next_match(self) -> None:
        """Find next match for the last query."""
        if not self.last_search_query:
            self.notify("No active search query.", severity="warning")
            return
            
        navigator = self.query_one(ColumnNavigator)
        
        found_node = await navigator.find_next(self.last_search_query, direction=1)
        if not found_node:
             self.notify(f"Not found: '{self.last_search_query}'", severity="warning")
        else:
             # Show stats (current/total)
             if self.model:
                 current, total = self.model.get_search_stats(self.last_search_query, found_node.id)
                 if total > 0:
                     self.query_one(StatusBar).search_stats = f"{current}/{total}"

    async def action_prev_match(self) -> None:
        """Find previous match for the last search query."""
        if not self.last_search_query:
            self.notify("No active search query.", severity="warning")
            return

        navigator = self.query_one(ColumnNavigator)
        
        found_node = await navigator.find_next(self.last_search_query, direction=-1)
        if not found_node:
             self.notify(f"Not found: '{self.last_search_query}'", severity="warning")
        else:
             # Show stats
             if self.model:
                 current, total = self.model.get_search_stats(self.last_search_query, found_node.id)
                 if total > 0:
                     self.query_one(StatusBar).search_stats = f"{current}/{total}"

    def action_copy_path(self) -> None:
        """Copies the jq-style path of the currently selected node to the clipboard."""
        if not self.current_node:
            return
            
        full_path = self.model.get_path(self.current_node.id)
        if full_path:
            try:
                pyperclip.copy(full_path)
                self.notify(f"Copied path: {full_path}")
            except Exception as e:
                self.notify(f"Failed to copy: {e}", severity="error")
        else:
             self.notify("Root path copied")

    def action_copy_source(self) -> None:
        """Copies the raw source (JSON/YAML) of the selected node to clipboard."""
        if not self.current_node:
            return
            
        import json
        
        try:
            if self.current_node.is_container:
                data = self.model.reconstruct_json(self.current_node.id, max_depth=5)
            else:
                data = self.current_node.value

            if self.format == "yaml" and yaml:
                source_str = yaml.dump(data, sort_keys=False, default_flow_style=False)
                label = "YAML source"
            else:
                source_str = json.dumps(data, indent=2)
                label = "source"
            
            pyperclip.copy(source_str)
            
            preview = source_str[:50].replace('\n', ' ')
            if len(source_str) > 50: preview += "..."
            
            self.notify(f"Copied {label}: {preview}")
            
        except Exception as e:
            self.notify(f"Failed to copy source: {e}", severity="error")

def run():
    parser = argparse.ArgumentParser(
        description="Twig: Inspect. Navigate. Understand. A modern, terminal-based data explorer."
    )
    parser.add_argument(
        "file",
        help="The JSON/data file to explore."
    )
    parser.add_argument(
        "-o", "--output",
        help="Output file. For --fix, it saves the repaired JSON. For --print, it saves the formatted JSON. If omitted, prints to stdout."
    )
    
    try:
        from importlib.metadata import version, PackageNotFoundError
        twg_version = version("twg")
    except ImportError:
        twg_version = "unknown"
    except PackageNotFoundError:
        twg_version = "dev"

    parser.add_argument(
        "-v", "--version",
        action="version",
        version=f"%(prog)s {twg_version}"
    )
    parser.add_argument(
        "--fix",
        action="store_true",
        help="Attempt to automatically repair malformed JSON and exit."
    )
    parser.add_argument(
        "-p", "--print",
        action="store_true",
        help="Print formatted and syntax-highlighted JSON to stdout and exit."
    )
    
    parser.add_argument(
        "--indent",
        type=int,
        default=2,
        help="Number of spaces for indentation (default: 2)."
    )
    parser.add_argument(
        "--rebuild-db",
        action="store_true",
        help="Force rebuild of the internal SQLite database cache."
    )
    
    args = parser.parse_args()

    if not args.fix and not args.print:
        if not os.path.exists(args.file):
            print(f"Error: File not found: {args.file}", file=sys.stderr)
            sys.exit(1)

        if not args.file.lower().endswith((".json", ".yaml", ".yml")):
            print(f"Error: Invalid file type '{args.file}'. Twig currently only supports .json and .yaml files.", file=sys.stderr)
            sys.exit(1)
            
        app = TwigApp(args.file, force_rebuild=args.rebuild_db)
        result = app.run()
        
        if result is not None and isinstance(result, str):
            console = Console(stderr=True)
            console.print(result)
            sys.exit(1)
        return

    if not os.path.exists(args.file):
        print(f"Error: File not found: {args.file}", file=sys.stderr)
        sys.exit(1)

    is_yaml = args.file.lower().endswith(('.yaml', '.yml'))

    if is_yaml:
        if args.fix:
            print("Error: Fix/Repair is currently only supported for JSON files.", file=sys.stderr)
            sys.exit(1)
        
        try:
            with open(args.file, 'r', encoding='utf-8') as f:
                content = f.read()
            
            try:
                if yaml:
                    parsed = yaml.safe_load(content)
                else:
                    print("Error: PyYAML is not installed. Cannot process YAML files.", file=sys.stderr)
                    sys.exit(1)
            except Exception as e:
                print(f"Error: Failed to parse YAML: {e}", file=sys.stderr)
                sys.exit(1)
                
            if args.output:
                with open(args.output, 'w', encoding='utf-8') as f:
                    yaml.dump(parsed, f, sort_keys=False, default_flow_style=False)
                print(f"Formatted YAML written to {args.output}", file=sys.stderr)
            
            else:
                file_size = len(content.encode('utf-8'))
                
                if isinstance(parsed, list):
                    item_count = len(parsed)
                    type_label = "Array"
                elif isinstance(parsed, dict):
                    item_count = len(parsed)
                    type_label = "Object"
                else:
                    item_count = 1
                    type_label = "Primitive"
                
                err_console = Console(stderr=True)
                meta_text = Text()
                meta_text.append(f" File: ", style="bold cyan")
                meta_text.append(f"{os.path.basename(args.file)} \n")
                meta_text.append(f" Size: ", style="bold cyan")
                meta_text.append(f"{file_size / 1024:.1f} KB \n")
                meta_text.append(f" Type: ", style="bold cyan")
                meta_text.append(f"{type_label} ({item_count} items)")
                
                err_console.print(Panel(
                    meta_text,
                    title="Twig Metadata",
                    border_style="blue",
                    box=box.ROUNDED,
                    expand=False
                ))

                console = Console()
                syntax = Syntax(content, "yaml", theme="monokai", line_numbers=False)
                console.print(syntax)
                
        except Exception as e:
            print(f"Error processing YAML file: {e}", file=sys.stderr)
            sys.exit(1)
        
        sys.exit(0)

    try:
        with open(args.file, 'r', encoding='utf-8') as f:
            content = f.read()

        if args.fix:
            try:
                content = repair_json(content)
            except Exception as e:
                print(f"Error repairing JSON: {e}", file=sys.stderr)
                sys.exit(1)
        
        try:
            import json
            parsed = json.loads(content)
        except json.JSONDecodeError as e:
            if args.print:
                 print(f"Error: Failed to parse JSON. Use --fix to attempt repair.\n{e}", file=sys.stderr)
                 sys.exit(1)
            else:
                 print(f"Error: Resulting JSON is invalid even after attempted fix.\n{e}", file=sys.stderr)
                 sys.exit(1)

        if args.output:
            # Write to file (formatted)
            with open(args.output, 'w', encoding='utf-8') as f:
                json.dump(parsed, f, indent=args.indent)
            
            action = "Fixed" if args.fix else "Formatted"
            print(f"{action} JSON written to {args.output}", file=sys.stderr)
        
        else:
            if args.print:
                file_size = len(content.encode('utf-8'))
                
                if isinstance(parsed, list):
                    item_count = len(parsed)
                    type_label = "Array"
                elif isinstance(parsed, dict):
                    item_count = len(parsed)
                    type_label = "Object"
                else:
                    item_count = 1
                    type_label = "Primitive"
                
                err_console = Console(stderr=True)
                meta_text = Text()
                meta_text.append(f" File: ", style="bold cyan")
                meta_text.append(f"{os.path.basename(args.file)} \n")
                meta_text.append(f" Size: ", style="bold cyan")
                meta_text.append(f"{file_size / 1024:.1f} KB \n")
                meta_text.append(f" Type: ", style="bold cyan")
                meta_text.append(f"{type_label} ({item_count} items)")
                
                err_console.print(Panel(
                    meta_text,
                    title="Twig Metadata",
                    border_style="blue",
                    box=box.ROUNDED,
                    expand=False
                ))

                # JSON to stdout
                console = Console()
                console.print(JSON(content, indent=args.indent))
            
            else:
                print(json.dumps(parsed, indent=args.indent))

        sys.exit(0)

    except Exception as e:
        print(f"Error processing file: {e}", file=sys.stderr)
        sys.exit(1)
    

if __name__ == "__main__":
    run()
