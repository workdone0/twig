from textual.app import ComposeResult
from textual.containers import HorizontalScroll, Vertical
from textual.widgets import OptionList
from textual.message import Message
from textual.binding import Binding
import uuid
from typing import List

from twg.core.model import TwigModel, Node, DataType

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

    def __init__(self, model: TwigModel, parent_node_id: uuid.UUID, index: int, initial_select_index: int = 0):
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
        opt_list = self.query_one(TwigOptionList)
        if opt_list.option_count > 0:
            # Respect the initial requested selection
            target = self.initial_select_index if self.initial_select_index < opt_list.option_count else 0
            opt_list.highlighted = target

    def on_option_list_option_highlighted(self, event: OptionList.OptionHighlighted) -> None:
        has_focus = self.query_one(TwigOptionList).has_focus
        
        if has_focus:
            node_id = self.node_map.get(event.option_index)
            if node_id:
                self.post_message(self.Highlighted(self.index, node_id))
        else:
            pass # No focus, ignore


    async def find_next(self, query: str, direction: int = 1) -> bool:
        """
        Finds the next item matching the query.
        Returns True if found and selected, False otherwise.
        """
        opt_list = self.query_one(TwigOptionList)
        count = opt_list.option_count
        if count == 0:
            return False

        start_index = opt_list.highlighted if opt_list.highlighted is not None else -1
        
        # indices to check
        indices = list(range(count))
        # rotate so we start after start_index
        if direction > 0:
            # next: start+1 ... end, 0 ... start
            check_order = indices[start_index+1:] + indices[:start_index+1]
        else:
            # prev: start-1 ... 0, end ... start
            check_order = []
            curr = start_index - 1
            while len(check_order) < count:
                if curr < 0: curr = count - 1
                check_order.append(curr)
                curr -= 1

        query = query.lower()
        
        for idx in check_order:
            node_id = self.node_map.get(idx)
            if node_id:
                node = self.model.get_node(node_id)
                # Matches key (primary) or raw value string (secondary)
                # Check key
                match = query in str(node.key).lower()
                # Check value (if primitive)
                if not match and not node.is_container:
                    match = query in str(node.value).lower()
                
                if node and match:
                    opt_list.highlighted = idx
                    return True
        
        return False

