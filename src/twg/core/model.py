from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, List, Optional, Generator
import uuid
import re
import sqlite3
from twg.core.db import DatabaseManager

class DataType(Enum):
    OBJECT = "object"
    ARRAY = "array"
    STRING = "string"
    INTEGER = "integer"
    FLOAT = "float"
    BOOLEAN = "boolean"
    NULL = "null"
    
    @staticmethod
    def from_value(data: Any) -> DataType:
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

@dataclass
class Node:
    """
    Lightweight Node representation from SQLite.
    Does not store children or raw values in memory.
    """
    id: uuid.UUID
    key: str
    value: Any
    type: DataType
    parent: Optional[uuid.UUID]
    path: str
    is_expanded: bool = False
    
    @property
    def is_container(self) -> bool:
        return self.type in (DataType.OBJECT, DataType.ARRAY)

class SQLiteModel:
    """
    SQLite-backed data model.
    """
    def __init__(self, db_path: str):
        self.db_manager = DatabaseManager()
        self.db_path = db_path
        # We keep a connection for read operations
        self.conn = self.db_manager.get_connection(self.db_path)
        
        # Cache root id to avoid query
        self.root_id = self._fetch_root_id()

    def _fetch_root_id(self) -> Optional[uuid.UUID]:
        cursor = self.conn.execute("SELECT id FROM nodes WHERE parent_id IS NULL LIMIT 1")
        row = cursor.fetchone()
        return uuid.UUID(row["id"]) if row else None

    def close(self):
        self.conn.close()

    def row_to_node(self, row: sqlite3.Row) -> Node:
        # Convert type string back to Enum
        try:
             # simple lookup by value
             dtype = DataType(row["type"])
        except ValueError:
             dtype = DataType.STRING
             
        # Deserialize value from DB (stored as TEXT)
        # We attempt to cast back to native types where appropriate
        val = row["value"]
        if dtype == DataType.NULL:
            val = None
        elif dtype == DataType.BOOLEAN:
            val = (val == "True" or val == "true")
        elif dtype == DataType.INTEGER:
            try: val = int(val)
            except: pass
        elif dtype == DataType.FLOAT:
             try: val = float(val)
             except: pass
             
        return Node(
            id=uuid.UUID(row["id"]),
            key=row["key"],
            value=val,
            type=dtype,
            parent=uuid.UUID(row["parent_id"]) if row["parent_id"] else None,
            path=row["path"],
            is_expanded=bool(row["is_expanded"])
        )

    def get_node(self, node_id: uuid.UUID) -> Optional[Node]:
        cursor = self.conn.execute(
            "SELECT * FROM nodes WHERE id = ?", 
            (str(node_id),)
        )
        row = cursor.fetchone()
        if row:
            return self.row_to_node(row)
        return None

    def get_children(self, node_id: uuid.UUID) -> List[Node]:
        """Returns ordered children of the node."""
        cursor = self.conn.execute(
            "SELECT * FROM nodes WHERE parent_id = ? ORDER BY rank", 
            (str(node_id),)
        )
        return [self.row_to_node(row) for row in cursor.fetchall()]

    def get_children_count(self, node_id: uuid.UUID) -> int:
        """Returns the number of children."""
        cursor = self.conn.execute(
            "SELECT COUNT(*) FROM nodes WHERE parent_id = ?", 
            (str(node_id),)
        )
        return cursor.fetchone()[0]

    def get_path(self, node_id: uuid.UUID) -> str:
        """Returns the jq-style path from the DB column."""
        node = self.get_node(node_id)
        if node:
            return node.path
        return ""

    def find_next_node(self, query: str, start_node_id: Optional[uuid.UUID] = None, direction: int = 1) -> Optional[Node]:
        """
        Global search using substring match (LIKE %query%).
        
        Searches 'key' and 'value' columns.
        Results are ordered by path to provide a consistent navigation order.
        """
        query = query.strip()
        if not query: return None
        
        # Substring pattern
        like_pattern = f"%{query}%"
        start_path = None

        if start_node_id:
            n = self.get_node(start_node_id)
            if n:
                start_path = n.path
        
        # We search KEY and VALUE columns.
        # Note: 'value' column stores primitives as strings.
        
        if start_path:
            if direction > 0:
                 # Find next match after current path
                 sql_next = """
                    SELECT * FROM nodes 
                    WHERE (key LIKE ? OR value LIKE ?) AND path > ?
                    ORDER BY path ASC LIMIT 1
                 """
                 cursor = self.conn.execute(sql_next, (like_pattern, like_pattern, start_path))
                 row = cursor.fetchone()
                 if row: return self.row_to_node(row)
                 
                 # Wrap around: Find first match from start
                 sql_first = """
                    SELECT * FROM nodes 
                    WHERE (key LIKE ? OR value LIKE ?)
                    ORDER BY path ASC LIMIT 1
                 """
                 cursor = self.conn.execute(sql_first, (like_pattern, like_pattern))
                 row = cursor.fetchone()
                 if row: return self.row_to_node(row)
                 
            else:
                 # Find prev match before current path
                 sql_prev = """
                    SELECT * FROM nodes 
                    WHERE (key LIKE ? OR value LIKE ?) AND path < ?
                    ORDER BY path DESC LIMIT 1
                 """
                 cursor = self.conn.execute(sql_prev, (like_pattern, like_pattern, start_path))
                 row = cursor.fetchone()
                 if row: return self.row_to_node(row)
                 
                 # Wrap around: Find last match
                 sql_last = """
                    SELECT * FROM nodes 
                    WHERE (key LIKE ? OR value LIKE ?)
                    ORDER BY path DESC LIMIT 1
                 """
                 cursor = self.conn.execute(sql_last, (like_pattern, like_pattern))
                 row = cursor.fetchone()
                 if row: return self.row_to_node(row)
                 
        else:
             # No context, just return first
             sql_first = """
                SELECT * FROM nodes 
                WHERE (key LIKE ? OR value LIKE ?)
                ORDER BY path ASC LIMIT 1
             """
             cursor = self.conn.execute(sql_first, (like_pattern, like_pattern))
             row = cursor.fetchone()
             if row: return self.row_to_node(row)

        return None

    def resolve_path(self, path: str) -> Optional[Node]:
        """
        Direct lookup by path column.
        """
        path = path.strip()
        if not path: return None
        if not path.startswith("."): 
             # Normalize paths to always start with dot (except empty, handled above)
             path = "." + path 

        # Exact match
        cursor = self.conn.execute("SELECT * FROM nodes WHERE path = ?", (path,))
        row = cursor.fetchone()
        if row:
            return self.row_to_node(row)
            
        # Smart Fallback for single-document YAMLs (wrapped in Stream array)
        # If user types .kind, but it's actually .[0].kind
        if path.startswith("."):
            fallback_path = ".[0]" + path[1:] # .kind -> .[0].kind
            cursor = self.conn.execute("SELECT * FROM nodes WHERE path = ?", (fallback_path,))
            row = cursor.fetchone()
            if row:
                return self.row_to_node(row)

        return None

    def get_search_stats(self, query: str, current_node_id: Optional[uuid.UUID]) -> tuple[int, int]:
        """
        Returns (current_match_index, total_matches) for the given query.
        Index is 1-based. Returns (0, 0) if no matches or query empty.
        """
        query = query.strip()
        if not query: return (0, 0)
        
        like_pattern = f"%{query}%"
        
        sql_count = "SELECT COUNT(*) FROM nodes WHERE key LIKE ? OR value LIKE ?"
        cursor = self.conn.execute(sql_count, (like_pattern, like_pattern))
        total = cursor.fetchone()[0]
        
        if total == 0:
            return (0, 0)
            
        current_index = 0
        if current_node_id:
             node = self.get_node(current_node_id)
             if node:
                 # Count how many matches have a path <= current node's path
                 sql_rank = """
                    SELECT COUNT(*) 
                    FROM nodes 
                    WHERE (key LIKE ? OR value LIKE ?) AND path <= ?
                 """
                 cursor = self.conn.execute(sql_rank, (like_pattern, like_pattern, node.path))
                 current_index = cursor.fetchone()[0]
                 
        return (current_index, total)

    def reconstruct_json(self, node_id: uuid.UUID, max_depth: int = 3, current_depth: int = 0) -> Any:
        """
        Reconstructs the JSON value for a given node.
        Limits depth to prevent excessive loading.
        """
        node = self.get_node(node_id)
        if not node: return None
        
        if not node.is_container:
            return node.value
            
        if current_depth >= max_depth:
            return "..."
            
        children = self.get_children(node_id)
        
        if node.type == DataType.OBJECT:
            res = {}
            for child in children:
                res[child.key] = self.reconstruct_json(child.id, max_depth, current_depth + 1)
            return res
            
        elif node.type == DataType.ARRAY:
            res = []
            for child in children:
                res.append(self.reconstruct_json(child.id, max_depth, current_depth + 1))
            return res
            
        return None
