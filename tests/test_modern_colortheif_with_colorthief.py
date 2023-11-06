from colorthief import ColorThief
import pillow_avif  # noqa
from pathlib import Path

from modern_colorthief import ColorThief

BASE_DIR = Path(__file__).resolve().parent


def test_colorthief_with_modern_colorthief():
    image_path = Path(BASE_DIR, "ichigo.avif")

    color_theif_original = ColorThief(image_path).get_color()
    color_theif_modern = ColorThief(image_path).get_color()

    assert color_theif_original == color_theif_modern
