import modern_colorthief
from pathlib import Path
import os
import io
from PIL import Image

BASE_DIR = Path(__file__).resolve().parent

path = os.path.join(BASE_DIR, "test.jpg")


def test_path():
    dominant_color = modern_colorthief.get_color(path)

    assert dominant_color == (201, 160, 118)


def test_bytesio():
    img = Image.open(path, mode="r")

    img_byte_arr = io.BytesIO()
    img.save(img_byte_arr, format="PNG")

    dominant_color = modern_colorthief.get_color(img_byte_arr)
    assert dominant_color == (201, 160, 118)


def test_bytes():
    img = Image.open(path, mode="r")

    img_byte_arr = io.BytesIO()
    img.save(img_byte_arr, format="PNG")

    dominant_color = modern_colorthief.get_color(img_byte_arr.getvalue())
    assert dominant_color == (201, 160, 118)
