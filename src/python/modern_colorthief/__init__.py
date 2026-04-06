import io

# Rust import
from ._modern_colorthief import *

__doc__ = _modern_colorthief.__doc__
__version__ = _modern_colorthief.__version__


def get_palette(
    image: str | bytes | io.BytesIO,
    color_count: int = 10,
    quality: int = 10,
) -> list[tuple[int, int, int]]:
    match image:
        case str():
            return _get_palette_given_location(image, color_count, quality)
        case bytes():
            return _get_palette_given_bytes(image, color_count, quality)
        case io.BytesIO():
            return _get_palette_given_bytes(image.getvalue(), color_count, quality)
        case _:
            raise TypeError("Unsupported image type")


def get_color(
    image: str | bytes | io.BytesIO,
    quality: int = 10,
) -> tuple[int, int, int]:
    match image:
        case str():
            return _get_color_given_location(image, quality)
        case bytes():
            return _get_color_given_bytes(image, quality)
        case io.BytesIO():
            return _get_color_given_bytes(image.getvalue(), quality)
        case _:
            raise TypeError("Unsupported image type")
