from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, List, Optional
import uuid
import re

class DataType(Enum):
    OBJECT = "object"
    ARRAY = "array"
    STRING = "string"
    INTEGER = "integer"
    FLOAT = "float"
    BOOLEAN = "boolean"
    NULL = "null"

@dataclass
class Node:
    """
    Represents a single node in the JSON tree.
    
    Attributes:
        id: Unique identifier for the node.
        key: The key (for objects) or index (for arrays) of this node.
        value: The value to display (leaf value or container summary).
        type: The data type of the node.
        parent: UUID of the parent node.
        children: List of UUIDs of child nodes.
        raw_value: The raw Python object this node represents (for lazy loading).
        expanded: Whether children have been generated.
    """
    id: uuid.UUID = field(default_factory=uuid.uuid4)
    key: str = ""
    value: Any = None
    type: DataType = DataType.NULL
    parent: Optional[uuid.UUID] = None
    children: List[uuid.UUID] = field(default_factory=list)
    raw_value: Any = field(default=None, repr=False)
    expanded: bool = False
    metadata: Dict[str, Any] = field(default_factory=dict)

    @property
    def is_container(self) -> bool:
        return self.type in (DataType.OBJECT, DataType.ARRAY)

def get_type_from_value(data: Any) -> DataType:
    if data is None:
        return DataType.NULL
    elif isinstance(data, bool):
        return DataType.BOOLEAN
    elif isinstance(data, int):
        return DataType.INTEGER
    elif isinstance(data, float):
        return DataType.FLOAT
    elif isinstance(data, str):
        return DataType.STRING
    elif isinstance(data, dict):
        return DataType.OBJECT
    elif isinstance(data, list):
        return DataType.ARRAY
    return DataType.STRING

