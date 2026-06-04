"""CLI helper function tests."""

import sys
from pathlib import Path
from unittest.mock import patch

from modern_colorthief.cli import main, rgb_to_hex

BASE_DIR = Path(__file__).resolve().parent
TEST_IMAGE = BASE_DIR / "test.jpg"


# -- rgb_to_hex --

def test_rgb_to_hex_white():
    assert rgb_to_hex((255, 255, 255)) == "#ffffff"


def test_rgb_to_hex_black():
    assert rgb_to_hex((0, 0, 0)) == "#000000"


def test_rgb_to_hex_red():
    assert rgb_to_hex((255, 0, 0)) == "#ff0000"


def test_rgb_to_hex_single_digit_padding():
    assert rgb_to_hex((1, 2, 3)) == "#010203"


def test_rgb_to_hex_mixed():
    assert rgb_to_hex((179, 51, 55)) == "#b33337"


# -- CLI flags --

def test_cli_quality_flag(capsys):
    with patch.object(sys, "argv", ["modern_colorthief", str(TEST_IMAGE), "--quality", "5"]):
        main()
    assert capsys.readouterr().out.strip().startswith("#")


def test_cli_palette_count_flag(capsys):
    with patch.object(sys, "argv", ["modern_colorthief", str(TEST_IMAGE), "--palette", "--count", "3"]):
        main()
    lines = capsys.readouterr().out.strip().split("\n")
    assert len(lines) <= 3


def test_cli_hex_output_format(capsys):
    """Output is valid lowercase hex."""
    with patch.object(sys, "argv", ["modern_colorthief", str(TEST_IMAGE)]):
        main()
    out = capsys.readouterr().out.strip()
    assert out.startswith("#")
    assert len(out) == 7
    assert all(c in "0123456789abcdef" for c in out[1:])
