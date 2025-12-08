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

    def __init__(self, model: TwigModel, parent_node_id: uuid.UUID, index: int):
        self.model = model
        self.parent_node_id = parent_node_id
        self.index = index
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
        """Select the first item by default when the column is mounted."""
        opt_list = self.query_one(TwigOptionList)
        if opt_list.option_count > 0:
            opt_list.highlighted = 0

    def on_option_list_option_highlighted(self, event: OptionList.OptionHighlighted) -> None:
        # Only emit highlighted event if this column has focus.
        # This prevents recursive expansion when new columns are mounted.
        if self.query_one(TwigOptionList).has_focus:
            node_id = self.node_map.get(event.option_index)
            if node_id:
                self.post_message(self.Highlighted(self.index, node_id))


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
        self.columns: List[Column] = []
        super().__init__()

    def compose(self) -> ComposeResult:
        # Start with the root column
        if self.model.root_id:
            yield Column(self.model, self.model.root_id, 0)

    def on_mount(self) -> None:
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
                         self.on_column_highlighted(Column.Highlighted(first_col.index, node_id))

    def on_column_highlighted(self, message: Column.Highlighted) -> None:
        """
        Handles selection within a column.
        
        Removes all columns to the right of the selected column, and if the selected
        node is a container, spawns a new column for it.
        """
        # 1. Remove all columns to the right
        to_remove = [child for child in self.children if isinstance(child, Column) and child.index > message.column_index]
        for child in to_remove:
            child.remove()
        
        # 2. Check if the selected node is a container
        node = self.model.get_node(message.node_id)
        if node and node.is_container:
            # 3. Add new column for the selected node
            new_col_index = message.column_index + 1
            new_col = Column(self.model, node.id, new_col_index)
            self.mount(new_col)
            # Scroll to the new column but DO NOT steal focus yet (Finder style)
            self.call_after_refresh(self.scroll_to_widget, new_col)
        
        # 4. Notify app about selection
        self.post_message(self.NodeSelected(message.node_id))

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
                             self.on_column_highlighted(Column.Highlighted(target_col.index, node_id))
                    return
