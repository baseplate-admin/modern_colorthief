"""Input type dispatch: str, bytes, BytesIO all work correctly."""

import io
from pathlib import Path

import modern_colorthief

BASE_DIR = Path(__file__).resolve().parent
TEST_IMAGE = BASE_DIR / "test.jpg"


def test_str_input_color():
    c = modern_colorthief.get_color(str(TEST_IMAGE))
    assert len(c) == 3


def test_bytes_input_color():
    with open(TEST_IMAGE, "rb") as f:
        c = modern_colorthief.get_color(f.read())
    assert len(c) == 3


def test_bytesio_input_color():
    with open(TEST_IMAGE, "rb") as f:
        c = modern_colorthief.get_color(io.BytesIO(f.read()))
    assert len(c) == 3


def test_str_input_palette():
    p = modern_colorthief.get_palette(str(TEST_IMAGE))
    assert len(p) > 0


def test_bytes_input_palette():
    with open(TEST_IMAGE, "rb") as f:
        p = modern_colorthief.get_palette(f.read())
    assert len(p) > 0


def test_bytesio_input_palette():
    with open(TEST_IMAGE, "rb") as f:
        p = modern_colorthief.get_palette(io.BytesIO(f.read()))
    assert len(p) > 0


def test_bytes_not_mutated():
    """Input bytes are not modified."""
    with open(TEST_IMAGE, "rb") as f:
        original = f.read()
    modern_colorthief.get_color(original)
    modern_colorthief.get_palette(original)
    with open(TEST_IMAGE, "rb") as f:
        assert original == f.read()


def test_bytesio_no_seek_required():
    """BytesIO.getvalue() works without seek(0)."""
    with open(TEST_IMAGE, "rb") as f:
        data = f.read()
    bio = io.BytesIO(data)
    bio.read()  # exhaust stream
    c = modern_colorthief.get_color(bio)
    assert len(c) == 3
