"""Regression checks for production snapshot decoding."""
import copy
import unittest
from modal_manage import unpack_terrain, verify_inventories


class TerrainTests(unittest.TestCase):
    def test_legacy_and_compressed_channels(self):
        self.assertEqual(unpack_terrain('..AAaaJL', 8), '..AAaaJL')
        self.assertEqual(unpack_terrain('~4:.Ab~2:_', 8), '....Ab__')
        self.assertEqual(unpack_terrain('~4:1~4:-', 8), '1111----')

    def test_expanded_map(self):
        size = 304 * 80
        self.assertEqual(len(unpack_terrain(f'~{size}:.', size)), size)

    def test_invalid_and_mismatched_channels(self):
        for value in ['~0:.', '~9:.', '~10:', '~x:a', '~2', 'é', '.......']:
            with self.subTest(value=value), self.assertRaises(RuntimeError):
                unpack_terrain(value, 8)


class InventoryTests(unittest.TestCase):
    def setUp(self):
        stock = dict.fromkeys("wood food stone gold iron coal clay fiber timber steel bricks cloth rations".split(), 0.0)
        self.state = {
            "island_origins": [{"column": 0, "row": 0}, {"column": 184, "row": 0}],
            "inventories": [dict(stock, wood=10.0), dict(stock, food=7.0)],
            "stored_inventories": [stock.copy(), dict(stock, food=7.0)],
        }

    def test_local_inventories_with_nearby_ship_cargo(self):
        verify_inventories(self.state)

    def test_shared_stockpile_is_not_current_snapshot(self):
        with self.assertRaises(RuntimeError):
            verify_inventories({"stockpile": self.state["inventories"][0]})

    def test_missing_island_catalog_or_invalid_balance(self):
        for field in ("inventories", "stored_inventories"):
            for invalid in (None, [], [{}], self.state[field][:1]):
                state = copy.deepcopy(self.state)
                state[field] = invalid
                with self.subTest(field=field, invalid=invalid), self.assertRaises(RuntimeError):
                    verify_inventories(state)
            for invalid in (-1.0, float("inf"), float("nan"), "10", True):
                state = copy.deepcopy(self.state)
                state[field][0]["wood"] = invalid
                with self.subTest(field=field, invalid=invalid), self.assertRaises(RuntimeError):
                    verify_inventories(state)
            state = copy.deepcopy(self.state)
            del state[field][0]["wood"]
            with self.assertRaises(RuntimeError):
                verify_inventories(state)


if __name__ == '__main__':
    unittest.main()
