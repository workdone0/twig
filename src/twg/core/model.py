from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, List, Optional
import uuid

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
        value: The actual value of the node.
        type: The data type of the node.
        parent: UUID of the parent node.
        children: List of UUIDs of child nodes.
    """
    id: uuid.UUID = field(default_factory=uuid.uuid4)
    key: str = ""
    value: Any = None
    type: DataType = DataType.NULL
    parent: Optional[uuid.UUID] = None
    children: List[uuid.UUID] = field(default_factory=list)

    @property
    def is_container(self) -> bool:
        return self.type in (DataType.OBJECT, DataType.ARRAY)

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
            self.nodes[node.parent].children.append(node.id)

    def get_node(self, node_id: uuid.UUID) -> Optional[Node]:
        return self.nodes.get(node_id)

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
        # Reverse chain to go root -> leaf
        chain.reverse()
        
        parts = []
        for i, node in enumerate(chain):
            if i == 0: # Root
                continue
            
            parent = chain[i-1]
            if parent.type == DataType.ARRAY:
                parts.append(f"[{node.key}]")
            else:
                # Object property
                # TODO: Handle keys with special characters (add quotes)
                parts.append(f".{node.key}")
                
        path = "".join(parts) if parts else "."
        if not path.startswith("."):
            path = "." + path
        return path
