from textual.app import ComposeResult
from textual.containers import HorizontalScroll, Vertical
from textual.widgets import OptionList
from textual.message import Message
from textual.binding import Binding
import uuid
from typing import List, Optional
import asyncio
import re
from rich.text import Text

from twg.core.model import SQLiteModel, Node, DataType

class TwigOptionList(OptionList):
    """
    A customized OptionList that handles focus movement events.
    """
    BINDINGS = [
        Binding("left", "move_focus(-1)", "Back", show=False),
        Binding("right", "move_focus(1)", "Forward", show=False),
    ]

    class MoveFocus(Message):
        def __init__(self, direction: int):
            self.direction = direction
            super().__init__()

    def action_move_focus(self, direction: int) -> None:
        self.post_message(self.MoveFocus(direction))

class Column(Vertical):
    """A single column in the Miller Column view."""
    
    class Highlighted(Message):
        def __init__(self, column_index: int, node_id: uuid.UUID):
            self.column_index = column_index
            self.node_id = node_id
            super().__init__()

    def __init__(self, model: SQLiteModel, parent_node_id: uuid.UUID, index: int, initial_select_index: int = 0):
        self.model = model
        self.parent_node_id = parent_node_id
        self.index = index
        self.initial_select_index = initial_select_index
        super().__init__(classes="column")

    def compose(self) -> ComposeResult:
        try:
            parent = self.model.get_node(self.parent_node_id)
            if not parent:
                return

            options = []
            self.node_map = {} # Map option index to node ID
            
            # Lazy load children
            children = self.model.get_children(parent.id)
            
            for i, child in enumerate(children):
                if child:
                    # Icon logic
                    icon = " "
                    if child.type == DataType.OBJECT:
                        icon = "▶" 
                    elif child.type == DataType.ARRAY:
                        icon = "▶" 
                    
                    # Value preview (truncated)
                    val_str = ""
                    if not child.is_container:
                        val_str = f": {str(child.value)}"
                        if len(val_str) > 20:
                            val_str = val_str[:17] + "..."
                    
                    label = f"{icon} {child.key}{val_str}"
                    options.append(label)
                    self.node_map[i] = child.id
            
            opt_list = TwigOptionList(id=f"col-{self.index}")
            opt_list.add_options(options)
            yield opt_list
            
        except Exception as e:
             from textual.widgets import Label
             yield Label(f"Error loading column: {e}")

    def on_mount(self) -> None:
        """Select the initial item by default when the column is mounted."""
        # Check if list exists (might have failed to load)
        if not self.query(TwigOptionList):
            return

        opt_list = self.query_one(TwigOptionList)
        if opt_list.option_count > 0:
            # Respect the initial requested selection
            target = self.initial_select_index if self.initial_select_index < opt_list.option_count else 0
            opt_list.highlighted = target
            opt_list.scroll_to_highlight()

    def on_option_list_option_highlighted(self, event: OptionList.OptionHighlighted) -> None:
        has_focus = self.query_one(TwigOptionList).has_focus
        
        if has_focus:
            node_id = self.node_map.get(event.option_index)
            if node_id:
                self.post_message(self.Highlighted(self.index, node_id))
        else:
            pass # No focus, ignore

    def highlight_text_match(self, index: int, query: str) -> None:
        """
        Highlights the query text within the option at the given index.
        """
        opt_list = self.query_one(TwigOptionList)
        if index < 0 or index >= opt_list.option_count:
            return

        node_id = self.node_map.get(index)
        if not node_id: return
        
        node = self.model.get_node(node_id)
        if not node: return
        
        # Reconstruct label logic (duplicate of compose, but needed for Text construction)
        icon = " "
        if node.type == DataType.OBJECT:
            icon = "▶" 
        elif node.type == DataType.ARRAY:
            icon = "▶" 
        
        val_str = ""
        if not node.is_container:
            val_str = f": {str(node.value)}"
            if len(val_str) > 20:
                val_str = val_str[:17] + "..."
        
        full_label = f"{icon} {node.key}{val_str}"
        
        text = Text(full_label)
        if query:
            pattern = re.compile(re.escape(query), re.IGNORECASE)
            for match in pattern.finditer(full_label):
                text.stylize("bold reverse yellow", match.start(), match.end())
        
        try:
            opt_list.replace_option_at_index(index, text)
        except AttributeError:
             pass

    # find_next removed as it was unused and global search is preferred.



