from abc import ABC, abstractmethod
from pathlib import Path
from typing import Tuple, Callable
from twg.core.model import SQLiteModel
from twg.core.db import DatabaseManager

class ProgressFile:
    """Wrapper around a file object to track bytes read."""
    def __init__(self, file_obj, total_size: int, callback: Callable[[int, int], None]):
        self.file_obj = file_obj
        self.total_size = total_size
        self.callback = callback
        self.bytes_read = 0
        self._last_report = 0
        self._report_interval = 1024 * 1024  # Report every 1MB

    def read(self, size=-1):
        data = self.file_obj.read(size)
        if data:
            self.bytes_read += len(data)
            if self.bytes_read - self._last_report > self._report_interval or self.bytes_read >= self.total_size:
                 self.callback(self.bytes_read, self.total_size)
                 self._last_report = self.bytes_read
        return data

    def __getattr__(self, name):
        return getattr(self.file_obj, name)

class BaseLoader(ABC):
    """
    Abstract base class for file loaders (JSON, YAML, etc).
    """
    def __init__(self):
        self.db_manager = DatabaseManager()

    def prepare_db(self, file_path: str, force_rebuild: bool = False) -> Tuple[Path, bool]:
        """
        Common logic to prepare the DB file.
        Returns (db_path, needs_ingestion).
        """
        path = Path(file_path)
        if not path.exists():
             raise FileNotFoundError(f"File not found: {file_path}")
             
        db_path = self.db_manager.get_db_path(file_path)
        
        # Check if DB already exists
        if db_path.exists() and not force_rebuild:
            try:
                # Simple check if valid
                conn = self.db_manager.get_connection(db_path)
                cursor = conn.execute("SELECT count(*) FROM nodes")
                # verify it has data
                count = cursor.fetchone()[0]
                conn.close()
                if count > 0:
                    return db_path, False
            except:
                pass # Rebuild if check fails
        
        # Init new DB
        if db_path.exists():
            try:
                db_path.unlink()
            except OSError:
                pass
                 
        self.db_manager.init_db(db_path)
        return db_path, True

    @abstractmethod
    def ingest_file(self, file_path: str, db_path: Path, progress_callback: Callable[[int, int], None] | None = None) -> None:
        """
        Parse file and insert into DB. To be implemented by subclasses.
        """
        pass

    def load_into_model(self, file_path: str, force_rebuild: bool = False) -> SQLiteModel:
        """
        Synchronous helper.
        """
        db_path, needs_ingestion = self.prepare_db(file_path, force_rebuild)
        if needs_ingestion:
            self.ingest_file(file_path, db_path)
            
        return SQLiteModel(str(db_path))
