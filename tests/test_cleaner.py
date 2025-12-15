
import unittest
import json
from twg.core.cleaner import repair_json

class TestCleaner(unittest.TestCase):
    def test_repair_json_valid(self):
        valid = '{"a": 1}'
        self.assertEqual(json.loads(repair_json(valid)), {"a": 1})

    def test_repair_json_trailing_comma(self):
        content = '{"a": 1,}'
        self.assertEqual(json.loads(repair_json(content)), {"a": 1})

    def test_repair_json_single_quotes(self):
        content = "{'a': 1}"
        self.assertEqual(json.loads(repair_json(content)), {"a": 1})

    def test_repair_json_unquoted_keys(self):
        content = '{a: 1}'
        self.assertEqual(json.loads(repair_json(content)), {"a": 1})

    def test_repair_json_missing_brace(self):
        content = '{"a": 1'
        self.assertEqual(json.loads(repair_json(content)), {"a": 1})

    def test_repair_json_missing_bracket_and_brace(self):
        content = '{"a": [1, 2'
        self.assertEqual(json.loads(repair_json(content)), {"a": [1, 2]})

    def test_repair_json_mixed_errors(self):
        # Trailing comma, single quotes, unquoted key, missing brace
        content = "{'a': [1, 2, ], b: 3"
        expected = {"a": [1, 2], "b": 3}
        self.assertEqual(json.loads(repair_json(content)), expected)

if __name__ == '__main__':
    unittest.main()
