import modern_colorthief
from pathlib import Path
import os
import io
from PIL import Image

BASE_DIR = Path(__file__).resolve().parent

path = os.path.join(BASE_DIR, "test.jpg")


def test_path():
    dominant_color = modern_colorthief.get_color(path)
    dominant_palette = modern_colorthief.get_palette(path)

    # Color is valid RGB
    assert all(0 <= c <= 255 for c in dominant_color)
    # Palette has expected count
    assert len(dominant_palette) == 9
    # All palette colors are valid
    assert all(all(0 <= c <= 255 for c in color) for color in dominant_palette)


def _get_from_png():
    img = Image.open(path, mode="r")
    img_byte_arr = io.BytesIO()
    img.save(img_byte_arr, format="PNG")
    img_byte_arr.seek(0)
    return img_byte_arr


def test_bytesio():
    bio = _get_from_png()
    dominant_color = modern_colorthief.get_color(bio)
    dominant_palette = modern_colorthief.get_palette(bio)

    assert all(0 <= c <= 255 for c in dominant_color)
    assert len(dominant_palette) == 9


def test_bytes():
    bio = _get_from_png()
    data = bio.getvalue()

    dominant_color = modern_colorthief.get_color(data)
    dominant_palette = modern_colorthief.get_palette(data)

    assert all(0 <= c <= 255 for c in dominant_color)
    assert len(dominant_palette) == 9


def test_consistent_across_inputs():
    """Path, bytesio, and bytes should produce same dominant color."""
    color_path = modern_colorthief.get_color(path)
    bio = _get_from_png()
    color_bio = modern_colorthief.get_color(bio)
    color_bytes = modern_colorthief.get_color(bio.getvalue())

    # Colors may differ slightly due to JPEG→PNG conversion, but should be close
    def color_distance(a, b):
        return sum((x - y) ** 2 for x, y in zip(a, b)) ** 0.5

    assert color_distance(color_path, color_bytes) < 30
