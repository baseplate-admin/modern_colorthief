import io
import contextlib

with contextlib.suppress(ImportError):
    import numpy as np
    from PIL import Image as im

# Rust import
from .modern_colorthief import *


__doc__ = modern_colorthief.__doc__
__version__ = modern_colorthief.__version__

__all__ = [
    "__doc__",
    "__version__",
    "get_palette",
    "get_color",
]


def get_palette(
    image: str | bytes | io.BytesIO | np.array,
    color_count: int | None = 10,
    quality: int | None = 10,
) -> list[tuple[int, int, int]]:
    if isinstance(image, str):
        return _get_palette_given_location(image, color_count, quality)

    if isinstance(image, bytes):
        return _get_palette_given_bytes(image, color_count, quality)

    if isinstance(image, io.BytesIO):
        return _get_palette_given_bytes(image.getvalue(), color_count, quality)

    if isinstance(image, np.array):
        img = im.fromarray(image)
        image_bytes = io.BytesIO()
        img.save(image_bytes, format="PNG")

        return _get_palette_given_bytes(image_bytes.getvalue(), color_count, quality)


def get_color(
    image: str | bytes | io.BytesIO,
    quality: int | None = 10,
) -> tuple[int, int, int]:
    if isinstance(image, str):
        return _get_color_given_location(image, quality)

    if isinstance(image, bytes):
        return _get_color_given_bytes(image, quality)

    if isinstance(image, io.BytesIO):
        return _get_color_given_bytes(image.getvalue(), quality)

    if isinstance(image, np.array):
        img = im.fromarray(image)
        image_bytes = io.BytesIO()
        img.save(image_bytes, format="PNG")

        return _get_color_given_bytes(image_bytes.getvalue(), quality)
