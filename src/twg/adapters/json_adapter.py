import ujson
from pathlib import Path
from typing import Any, Optional
import uuid

from twg.core.model import TwigModel, Node, DataType

class JsonAdapter:
    def load_into_model(self, file_path: str) -> TwigModel:
        path = Path(file_path)
        if not path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")
        
        with open(path, "r") as f:
            data = ujson.load(f)
            
        model = TwigModel()
        self._parse_recursive(model, data, key="root", parent_id=None)
        return model

    def _parse_recursive(self, model: TwigModel, data: Any, key: str, parent_id: Optional[uuid.UUID]) -> None:
        node_type = self._get_type(data)
        node = Node(
            key=key,
            value=data if not isinstance(data, (dict, list)) else None,
            type=node_type,
            parent=parent_id
        )
        
        if parent_id is None:
            model.root_id = node.id
            
        model.add_node(node)
        
        if isinstance(data, dict):
            for k, v in data.items():
                self._parse_recursive(model, v, key=k, parent_id=node.id)
        elif isinstance(data, list):
            for i, v in enumerate(data):
                self._parse_recursive(model, v, key=str(i), parent_id=node.id)

    def _get_type(self, data: Any) -> DataType:
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
