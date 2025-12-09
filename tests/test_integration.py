import asyncio
import unittest
import os
from textual.pilot import Pilot
from twg.ui.app import TwigApp

# Sample Data Path
SAMPLE_PATH = "samples/complex.json"

class TestTwigIntegration(unittest.IsolatedAsyncioTestCase):
    """
    Comprehensive Integration Test Suite for Twig.
    Covers: Jump, Navigation, Global Search, Smart Search, Directional Search, and Themes.
    """

    async def asyncSetUp(self):
        if not os.path.exists(SAMPLE_PATH):
            self.fail(f"Sample file {SAMPLE_PATH} not found.")
        self.app = TwigApp(SAMPLE_PATH)

    async def test_full_user_journey(self):
        """
        Simulates a complete user session exercising core features.
        Runs as a single flow to mimic real usage and state accumulation.
        """
        async with self.app.run_test() as pilot:
            await pilot.pause(0.5)
            navigator = self.app.query_one("ColumnNavigator")

            # ==========================================
            # 1. SMART SEARCH (Path Resolution)
            # ==========================================
            print("\n[TEST] 1. Smart Search (Path Jump)...")
            # Search for a specific deep path using the Search Bar
            await pilot.press("/")
            await pilot.press(*".exhibits[0].animals[3].diet") # 'Herbivore'
            await pilot.press("enter")
            
            # Wait for expansion (with buffer for asyncio.sleep in navigator)
            await pilot.pause(0.8) 
            
            # Verify focus
            focused = self.app.screen.focused
            print(f"       Focused: {focused}")
            # Should be in a deep column (col-4 typically for this depth)
            self.assertIn("col-4", str(focused.id), "Smart Search should jump to deep column")
            
            # Verify selection logic
            # We can't easily check the exact node without internal map access, 
            # but checking we are deep in the tree is a strong signal.

            # ==========================================
            # 2. NAVIGATION (Miller Columns)
            # ==========================================
            print("[TEST] 2. Navigation (Left/Right)...")
            # Move Left (Parent)
            await pilot.press("left")
            await pilot.pause(0.2)
            focused_left = self.app.screen.focused
            print(f"       Left Focus: {focused_left}")
            self.assertIn("col-3", str(focused_left.id), "Should move to parent column")
            
            # Move Right (Child) - should return to previous selection
            await pilot.press("right")
            await pilot.pause(0.2)
            focused_right = self.app.screen.focused
            print(f"       Right Focus: {focused_right}")
            self.assertIn("col-4", str(focused_right.id), "Should move back to child column")

            # ==========================================
            # 3. GLOBAL SEARCH & DIRECTION (Next/Prev)
            # ==========================================
            print("[TEST] 3. Global Search & Direction...")
            # Focus root to reset context
            col0 = self.app.query_one("#col-0")
            col0.focus()
            
            # Search for "species" - this appears multiple times in complex.json
            await pilot.press("/")
            await pilot.press(*"species")
            await pilot.press("enter")
            await pilot.pause(0.5)
            
            # Capture first match ID
            col_match_1 = self._get_focused_column(navigator)
            node_id_1 = self._get_selected_node_id(col_match_1)
            print(f"       Match 1 ID: {node_id_1}")
            
            # Find Next ('n')
            await pilot.press("n")
            await pilot.pause(0.5)
            
            col_match_2 = self._get_focused_column(navigator)
            node_id_2 = self._get_selected_node_id(col_match_2)
            print(f"       Match 2 ID: {node_id_2}")
            
            self.assertNotEqual(node_id_1, node_id_2, "Next (n) should move to a different node")
            
            # Find Previous ('N') - Should return to Match 1 (wrapping or direct prev)
            # Note: Depending on count, prev might be match 1.
            await pilot.press("N")
            await pilot.pause(0.5)
            
            col_match_prev = self._get_focused_column(navigator)
            node_id_prev = self._get_selected_node_id(col_match_prev)
            print(f"       Prev Match ID: {node_id_prev}")
            
            self.assertEqual(node_id_prev, node_id_1, "Prev (N) should return to previous match")

            # ==========================================
            # 4. THEME CYCLING
            # ==========================================
            print("[TEST] 4. Theme Cycling...")
            initial_theme = self.app.theme
            
            await pilot.press("t")
            new_theme = self.app.theme
            print(f"       Theme: {initial_theme} -> {new_theme}")
            
            self.assertNotEqual(initial_theme, new_theme, "Theme should change on pressing 't'")

            print("\n[SUCCESS] Integration Test Suite Passed.")

    def _get_focused_column(self, navigator):
        for child in navigator.children:
            if "Column" in str(type(child)) and child.query_one("TwigOptionList").has_focus:
                return child
        return None

    def _get_selected_node_id(self, column):
        if not column: return None
        opts = column.query_one("TwigOptionList")
        if opts.highlighted is not None:
             return column.node_map.get(opts.highlighted)
        return None

if __name__ == "__main__":
    unittest.main()
