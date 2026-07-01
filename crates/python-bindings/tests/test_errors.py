"""Error handling: invalid inputs, corrupt data, unsupported types."""

import io
from pathlib import Path

import pytest

import modern_colorthief

BASE_DIR = Path(__file__).resolve().parent


# -- Invalid paths --

def test_nonexistent_file_color():
    with pytest.raises(ValueError):
        modern_colorthief.get_color("does_not_exist.jpg")


def test_nonexistent_file_palette():
    with pytest.raises(ValueError):
        modern_colorthief.get_palette("does_not_exist.jpg")


def test_directory_instead_of_file():
    with pytest.raises(ValueError):
        modern_colorthief.get_color("/tmp")


def test_error_message_contains_path():
    with pytest.raises(ValueError, match="no_such_image.png"):
        modern_colorthief.get_color("no_such_image.png")


# -- Invalid bytes --

def test_empty_bytes_color():
    with pytest.raises(ValueError):
        modern_colorthief.get_color(b"")


def test_empty_bytes_palette():
    with pytest.raises(ValueError):
        modern_colorthief.get_palette(b"")


def test_text_as_bytes():
    with pytest.raises(ValueError):
        modern_colorthief.get_color(b"this is not an image")


def test_truncated_jpeg():
    with pytest.raises(ValueError):
        modern_colorthief.get_color(b"\xff\xd8\xff\xe0" + b"\x00" * 10)


def test_empty_bytesio_color():
    with pytest.raises(ValueError):
        modern_colorthief.get_color(io.BytesIO(b""))


def test_empty_bytesio_palette():
    with pytest.raises(ValueError):
        modern_colorthief.get_palette(io.BytesIO(b""))


# -- Quality bounds --

def test_quality_one_valid():
    color = modern_colorthief.get_color(str(BASE_DIR / "test.jpg"), quality=1)
    assert len(color) == 3


def test_quality_ten_max():
    color = modern_colorthief.get_color(str(BASE_DIR / "test.jpg"), quality=10)
    assert len(color) == 3


def test_quality_five_valid():
    color = modern_colorthief.get_color(str(BASE_DIR / "test.jpg"), quality=5)
    assert len(color) == 3


# -- Unsupported types --

@pytest.mark.parametrize("bad_input", [None, 42, 3.14, True, [], {}, set()])
def test_unsupported_type_color(bad_input):
    with pytest.raises(TypeError):
        modern_colorthief.get_color(bad_input)


@pytest.mark.parametrize("bad_input", [None, 42, 3.14, True, [], {}, set()])
def test_unsupported_type_palette(bad_input):
    with pytest.raises(TypeError):
        modern_colorthief.get_palette(bad_input)


def test_list_of_bytes():
    with pytest.raises(TypeError):
        modern_colorthief.get_color([b"\x89PNG"])


def test_dict_as_input():
    with pytest.raises(TypeError):
        modern_colorthief.get_palette({"path": "test.jpg"})
