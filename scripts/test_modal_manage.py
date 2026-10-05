"""Regression checks for production snapshot decoding."""
import unittest
from modal_manage import unpack_terrain


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


if __name__ == '__main__':
    unittest.main()