class ColumnNavigator(HorizontalScroll):
    """
    Manages the Miller Columns navigation.
    
    Handles column creation, removal, and focus management as the user navigates
    through the JSON structure.
    """

    class NodeSelected(Message):
        def __init__(self, node_id: uuid.UUID):
            self.node_id = node_id
            super().__init__()

    def __init__(self, model: SQLiteModel):
        self.model = model
        self._nav_id = 0 # To handle cancellation of jump/expand tasks
        self._pending_target_node_id: Optional[uuid.UUID] = None
        self._is_expanding = False
        super().__init__()

    def compose(self) -> ComposeResult:
        # Start with the root column
        if self.model.root_id:
            yield Column(self.model, self.model.root_id, 0)

    def __arrow_up(self) -> None:
        pass # Optional hook override

    async def _expand_node(self, column_index: int, node_id: uuid.UUID, initial_select_index: int = 0, animate: bool = True) -> None:
        """Helper to expand a node into a new column with a specific selection."""
        next_col_index = column_index + 1
        existing_next_col = None
        
        for child in self.children:
            if isinstance(child, Column) and child.index == next_col_index:
                existing_next_col = child
                break
        
        # If the next column exists and is showing the children of OUR selected node, we are good.
        if existing_next_col and existing_next_col.parent_node_id == node_id:
             self.post_message(self.NodeSelected(node_id))
             return

        to_remove = [child for child in self.children if isinstance(child, Column) and child.index > column_index]
        if to_remove:
             await asyncio.gather(*[child.remove() for child in to_remove])
        
        node = self.model.get_node(node_id)
        if node and node.is_container:
            for child in self.children:
                if isinstance(child, Column) and child.index == next_col_index:
                    await child.remove()

            new_col = Column(self.model, node.id, next_col_index, initial_select_index=initial_select_index)
            await self.mount(new_col)
            
            if animate:
                self.call_after_refresh(self.scroll_to_widget, new_col, animate=True)
            else:
                self.scroll_to_widget(new_col, animate=False)

        self.post_message(self.NodeSelected(node_id))

    async def expand_to_node(self, node_id: uuid.UUID) -> None:
        """
        Expands the column view to show and select the given node.
        Rebuilds the navigation path from root to the target node.
        """
        # Increment generation to cancel any running expansion
        self._nav_id += 1
        current_id = self._nav_id
        self._pending_target_node_id = node_id
        self._is_expanding = True
        
        try:
            node = self.model.get_node(node_id)
            if not node: return
            
            lineage = []
            curr = node
            while curr:
                lineage.append(curr)
                curr = self.model.get_node(curr.parent) if curr.parent else None
            lineage.reverse()
        
            for i in range(len(lineage) - 1):
                 if self._nav_id != current_id:
                     return

                 current_node = lineage[i]
                 next_node = lineage[i+1] # The one selected in Col i
                 
                 try:
                     children = self.model.get_children(current_node.id)
                     
                     select_idx = -1
                     for idx, child in enumerate(children):
                         if child.id == next_node.id:
                             select_idx = idx
                             break
                     
                     if select_idx != -1:
                         grandchild_idx = 0
                         if i + 2 < len(lineage):
                             grandchild = lineage[i+2]
                             grand_children = self.model.get_children(next_node.id)
                             for gc_idx, gc in enumerate(grand_children):
                                 if gc.id == grandchild.id:
                                     grandchild_idx = gc_idx
                                     break
                         
                         col_widget = None
                         for child in self.children:
                             if isinstance(child, Column) and child.index == i:
                                 col_widget = child
                                 break
                         
                         if col_widget:
                             if not col_widget.query(TwigOptionList):
                                 continue
                                 
                             opt_list = col_widget.query_one(TwigOptionList)
                             opt_list.highlighted = select_idx
                             opt_list.scroll_to_highlight(top=True) # Pin to top for predictable jumping
                             self.scroll_to_widget(col_widget, animate=False)
                             
                             await self._expand_node(i, next_node.id, initial_select_index=grandchild_idx, animate=False)
                             
                 except Exception as e:
                     self.notify(f"Jump Error: {e}", severity="error")
                     break
        
        finally:
             self._is_expanding = False

        if self._nav_id != current_id:
            return
            
        self._pending_target_node_id = None

        final_col_idx = len(lineage) - 1
        target_col_idx = max(0, len(lineage) - 2)
        
        def focus_target():
             if self._nav_id != current_id: return
             
             for child in self.children:
                 if isinstance(child, Column) and child.index == target_col_idx:
                     if child.query(TwigOptionList):
                         child.query_one(TwigOptionList).focus()
                         self.post_message(self.NodeSelected(node_id))
                     break
        self.call_after_refresh(focus_target)

    async def find_next(self, query: str, direction: int = 1) -> Optional[Node]:
        """
        Global search using the model's DFS.
        Returns the Node found, or None.
        """
        # 1. Determine start Node ID from current focus
        start_node_id = self._pending_target_node_id
        focused_col = None
        
        if not start_node_id:
            # Try finding focused column first
            for child in self.children:
                 if isinstance(child, Column) and child.query(TwigOptionList):
                     if child.query_one(TwigOptionList).has_focus:
                         focused_col = child
                         break
            
            # Fallback: Find deepest column with a highlight (if focus lost due to Loading modal)
            if not focused_col:
                 max_idx = -1
                 for child in self.children:
                     if isinstance(child, Column) and child.query(TwigOptionList):
                         opts = child.query_one(TwigOptionList)
                         if opts.highlighted is not None and child.index > max_idx:
                             max_idx = child.index
                             focused_col = child

            if focused_col and focused_col.query(TwigOptionList):
                opts = focused_col.query_one(TwigOptionList)
                if opts.highlighted is not None:
                    start_node_id = focused_col.node_map.get(opts.highlighted)
        
        # 2. Search in Model
        target_node = None
        
        # Feature: Smart Search (if query looks like a path, try resolving it first)
        if query.startswith("."):
            try:
                resolved = self.model.resolve_path(query)
                if resolved:
                    target_node = resolved
            except:
                pass

        if not target_node:
             # Note: Direction ignored for now, always forward global search since we use DFS generator
             target_node = self.model.find_next_node(query, start_node_id=start_node_id, direction=direction)
        
        if target_node:
            # 3. Expand to it
            await self.expand_to_node(target_node.id)
            # Post selection
            self.post_message(self.NodeSelected(target_node.id))
            
            # 4. Highlight text match in the target column
            for child in self.children:
                if isinstance(child, Column) and child.parent_node_id == target_node.parent:
                     if child.query(TwigOptionList):
                         opts = child.query_one(TwigOptionList)
                         if opts.highlighted is not None:
                             child.highlight_text_match(opts.highlighted, query)
            
            return target_node
        
        return None

    async def on_mount(self) -> None:
        # Focus the first option list
        if self.children:
            first_col = self.children[0]
            if isinstance(first_col, Column) and first_col.query(TwigOptionList):
                opt_list = first_col.query_one(TwigOptionList)
                opt_list.focus()
                # Manually trigger initial selection
                if opt_list.highlighted is not None:
                     node_id = first_col.node_map.get(opt_list.highlighted)
                     if node_id:
                         await self._expand_node(first_col.index, node_id, initial_select_index=0)

    async def on_column_highlighted(self, message: Column.Highlighted) -> None:
        """Handles selection within a column."""
        if self._is_expanding:
            return
        await self._expand_node(message.column_index, message.node_id, initial_select_index=0)

    def on_twig_option_list_move_focus(self, message: TwigOptionList.MoveFocus) -> None:
        self._move_focus(message.direction)

    def _move_focus(self, direction: int) -> None:
        # Find currently focused column index
        focused = self.screen.focused
        if not focused:
            return

        # Find which column contains the focused widget
        current_col = None
        for child in self.children:
            if isinstance(child, Column) and child.query_one(TwigOptionList).has_focus:
                current_col = child
                break
        
        if current_col:
            target_index = current_col.index + direction
            # Find target column
            for child in self.children:
                if isinstance(child, Column) and child.index == target_index:
                    target_col = child
                    if not target_col.query(TwigOptionList):
                        self.notify("Cannot focus empty column", severity="warning")
                        continue
                        
                    target_option_list = target_col.query_one(TwigOptionList)
                    target_option_list.focus()
                    self.scroll_to_widget(target_col, animate=True)
                    
                    if target_option_list.highlighted is not None:
                        node_id = target_col.node_map.get(target_option_list.highlighted)
                        if node_id:
                            self.run_worker(self.on_column_highlighted(Column.Highlighted(target_col.index, node_id)))
                    return
