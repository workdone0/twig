import sqlite3
import sys
import os
import hashlib
from pathlib import Path
from typing import Optional

class DatabaseManager:
    """
    Manages SQLite database connections and file locations.
    Uses a cache directory to store database files hashed by the input file path.
    """
    
    def __init__(self, debug: bool = False):
        self.debug = debug
        self._cache_dir = self._get_cache_dir()
        self._cache_dir.mkdir(parents=True, exist_ok=True)

    def _get_cache_dir(self) -> Path:
        """
        Returns the platform-specific cache directory.
        """
        if sys.platform == "win32":
            local_app_data = os.environ.get("LOCALAPPDATA")
            if local_app_data:
                return Path(local_app_data) / "twig"
            return Path.home() / "AppData" / "Local" / "twig"
        elif sys.platform == "darwin":
            return Path.home() / "Library" / "Caches" / "twig"
        else:
            # Linux / XDG
            xdg_cache = os.environ.get("XDG_CACHE_HOME")
            if xdg_cache:
                return Path(xdg_cache) / "twig"
            return Path.home() / ".cache" / "twig"

    def get_db_path(self, input_file_path: str) -> Path:
        """
        Generates a deterministic database file path for a given input file.
        Hash includes the absolute path and modification time/size could allow invalidation,
        but for now we just hash the path.
        
        To handle file updates, the loader should check mtime or we just overwrite if requested.
        """
        abs_path = str(Path(input_file_path).resolve())
        # Use simple md5 for filename
        file_hash = hashlib.md5(abs_path.encode("utf-8")).hexdigest()
        filename = f"{Path(input_file_path).name}_{file_hash}.db"
        return self._cache_dir / filename

    def get_connection(self, db_path: Path) -> sqlite3.Connection:
        """
        Returns a configured sqlite3 connection.
        """
        conn = sqlite3.connect(str(db_path), check_same_thread=False)
        conn.row_factory = sqlite3.Row
        
        # Performance tuning for bulk loads and reads
        conn.execute("PRAGMA journal_mode = WAL;")  # Write-Ahead Logging
        conn.execute("PRAGMA synchronous = NORMAL;") 
        conn.execute("PRAGMA temp_store = MEMORY;")
        conn.execute("PRAGMA cache_size = -64000;") # ~64MB cache
        
        return conn

    def init_db(self, db_path: Path) -> None:
        """
        Initializes the database schema.
        """
        schema_path = Path(__file__).parent / "schema.sql"
        if not schema_path.exists():
            raise FileNotFoundError("schema.sql not found")
            
        with open(schema_path, "r", encoding="utf-8") as f:
            schema_sql = f.read()
            
        conn = self.get_connection(db_path)
        try:
            conn.executescript(schema_sql)
            conn.commit()
        finally:
            conn.close()

    def clear_cache(self) -> None:
        """
        Clears all cached database files.
        """
        for file in self._cache_dir.glob("*.db"):
            try:
                file.unlink()
            except OSError:
                pass
                
        for file in self._cache_dir.glob("*.db-wal"):
            try:
                file.unlink()
            except OSError:
                pass
                
        for file in self._cache_dir.glob("*.db-shm"):
            try:
                file.unlink()
            except OSError:
                pass
