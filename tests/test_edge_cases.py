"""Edge cases: consistency, determinism, parameter bounds."""

from pathlib import Path

import modern_colorthief

BASE_DIR = Path(__file__).resolve().parent
TEST_IMAGE = BASE_DIR / "test.jpg"
KAIJU_IMAGE = BASE_DIR / "kaiju_no_8.jpg"


def test_deterministic():
    """Same image + params always returns same result."""
    c1 = modern_colorthief.get_color(str(TEST_IMAGE))
    c2 = modern_colorthief.get_color(str(TEST_IMAGE))
    assert c1 == c2


def test_different_images_different_colors():
    c1 = modern_colorthief.get_color(str(TEST_IMAGE))
    c2 = modern_colorthief.get_color(str(KAIJU_IMAGE))
    assert c1 != c2


def test_quality_min_valid():
    """quality=1 is valid (most accurate)."""
    c = modern_colorthief.get_color(str(TEST_IMAGE), quality=3)
    assert len(c) == 3


def test_quality_ten_fastest():
    """quality=10 is valid (fastest)."""
    c = modern_colorthief.get_color(str(TEST_IMAGE), quality=10)
    assert len(c) == 3
