import os
import json
import sys
from pathlib import Path
from typing import Any, Dict

class Config:
    """
    Handles loading and saving of application configuration.
    
    Configuration is stored in a JSON file in the standard user configuration directory:
    - Windows: %APPDATA%/twig/config.json
    - macOS/Linux: $XDG_CONFIG_HOME/twig/config.json (defaults to ~/.config/twig/config.json)
    """
    
    DEFAULT_CONFIG = {
        "theme": "catppuccin-mocha"
    }

    def __init__(self):
        self._config_dir = self._get_config_dir()
        self._config_file = self._config_dir / "config.json"
        self._data: Dict[str, Any] = self.DEFAULT_CONFIG.copy()
        self.load()

    def _get_config_dir(self) -> Path:
        """
        Returns the platform-specific configuration directory.
        """
        if sys.platform == "win32":
            app_data = os.environ.get("APPDATA")
            if app_data:
                return Path(app_data) / "twig"
            return Path.home() / "AppData" / "Roaming" / "twig"
        else:
            # macOS and Linux
            xdg_config = os.environ.get("XDG_CONFIG_HOME")
            if xdg_config:
                return Path(xdg_config) / "twig"
            return Path.home() / ".config" / "twig"

    def load(self) -> None:
        """
        Loads the configuration from disk. 
        If the file doesn't exist, keeps the default config.
        """
        if not self._config_file.exists():
            return

        try:
            with open(self._config_file, "r", encoding="utf-8") as f:
                user_config = json.load(f)
                # Update defaults with user config (merging)
                self._data.update(user_config)
        except (OSError, json.JSONDecodeError) as e:
            print(f"Warning: Failed to load config file: {e}", file=sys.stderr)

    def save(self) -> None:
        """
        Saves the current configuration to disk.
        Creates the directory if it doesn't exist.
        """
        try:
            self._config_dir.mkdir(parents=True, exist_ok=True)
            with open(self._config_file, "w", encoding="utf-8") as f:
                json.dump(self._data, f, indent=4)
        except OSError as e:
             print(f"Warning: Failed to save config file: {e}", file=sys.stderr)

    def get(self, key: str, default: Any = None) -> Any:
        """Get a configuration value."""
        return self._data.get(key, default)

    def set(self, key: str, value: Any) -> None:
        """Set a configuration value and save immediately."""
        self._data[key] = value
        self.save()
