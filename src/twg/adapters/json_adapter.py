import ujson
from pathlib import Path
from typing import Any, Optional
import uuid

from twg.core.model import TwigModel, Node, DataType, get_type_from_value

class JsonAdapter:
    def load_into_model(self, file_path: str) -> TwigModel:
        path = Path(file_path)
        if not path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")
        
        if not path.is_file():
             raise IsADirectoryError(f"Path is a directory, not a file: {file_path}")

        try:
            with open(path, "r", encoding="utf-8") as f:
                data = ujson.load(f)
        except ValueError as e:
            # ujson raises ValueError for invalid JSON
            raise ValueError(f"Invalid JSON file: {e}")
        except PermissionError:
            raise PermissionError(f"Permission denied: {file_path}")
        except UnicodeDecodeError:
             raise ValueError("File is not a valid UTF-8 text file.")
        except Exception as e:
            # Catch-all for any other IO issues
            raise RuntimeError(f"Failed to load file: {e}")
            
        model = TwigModel()
        
        # Create root node directly
        root_data = data
        root_type = get_type_from_value(root_data)
        
        root_node = Node(
            key="root",
            value=None, # Root container doesn't show a value
            type=root_type,
            parent=None,
            raw_value=root_data
        )
        
        model.root_id = root_node.id
        model.add_node(root_node)
        
        return model
