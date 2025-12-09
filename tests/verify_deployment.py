import asyncio
import unittest
from textual.pilot import Pilot
from twg.ui.app import TwigApp
from twg.core.model import TwigModel
import os

# Specific path to testing sample
SAMPLE_PATH = "samples/complex.json"
JUMP_TARGET_PATH = ".exhibits[0].animals[3].diet"

class TestTwigDeployment(unittest.IsolatedAsyncioTestCase):
    async def tests_full_feature_set(self):
        if not os.path.exists(SAMPLE_PATH):
            self.fail(f"Sample file {SAMPLE_PATH} not found.")

        self.app = TwigApp(SAMPLE_PATH)
        
        async with self.app.run_test() as pilot:
            # Wait for app to be fully mounted and data loaded
            await pilot.pause(0.5)

            # --- PHASE 1: JUMP VERIFICATION ---
            print("\n[TEST] 1. Triggering Jump to deep path logic directly...")
            
            # Logic extracted from action_jump's check_jump callback
            navigator = self.app.query_one("ColumnNavigator")
            node = self.app.model.resolve_path(JUMP_TARGET_PATH)
            if node:
                await navigator.expand_to_node(node.id)
            else:
                self.fail(f"Could not resolve path: {JUMP_TARGET_PATH}")
            
            # Wait for expansion
            await pilot.pause(0.5)
            
            # Verify Focus is on the correct Deep Column
            focused = self.app.screen.focused
            print(f"[TEST] 1. Post-Jump Focus: {focused}")
            
            # Debug: Print all columns
            cols = self.app.query("Column")
            print(f"[TEST] Existing Columns: {[c.id for c in cols]}")

            self.assertTrue(focused, "Focus should not be None after Jump")
            self.assertIn("col-4", str(focused.id), "Focus should be on Column 4 (Diet)")
            
            # --- PHASE 2: NAV REGRESSION TEST ---
            print("[TEST] 2. Testing Horizontal Navigation (Left)...")
            await pilot.press("left")
            await pilot.pause(0.1)
            
            focused = self.app.screen.focused
            print(f"[TEST] 2. Post-Left Focus: {focused}")
            self.assertIn("col-3", str(focused.id), "Should navigate left to Column 3 (Animal 3)")
            
            print("[TEST] 3. Testing Horizontal Navigation (Right)...")
            await pilot.press("right")
            await pilot.pause(0.1)
            
            focused = self.app.screen.focused
            print(f"[TEST] 3. Post-Right Focus: {focused}")
            self.assertIn("col-4", str(focused.id), "Should navigate right back to Column 4")

            # --- PHASE 3: SEARCH VERIFICATION ---
            print("[TEST] 4. Testing Search...")
            
            # Focus Root to ensure search starts from top
            col0 = self.app.query_one("#col-0")
            col0.focus()

            found = await navigator.find_next("zebra")
            await pilot.pause(0.5)
            
            print(f"[TEST] 4. Search Result: {found}")
            self.assertTrue(found, "Search should find 'zebra'")
            
            focused = self.app.screen.focused
            print(f"[TEST] 4. Post-Search Focus: {focused}")
            self.assertTrue("col-4" in str(focused.id) or "col-5" in str(focused.id), 
                            "Focus should be deep in the tree")

if __name__ == "__main__":
    unittest.main()
