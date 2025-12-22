import ijson
import uuid
import sqlite3
import os
from pathlib import Path
from typing import Callable, Optional
from twg.core.model import DataType
from twg.adapters.base_loader import BaseLoader, ProgressFile

class SQLiteLoader(BaseLoader):
    """
    Loads JSON file into SQLite database using ijson streaming.
    """
    
    def __init__(self):
        super().__init__()

    def ingest_file(self, file_path: str, db_path: Path, progress_callback: Optional[Callable[[int, int], None]] = None) -> None:
        """
        Parses JSON and inserts into DB.
        """
        conn = self.db_manager.get_connection(db_path)
        
        try:
            # High-performance bulk load settings
            conn.execute("PRAGMA synchronous = OFF")
            conn.execute("PRAGMA journal_mode = MEMORY")
            
            self._drop_indexes(conn)
            conn.commit()
            
            with open(file_path, 'rb') as f:
                self._parse_and_insert(f, conn)
            
            conn.commit()
            
            self._rebuild_indexes(conn)
            conn.commit()
            
        except Exception as e:
             raise e
        finally:
            conn.close()

    def _drop_indexes(self, conn: sqlite3.Connection):
        """Drop indexes and FTS table to speed up insertion."""
        conn.executescript("""
            DROP INDEX IF EXISTS idx_parent_rank;
            DROP INDEX IF EXISTS idx_path;
            DROP TRIGGER IF EXISTS nodes_ai;
            DROP TRIGGER IF EXISTS nodes_ad;
            DROP TRIGGER IF EXISTS nodes_au;
            DROP TABLE IF EXISTS nodes_search;
        """)

    def _rebuild_indexes(self, conn: sqlite3.Connection):
        """Re-create indexes and FTS table after bulk insert."""
        conn.executescript("""
            -- Standard Indexes
            CREATE INDEX IF NOT EXISTS idx_parent_rank ON nodes(parent_id, rank);
            CREATE INDEX IF NOT EXISTS idx_path ON nodes(path);

            -- FTS5 Search Table
            CREATE VIRTUAL TABLE IF NOT EXISTS nodes_search USING fts5(
                key, 
                value, 
                path, 
                content='nodes', 
                content_rowid='rowid'
            );

            -- Populate FTS
            INSERT INTO nodes_search(rowid, key, value, path) 
            SELECT rowid, key, value, path FROM nodes;

            -- Triggers for updates
            CREATE TRIGGER IF NOT EXISTS nodes_ai AFTER INSERT ON nodes BEGIN
              INSERT INTO nodes_search(rowid, key, value, path) VALUES (new.rowid, new.key, new.value, new.path);
            END;

            CREATE TRIGGER IF NOT EXISTS nodes_ad AFTER DELETE ON nodes BEGIN
              INSERT INTO nodes_search(nodes_search, rowid, key, value, path) VALUES('delete', old.rowid, old.key, old.value, old.path);
            END;

            CREATE TRIGGER IF NOT EXISTS nodes_au AFTER UPDATE ON nodes BEGIN
              INSERT INTO nodes_search(nodes_search, rowid, key, value, path) VALUES('delete', old.rowid, old.key, old.value, old.path);
              INSERT INTO nodes_search(rowid, key, value, path) VALUES (new.rowid, new.key, new.value, new.path);
            END;
        """)

    def _parse_and_insert(self, file_obj, conn: sqlite3.Connection):
        """
        Uses ijson to stream events and build the tree.
        """
        # ijson parser
        parser = ijson.parse(file_obj)
        
        # Helper to generate UUIDs
        def new_id(): return str(uuid.uuid4())
        
        root_id = new_id()
        
        # Stack items: (node_id, type, count/index, path_prefix)
        stack = [] 
        
        # Generator for batch inserts
        # We need to collect rows and executemany
        batch_size = 10000
        rows = []
        
        cursor = conn.cursor()
        
        def flush():
            if rows:
                cursor.executemany(
                    "INSERT INTO nodes (id, parent_id, key, value, type, rank, path, is_expanded) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    rows
                )
                rows.clear()
        
        # ijson events stream
        first_event = True
        
        current_key = None # For map values
        
        for prefix, event, value in parser:
            
            if first_event:
                first_event = False
                # Determine root type
                root_type = DataType.NULL
                if event == 'start_map':
                    root_type = DataType.OBJECT
                elif event == 'start_array':
                    root_type = DataType.ARRAY
                else: 
                     # Primitive root
                     root_type = DataType.from_value(value)
                     
                root_row = (
                    root_id, 
                    None, 
                    "root", 
                    str(value) if not (event == 'start_map' or event == 'start_array') else None,
                    root_type.value,
                    0,
                    ".", # Root path
                    0
                )
                rows.append(root_row)
                
                if event in ('start_map', 'start_array'):
                    stack.append([root_id, root_type, 0, "."])
                continue
            
            # Processing children
            if not stack:
                 break
                 
            parent_id, parent_type, rank, parent_path = stack[-1]
            
            if event == 'end_map' or event == 'end_array':
                stack.pop()
                continue
            
            if event == 'map_key':
                current_key = value
                continue
            
            # Now we have a value (container start or primitive)
            
            # Determine Key
            if parent_type == DataType.ARRAY:
                node_key = str(rank)
            else:
                node_key = current_key
            
            # Determine Path
            if parent_type == DataType.ARRAY:
                node_path = f"{parent_path}[{node_key}]"
            else:
                if parent_path == ".":
                    node_path = f".{node_key}"
                else:
                    node_path = f"{parent_path}.{node_key}"
            
            # Update rank for parent
            stack[-1][2] += 1
            
            # Determine Type
            node_type = DataType.STRING # Default
            val_str = None
            
            is_container = False
            
            if event == 'start_map':
                node_type = DataType.OBJECT
                is_container = True
            elif event == 'start_array':
                node_type = DataType.ARRAY
                is_container = True
            elif event == 'boolean':
                node_type = DataType.BOOLEAN
                val_str = str(value)
            elif event == 'integer':
                node_type = DataType.INTEGER
                val_str = str(value)
            elif event == 'double': # ijson uses 'double' for floats
                node_type = DataType.FLOAT
                val_str = str(value)
            elif event == 'number': 
                node_type = DataType.FLOAT # generic number
                val_str = str(value)
            elif event == 'string':
                node_type = DataType.STRING
                val_str = value
            elif event == 'null':
                node_type = DataType.NULL
                val_str = "null"
            
            node_id = new_id()
            
            rows.append((
                node_id,
                parent_id,
                node_key,
                val_str,
                node_type.value,
                rank,
                node_path,
                0
            ))
            
            if len(rows) >= batch_size:
                flush()
                
            if is_container:
                stack.append([node_id, node_type, 0, node_path])
        
        flush()
