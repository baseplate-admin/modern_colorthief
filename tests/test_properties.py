"""Property-based tests: return value structure and invariants."""

from pathlib import Path

import modern_colorthief

BASE_DIR = Path(__file__).resolve().parent
TEST_IMAGE = BASE_DIR / "test.jpg"


def test_color_returns_valid_rgb():
    """get_color returns a 3-tuple of ints in [0, 255]."""
    c = modern_colorthief.get_color(str(TEST_IMAGE))
    assert isinstance(c, tuple) and len(c) == 3
    assert all(isinstance(x, int) and 0 <= x <= 255 for x in c)


def test_palette_returns_valid_rgb_list():
    """get_palette returns list of valid RGB tuples."""
    p = modern_colorthief.get_palette(str(TEST_IMAGE))
    assert isinstance(p, list) and len(p) > 0
    for color in p:
        assert isinstance(color, tuple) and len(color) == 3
        assert all(isinstance(x, int) and 0 <= x <= 255 for x in color)


def test_palette_deduplicated():
    """Palette contains no duplicate colors."""
    p = modern_colorthief.get_palette(str(TEST_IMAGE))
    assert len(p) == len(set(p))


def test_palette_count_bounded():
    """Palette length does not exceed requested color_count."""
    for count in [3, 5]:
        p = modern_colorthief.get_palette(str(TEST_IMAGE), color_count=count)
        assert len(p) <= count
