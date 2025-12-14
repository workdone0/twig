import os
import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch
from twg.core.config import Config

class TestConfig(unittest.TestCase):
    def setUp(self):
        self.temp_dir = TemporaryDirectory()
        self.config_dir = Path(self.temp_dir.name) / "config"
        
        # Patch the _get_config_dir method to return our temp dir
        self.patcher = patch.object(Config, '_get_config_dir', return_value=self.config_dir)
        self.mock_get_dir = self.patcher.start()

    def tearDown(self):
        self.patcher.stop()
        self.temp_dir.cleanup()

    def test_default_config(self):
        config = Config()
        self.assertEqual(config.get("theme"), "catppuccin-mocha")

    def test_save_and_load(self):
        config = Config()
        config.set("theme", "dracula")
        
        # Create a new instance to verify persistence
        new_config = Config()
        self.assertEqual(new_config.get("theme"), "dracula")
        
        # Verify file content
        config_file = self.config_dir / "config.json"
        self.assertTrue(config_file.exists())
        with open(config_file, "r") as f:
            data = json.load(f)
            self.assertEqual(data["theme"], "dracula")

    def test_load_existing_file(self):
        # Create a config file manually
        self.config_dir.mkdir(parents=True)
        config_file = self.config_dir / "config.json"
        with open(config_file, "w") as f:
            json.dump({"theme": "solarized-light"}, f)
            
        config = Config()
        self.assertEqual(config.get("theme"), "solarized-light")

if __name__ == '__main__':
    unittest.main()