MAX_SEARCH_DEPTH = 10
MAX_SEARCH_WIDTH = 5

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

    def __init__(self, model: TwigModel):
        self.model = model
        super().__init__()

    def compose(self) -> ComposeResult:
        # Start with the root column
        if self.model.root_id:
            yield Column(self.model, self.model.root_id, 0)
            
    def _find_in_children(self, parent_id: uuid.UUID, query: str, depth: int = 0, path: List[int] = []) -> tuple[bool, List[int], int]:
        """
        Recursively search in children.
        Returns (found, path_of_indices_to_expand, match_index_in_final_col)
        """
        if depth > MAX_SEARCH_DEPTH: 
            return False, [], -1
            
        children = self.model.get_children(parent_id)
        query_lower = query.lower()
        
        # 1. Check current level for match
        for i, child in enumerate(children):
            if child:
                match = query_lower in str(child.key).lower()
                if not match and not child.is_container:
                    match = query_lower in str(child.value).lower()
                
                if match:
                    return True, path, i
        
        # 2. Recurse deeper (first few containers only for performance optimization)
        count = 0 
        for i, child in enumerate(children):
             if child and child.is_container:
                 # Recurse
                 found, sub_path, match_idx = self._find_in_children(child.id, query, depth + 1, path + [i])
                 if found:
                     return True, sub_path, match_idx
                 
                 count += 1
                 if count > MAX_SEARCH_WIDTH: # Only check first N containers to avoid freezing
                     break
        
        return False, [], -1

    async def _expand_node(self, column_index: int, node_id: uuid.UUID, initial_select_index: int = 0) -> None:
        """Helper to expand a node into a new column with a specific selection."""
        # 1. Remove all columns to the right
        to_remove = [child for child in self.children if isinstance(child, Column) and child.index > column_index]
        for child in to_remove:
            child.remove()
        
        # 2. Check if the selected node is a container
        node = self.model.get_node(node_id)
        if node and node.is_container:
            # 3. Add new column for the selected node
            new_col_index = column_index + 1
            new_col = Column(self.model, node.id, new_col_index, initial_select_index=initial_select_index)
            await self.mount(new_col)
            
            # Scroll to the new column
            self.call_after_refresh(self.scroll_to_widget, new_col)

        # 4. Notify app about selection (of the parent node that caused expansion)
        self.post_message(self.NodeSelected(node_id))

    async def expand_to_node(self, node_id: uuid.UUID) -> None:
        """
        Expands the column view to show and select the given node.
        Rebuilds the navigation path from root to the target node.
        """
        # 1. Build lineage (Root -> ... -> Node)
        node = self.model.get_node(node_id)
        if not node: return
        
        lineage = []
        curr = node
        while curr:
            lineage.append(curr)
            curr = self.model.get_node(curr.parent) if curr.parent else None
        lineage.reverse()
        
        # 2. Iterate and expand
        # lineage[0] is Root. Col 0 shows children of Root.
        
        for i in range(len(lineage) - 1):
             current_node = lineage[i]
             next_node = lineage[i+1] # The one selected in Col i
             
             # Find index of next_node in current_node's children
             children = self.model.get_children(current_node.id)
             try:
                 # Note: iterating to find index might be O(N) but handled by lazy load usually small enough?
                 # Or we can optimize model to look up index? 
                 # For now list.index is fine.
                 # We need to match IDs. 'children' contains Node objects.
                 select_idx = -1
                 for idx, child in enumerate(children):
                     if child.id == next_node.id:
                         select_idx = idx
                         break
                 
                 if select_idx != -1:
                     # Calculate what to select in the NEXT column (grandchild)
                     grandchild_idx = 0
                     if i + 2 < len(lineage):
                         grandchild = lineage[i+2]
                         grand_children = self.model.get_children(next_node.id)
                         for gc_idx, gc in enumerate(grand_children):
                             if gc.id == grandchild.id:
                                 grandchild_idx = gc_idx
                                 break
                     
                     # 1. Highlight in current column (if it exists)
                     # Wait, we need to ensure Col[i] exists.
                     # Since we expand sequentially, Col[0] exists. 
                     # Col[i] creates Col[i+1]. So Col[i] should exist.
                     
                     # Find column widget
                     col_widget = None
                     for child in self.children:
                         if isinstance(child, Column) and child.index == i:
                             col_widget = child
                             break
                     
                     if col_widget:
                         # Manually update highlight
                         opt_list = col_widget.query_one(TwigOptionList)
                         opt_list.highlighted = select_idx
                         self.scroll_to_widget(col_widget)
                         
                         # 2. Expand
                         await self._expand_node(i, next_node.id, initial_select_index=grandchild_idx)
                         
             except Exception as e:
                 # Logic error or race condition
                 self.notify(f"Jump Error: {e}", severity="error")
                 break
                 
        # 3. Focus the final column
        final_col_idx = len(lineage) - 1
        target_col_idx = max(0, len(lineage) - 2)
        
        def focus_target():
             for child in self.children:
                 if isinstance(child, Column) and child.index == target_col_idx:
                     child.query_one(TwigOptionList).focus()
                     # Post selection message
                     self.post_message(self.NodeSelected(node_id))
                     break
        self.call_after_refresh(focus_target)

    async def find_next(self, query: str, direction: int = 1) -> bool:
        """
        Find in the child column(s) recursively if available.
        Prioritizes searching deeper into the visible path.
        """
        # 1. Identify currently focused column
        focused_col = None
        for child in self.children:
             if isinstance(child, Column) and child.query_one(TwigOptionList).has_focus:
                 focused_col = child
                 break
        
        if not focused_col:
            return False

        # 2. Iterate through subsequent columns (Child, Grandchild, etc.)
        start_index = focused_col.index + 1
        # Find max index
        max_index = start_index - 1
        visible_cols = {}
        for child in self.children:
            if isinstance(child, Column):
                visible_cols[child.index] = child
                if child.index > max_index:
                    max_index = child.index
        
        # Search from next column up to the end
        for idx in range(start_index, max_index + 1):
            col = visible_cols.get(idx)
            if col:
                found = await col.find_next(query, direction)
                if found:
                    col.query_one(TwigOptionList).focus()
                    self.scroll_to_widget(col)
                    # Trigger highlight update
                    opts = col.query_one(TwigOptionList)
                    if opts.highlighted is not None:
                         node_id = col.node_map.get(opts.highlighted)
                         if node_id:
                             self.post_message(self.NodeSelected(node_id))
                    return True

        # 3. Deep Search (Recursion)
        # Start from the LAST visible column.
        last_col = visible_cols.get(max_index)
        if last_col:
            opts = last_col.query_one(TwigOptionList)
            if opts.highlighted is not None:
                parent_id = last_col.node_map.get(opts.highlighted)
                if parent_id:
                    # Start recursion from here
                    found, path, match_idx = self._find_in_children(parent_id, query)
                    
                    if found:
                        current_col_idx = last_col.index
                        current_node_id = parent_id
                        
                        # Apply expansions
                        for choice_idx in path:
                             # Expand current node with choice_idx selected
                             await self._expand_node(current_col_idx, current_node_id, initial_select_index=choice_idx)
                             
                             # Get the node ID for the next step recursively
                             children = self.model.get_children(current_node_id)
                             if 0 <= choice_idx < len(children):
                                 current_node_id = children[choice_idx].id
                                 current_col_idx += 1
                             else:
                                 break
                        
                        # Expand the final container
                        await self._expand_node(current_col_idx, current_node_id, initial_select_index=match_idx)
                        
                        # Focus the final column
                        final_col_idx = current_col_idx + 1
                        def focus_final():
                            for child in self.children:
                                if isinstance(child, Column) and child.index == final_col_idx:
                                    child.query_one(TwigOptionList).focus()
                                    
                                    # Identify the matched node
                                    c_node_id = child.node_map.get(match_idx)
                                    if c_node_id:
                                        # 1. Update Inspector/Breadcrumbs
                                        self.post_message(self.NodeSelected(c_node_id))
                                        
                                        # 2. Trigger expansion of the NEXT level (children of the match)
                                        # We use run_worker because we are in a sync callback
                                        self.run_worker(self._expand_node(child.index, c_node_id, initial_select_index=0))
                                    break
                        self.call_after_refresh(focus_final)

                        return True

        return await focused_col.find_next(query, direction)

    async def on_mount(self) -> None:
        # Focus the first option list
        if self.children:
            first_col = self.children[0]
            if isinstance(first_col, Column):
                opt_list = first_col.query_one(TwigOptionList)
                opt_list.focus()
                # Manually trigger initial selection
                if opt_list.highlighted is not None:
                     node_id = first_col.node_map.get(opt_list.highlighted)
                     if node_id:
                         await self._expand_node(first_col.index, node_id, initial_select_index=0)

    async def on_column_highlighted(self, message: Column.Highlighted) -> None:
        """Handles selection within a column."""
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
                    target_option_list = target_col.query_one(TwigOptionList)
                    target_option_list.focus()
                    self.scroll_to_widget(target_col)
                    
                    # Manually trigger highlight/selection update for the newly focused column
                    # because OptionList doesn't re-emit highlighted on focus
                    if target_option_list.highlighted is not None:
                         node_id = target_col.node_map.get(target_option_list.highlighted)
                         if node_id:
                             # We simulate a highlight event to trigger expansion and inspector update
                             # Use run_worker because we are in a sync message handler
                             self.run_worker(self.on_column_highlighted(Column.Highlighted(target_col.index, node_id)))
                    return