class TwigModel:
    """
    The data model for the application.
    
    Stores all nodes in a flat dictionary keyed by UUID for O(1) access.
    Manages the tree structure and provides helper methods for traversal.
    """
    def __init__(self):
        self.nodes: Dict[uuid.UUID, Node] = {}
        self.root_id: Optional[uuid.UUID] = None

    def add_node(self, node: Node) -> None:
        self.nodes[node.id] = node
        if node.parent and node.parent in self.nodes:
            # Only append if not already present (sanity check)
            parent = self.nodes[node.parent]
            if node.id not in parent.children:
                parent.children.append(node.id)

    def get_node(self, node_id: uuid.UUID) -> Optional[Node]:
        return self.nodes.get(node_id)

    def get_children(self, node_id: uuid.UUID) -> List[Node]:
        """Returns the list of child nodes. Generates them if not already expanded."""
        node = self.get_node(node_id)
        if not node:
            return []
        
        if not node.expanded and node.is_container:
            self._expand_node(node)
            
        return [self.get_node(child_id) for child_id in node.children if self.get_node(child_id)]

    def _expand_node(self, node: Node) -> None:
        """Generates child nodes from raw_value."""
        if node.expanded:
            return
            
        data = node.raw_value

        if isinstance(data, list):
            for i, v in enumerate(data):
                self._create_child(node, str(i), v)

        elif isinstance(data, dict):
             for k, v in data.items():
                self._create_child(node, k, v)
        
        node.expanded = True

    def _create_child(self, parent: Node, key: str, value: Any) -> None:
        
        node_type = get_type_from_value(value)
        display_value = value if node_type not in (DataType.OBJECT, DataType.ARRAY) else None
        
        child = Node(
            key=key,
            value=display_value,
            type=node_type,
            parent=parent.id,
            raw_value=value
        )
        self.add_node(child)

    def get_path(self, node_id: uuid.UUID) -> str:
        """Returns the jq-style path to the node."""
        node = self.get_node(node_id)
        if not node:
            return ""
        
        # 1. Collect nodes from leaf to root
        chain = []
        curr = node
        while curr:
            chain.append(curr)
            curr = self.get_node(curr.parent) if curr.parent else None
        
        # 2. Build path from root to leaf
        chain.reverse()
        
        parts = []
        for i, node in enumerate(chain):
            if i == 0: # Root
                continue
            
            parent = chain[i-1]
            if parent.type == DataType.ARRAY:
                parts.append(f"[{node.key}]")
            else:
                parts.append(f".{node.key}")
                
        path = "".join(parts) if parts else "."
        if not path.startswith("."):
            path = "." + path
        return path

    def find_next_node(self, query: str, start_node_id: Optional[uuid.UUID] = None, direction: int = 1) -> Optional[Node]:
        """
        Global search for next/prev node. 
        Collects all matches and traverses relative to start_node_id.
        Direction: 1 (Next), -1 (Prev).
        """
        if not self.root_id:
            return None
        
        query = query.lower()
        
        # Generator to iterate all nodes in order
        def traverse(node_id: uuid.UUID):
            node = self.nodes.get(node_id)
            if not node: return
            
            yield node
            
            # Force expansion for search completeness (since we dropped lazy load limits)
            if node.is_container and not node.expanded:
                 self._expand_node(node)
                 
            for child_id in node.children:
                yield from traverse(child_id)

        matches = []
        count = 0
        
        for node in traverse(self.root_id):
            count += 1
            
            match = query in str(node.key).lower()
            if not match and not node.is_container:
                match = query in str(node.value).lower()
            
            if match:
                matches.append(node)
        
        if not matches:
             return None

        # 2. Find start index
        current_idx = -1
        if start_node_id:
            for i, m in enumerate(matches):
                if m.id == start_node_id:
                    current_idx = i
                    break
        
        # 3. Calculate target index
        if current_idx == -1:
            # If start not found (or None), start at beginning (if Next) or End (if Prev)
            if direction > 0:
                target_idx = 0
            else:
                target_idx = len(matches) - 1
        else:
            target_idx = (current_idx + direction) % len(matches)
            
        return matches[target_idx]

    def resolve_path(self, path: str) -> Optional[Node]:
        """
        Resolves a jq-style path (e.g. .users[0].name) to a Node (UUID).
        Raises ValueError if path syntax is invalid.
        """
        segments = self._parse_jq_path(path)
        
        path = path.strip()
        if not path:
             raise ValueError("Path cannot be empty")
             
        if path != "." and not segments:
             raise ValueError(f"Invalid jq path syntax: {path}")
        
        if not self.root_id:
            return None
            
        current_node = self.get_node(self.root_id)
        if not current_node:
            return None
            
        if not segments:
             return current_node
            
        # Iterate through segments
        seg_idx = 0
        while seg_idx < len(segments):
            segment = segments[seg_idx]
            
            # Ensure children are expanded
            children = self.get_children(current_node.id)
            
            found = False
            
            # Try finding direct match
            for child in children:
                # Compare keys. 
                if str(child.key) == str(segment):
                    current_node = child
                    found = True
                    seg_idx += 1 # Consumed segment
                    break
            
            if not found:
                return None
            
        return current_node

    def _parse_jq_path(self, path: str) -> List[str | int]:
        """Parses a jq path string into specific segments."""
        path = path.strip()
        if path == "." or path == "":
            return []
            
        # Correct regex for finding segments:
        matches = re.finditer(r'(?:\.|^)([^.\[\]]+)|\[(\d+)\]|\["([^"]+)"\]|\[\'([^\']+)\'\]', path)
        results = []
        for m in matches:
             if m.group(1): # key (no quotes)
                 k = m.group(1)
                 # Handle odd case where path starts with .key, group 1 is key.
                 # If path starts with ., the first match might be empty group 1 if regex isn't careful.
                 # My regex: (?:\.|^)([^.\[\]]+)
                 # .foo -> matches .foo -> group 1 is foo.
                 results.append(k)
             elif m.group(2): # Int
                 results.append(int(m.group(2)))
             elif m.group(3): # Double quote
                 results.append(m.group(3))
             elif m.group(4): # Single quote
                 results.append(m.group(4))
                 
        return results
