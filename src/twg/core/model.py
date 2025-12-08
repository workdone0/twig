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
        """Generates child nodes from raw_value, using buckets if too large."""
        if node.expanded:
            return
            
        data = node.raw_value
        BUCKET_SIZE = 1000

        # Check if we are inside a list bucket to offset indices
        start_index = node.metadata.get("start_index", 0)

        if isinstance(data, list):
            size = len(data)
            if size > BUCKET_SIZE:
                # Create buckets
                for i in range(0, size, BUCKET_SIZE):
                    end = min(i + BUCKET_SIZE, size)
                    # Bucket key shows absolute range
                    # Note: indices in label should be relative to parent if parent is list?
                    # No, usually buckets are for the immediate large list.
                    # But if we had recursive buckets (list of lists of lists), 
                    # the start_index passed to _create_bucket handles the offset logic.
                    # Here 'i' is relative to 'data', which is the slice if 'node' is a bucket.
                    
                    # Absolute start index for the new bucket
                    abs_start = start_index + i
                    abs_end = start_index + end - 1
                    
                    bucket_key = f"[{abs_start} ... {abs_end}]"
                    bucket_data = data[i:end]
                    
                    self._create_bucket(node, bucket_key, bucket_data, start_index=abs_start)
            else:
                # Normal expansion
                for i, v in enumerate(data):
                    # Key is absolute index
                    self._create_child(node, str(start_index + i), v)

        elif isinstance(data, dict):
            size = len(data)
            if size > BUCKET_SIZE:
                 # Dict bucketization (more complex, need keys list)
                 keys = list(data.keys())
                 for i in range(0, size, BUCKET_SIZE):
                    end = min(i + BUCKET_SIZE, size)
                    
                    start_key = keys[i]
                    end_key = keys[end-1]
                    lbl_start = (start_key[:10] + '..') if len(start_key) > 10 else start_key
                    lbl_end = (end_key[:10] + '..') if len(end_key) > 10 else end_key
                    
                    bucket_key = f"{{{lbl_start} ... {lbl_end}}}"
                    
                    bucket_keys = keys[i:end]
                    bucket_data = {k: data[k] for k in bucket_keys}
                    
                    self._create_bucket(node, bucket_key, bucket_data)
            else:
                 for k, v in data.items():
                    self._create_child(node, k, v)
        
        node.expanded = True

    def _create_bucket(self, parent: Node, key: str, value: Any, start_index: int = 0) -> None:
        """Creates a virtual intermediate node."""
        # Buckets are containers (we treat them as Arrays or Objects depending on context, 
        # but simplistically ARRAY works well for the UI icon)
        child = Node(
            key=key,
            value=None,
            type=DataType.ARRAY if isinstance(value, list) else DataType.OBJECT,
            parent=parent.id,
            raw_value=value
        )
        # Store start_index in the node context somehow if we want children to have absolute indices?
        # A simple hack is to attach it to the instance dynamically or use a subclass.
        # For now, let's update _expand_node to handle "offset" if the parent was valid list bucket.
        # Actually, simpler: if raw_value is a slice, the child keys will range 0..N.
        # But user wants [1000]. So we need the offset.
        # Let's attach metadata to the node.
        child.metadata = {"start_index": start_index}
        self.add_node(child)

    def _create_child(self, parent: Node, key: str, value: Any) -> None:
        # If parent has start_index metadata and is a list bucket, offset the key?
        # Actually, if we pass the correct key from _expand_node, we are good.
        # But _expand_node calls _create_child with str(i) where i is 0..len(slice).
        # We need to fix _expand_node logic for children of buckets to respect offset.
        # Let's handle passing explicit keys in the caller.
        
        node_type = get_type_from_value(value)
        # For leaf nodes, value is the actual value. For containers, it's None (or could be summary)
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
