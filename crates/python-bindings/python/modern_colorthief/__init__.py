import io
from PIL import Image

from ._modern_colorthief import *

__doc__ = _modern_colorthief.__doc__
__version__ = _modern_colorthief.__version__


def _to_rgba_pixels(image):
    """Convert any PIL Image to raw RGBA bytes, width, height."""
    img = Image.open(image) if isinstance(image, (str, bytes, io.BytesIO)) else image
    img = img.convert("RGBA")
    width, height = img.size
    return img.tobytes(), width, height


def get_palette(
    image: str | bytes | io.BytesIO | Image.Image,
    color_count: int = 10,
    quality: int = 10,
) -> list[tuple[int, int, int]]:
    """Extract a palette of dominant colors from an image.

    Uses the Median Cut Color Quantization algorithm implemented in Rust
    for high performance.

    Args:
        image: File path, raw image bytes, a BytesIO object, or a PIL Image.
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
    pixels, width, height = _to_rgba_pixels(image)
    return _get_palette_given_pixels(pixels, width, height, color_count, quality)


def get_color(
    image: str | bytes | io.BytesIO | Image.Image,
    quality: int = 10,
) -> tuple[int, int, int]:
    """Extract the single dominant color from an image.

    Internally extracts a small palette and returns the top color.

    Args:
        image: File path, raw image bytes, a BytesIO object, or a PIL Image.
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
    pixels, width, height = _to_rgba_pixels(image)
    return _get_color_given_pixels(pixels, width, height, quality)
