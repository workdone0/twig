import yaml
import uuid
import sqlite3
import os
from pathlib import Path
from typing import Any, Callable, Optional  
from twg.core.model import DataType
from twg.adapters.base_loader import BaseLoader



class YamlLoader(BaseLoader):
    """
    Loads YAML file into SQLite database using PyYAML event stream.
    """
    
    def ingest_file(self, file_path: str, db_path: Path, progress_callback: Optional[Callable[[int, int], None]] = None) -> None:
        conn = self.db_manager.get_connection(db_path)
        
        try:
            total_size = os.path.getsize(file_path)
            
            with open(file_path, 'rb') as f:
                if progress_callback:
                    wrapped_f = ProgressFile(f, total_size, progress_callback)
                    self._parse_and_insert(wrapped_f, conn)
                else:
                    self._parse_and_insert(f, conn)
            
            conn.commit()
            
            # Index
            conn.execute("INSERT INTO nodes_search(rowid, key, value, path) SELECT rowid, key, value, path FROM nodes")
            conn.commit()
            
        except Exception as e:
            raise e
        finally:
            conn.close()

    def _parse_and_insert(self, file_obj, conn: sqlite3.Connection):
        """
        Parses YAML events and inserts into DB.
        """
        # Use CLoader if available for speed
        try:
            from yaml import CLoader as Loader
        except ImportError:
            from yaml import Loader

        # Get events
        # yaml.parse returns a generator of events
        events = yaml.parse(file_obj, Loader=Loader)
        
        def new_id(): return str(uuid.uuid4())
        
        root_id = new_id()
        
        # Stack: [node_id, type, count, path_prefix]
        stack = [] 
        
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
        
        current_key = None 
        in_document = False
        
        # Treat the stream as an array of documents (Virtual Root)
        virtual_root_type = DataType.ARRAY
        root_row = (root_id, None, "root", None, virtual_root_type.value, 0, ".", 0)
        rows.append(root_row)
        
        stack.append([root_id, virtual_root_type, 0, "."])
        
        for event in events:
            if isinstance(event, yaml.StreamStartEvent):
                continue
            if isinstance(event, yaml.StreamEndEvent):
                continue
                
            if isinstance(event, yaml.DocumentStartEvent):
                # Start of a new document
                continue
            
            if isinstance(event, yaml.DocumentEndEvent):
                continue
            
            # Now content events: MappingStart, SequenceStart, Scalar, Alias
            
            # Determine hierarchy context
            if not stack: break
            
            parent_id, parent_type, rank, parent_path = stack[-1]
            
            is_key_scalar = False
            
            if parent_type == DataType.OBJECT:
                if current_key is None:
                    # Expecting Key
                    if isinstance(event, yaml.ScalarEvent):
                        current_key = event.value
                        continue
                    elif isinstance(event, (yaml.MappingStartEvent, yaml.SequenceStartEvent)):
                        # Fallback for complex keys
                        current_key = f"<{type(event).__name__}>"
                        pass
                    elif isinstance(event, (yaml.MappingEndEvent, yaml.SequenceEndEvent)):
                         # End of map
                         stack.pop()
                         continue
                
            elif parent_type == DataType.ARRAY:
                 if isinstance(event, (yaml.SequenceEndEvent)):
                     stack.pop()
                     continue

            # Ensure we are processing a value
            if isinstance(event, (yaml.MappingEndEvent, yaml.SequenceEndEvent)):
                 stack.pop()
                 continue

            # Determine Node details
            if parent_type == DataType.ARRAY:
                 node_key = str(rank)
            else:
                 node_key = current_key
                 current_key = None # Reset for next pair
            
            # Path
            if parent_type == DataType.ARRAY:
                node_path = f"{parent_path}[{node_key}]"
            else:
                if parent_path == ".":
                    node_path = f".{node_key}"
                else:
                    node_path = f"{parent_path}.{node_key}"
            
            # Update parent rank
            stack[-1][2] += 1
            
            node_id = new_id()
            node_type = DataType.STRING
            val_str = None
            is_container = False
            
            if isinstance(event, yaml.ScalarEvent):
                # Detect type
                # PyYAML tags: event.tag (e.g. tag:yaml.org,2002:int)
                tag = event.tag
                val = event.value
                
                if tag and 'int' in tag:
                    node_type = DataType.INTEGER
                    val_str = str(val)
                elif tag and 'float' in tag:
                    node_type = DataType.FLOAT
                    val_str = str(val)
                elif tag and 'bool' in tag:
                    node_type = DataType.BOOLEAN
                    val_str = str(val).lower()
                elif tag and 'null' in tag:
                    node_type = DataType.NULL
                    val_str = "null"
                else:
                    # Infer if implicit
                    if event.style is None and event.implicit:
                        # minimalistic inference
                        if val.lower() in ('true', 'false'):
                            node_type = DataType.BOOLEAN
                            val_str = val.lower()
                        elif val.lower() in ('null', '~'):
                            node_type = DataType.NULL
                            val_str = "null"
                        elif val.isdigit():
                            node_type = DataType.INTEGER
                            val_str = val
                        else:
                            node_type = DataType.STRING
                            val_str = val
                    else:
                        node_type = DataType.STRING
                        val_str = val

            elif isinstance(event, yaml.MappingStartEvent):
                node_type = DataType.OBJECT
                is_container = True
            
            elif isinstance(event, yaml.SequenceStartEvent):
                node_type = DataType.ARRAY
                is_container = True
            
            elif isinstance(event, yaml.AliasEvent):
                node_type = DataType.STRING
                val_str = f"*alias:{event.anchor}"
            
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
