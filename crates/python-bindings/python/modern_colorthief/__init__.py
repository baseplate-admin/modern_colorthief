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
    """Extract a palette of dominant colors from an image.

    Uses the Median Cut Color Quantization algorithm implemented in Rust
    for high performance.

    Args:
        image: File path, raw image bytes, or a BytesIO object.
        color_count: Number of colors to extract. Defaults to 10.
        quality: Downsample factor -- every Nth pixel is skipped.
            Higher is faster but less accurate. Defaults to 10.

    Returns:
        A deduplicated list of RGB tuples, each ``(R, G, B)`` with
        values in the range ``0..=255``.

    Raises:
        ValueError: If the image is invalid or the algorithm fails.
        TypeError: If the image type is unsupported.

    Example:
        >>> from modern_colorthief import get_palette
        >>> get_palette("photo.jpg", color_count=5)
        [(139, 69, 19), (220, 20, 60), (255, 250, 240), (34, 139, 34), (70, 130, 180)]
    """
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
    """Extract the single dominant color from an image.

    Internally extracts a small palette and returns the top color.

    Args:
        image: File path, raw image bytes, or a BytesIO object.
        quality: Downsample factor -- every Nth pixel is skipped.
            Higher is faster but less accurate. Defaults to 10.

    Returns:
        An RGB tuple ``(R, G, B)`` with values in the range ``0..=255``.

    Raises:
        ValueError: If the image is invalid or the algorithm fails.
        TypeError: If the image type is unsupported.

    Example:
        >>> from modern_colorthief import get_color
        >>> get_color("photo.jpg")
        (139, 69, 19)
    """
    match image:
        case str():
            return _get_color_given_location(image, quality)
        case bytes():
            return _get_color_given_bytes(image, quality)
        case io.BytesIO():
            return _get_color_given_bytes(image.getvalue(), quality)
        case _:
            raise TypeError("Unsupported image type")
