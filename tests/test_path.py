import modern_colorthief
from pathlib import Path
import os
import io
from PIL import Image

BASE_DIR = Path(__file__).resolve().parent

path = os.path.join(BASE_DIR, "test.jpg")

TARGET_PALTTE = [
    (31, 169, 167),
    (179, 51, 55),
    (219, 176, 127),
    (248, 233, 225),
    (160, 98, 87),
    (63, 47, 43),
    (132, 162, 107),
    (179, 119, 52),
    (237, 220, 155),
]
TARGET_COLOR = (201, 160, 118)


def test_path():
    dominant_color = modern_colorthief.get_color(path)
    dominant_palette = modern_colorthief.get_palette(path)

    assert dominant_color == TARGET_COLOR

    # If we do not use PILLOW the image output is slightly different
    assert dominant_palette == [
        (30, 169, 166),
        (179, 51, 55),
        (219, 176, 127),
        (248, 233, 225),
        (160, 98, 87),
        (63, 47, 42),
        (131, 163, 107),
        (179, 119, 52),
        (237, 220, 155),
    ]


def test_bytesio():
    img = Image.open(path, mode="r")

    img_byte_arr = io.BytesIO()
    img.save(img_byte_arr, format="PNG")

    dominant_color = modern_colorthief.get_color(img_byte_arr)
    dominant_palette = modern_colorthief.get_palette(img_byte_arr)

    assert dominant_color == TARGET_COLOR
    assert dominant_palette == TARGET_PALTTE


def test_bytes():
    img = Image.open(path, mode="r")

    img_byte_arr = io.BytesIO()
    img.save(img_byte_arr, format="PNG")

    dominant_color = modern_colorthief.get_color(img_byte_arr.getvalue())
    dominant_palette = modern_colorthief.get_palette(img_byte_arr.getvalue())

    assert dominant_color == TARGET_COLOR
    assert dominant_palette == TARGET_PALTTE
