"""Modern color palette extraction from images using GPU acceleration (Vulkan)."""

from typing import List, Tuple, Union

from . import modern_colorthief_gpu as _lib


def extract_palette(
    image: Union[str, bytes], size: int = 5, quality: int = 10
) -> List[Tuple[int, int, int]]:
    """Extract color palette from an image using GPU acceleration.

    Args:
        image: Path to image file or raw RGBA bytes.
        size: Number of colors to extract (default: 5).
        quality: Lower = higher quality, higher = faster (default: 10).

    Returns:
        List of (R, G, B) tuples.
    """
    if isinstance(image, str):
        return _lib.extract_palette_py(image, size, quality)
    elif isinstance(image, (bytes, bytearray)):
        # Assume 100x100 if no dimensions — caller should pass a file path instead
        # For bytes input we need width/height; delegate to the buffer function
        # Since we can't infer dimensions from raw bytes, require paths for file input
        # and treat bytes as needing explicit dimensions via extract_palette_from_buffer
        raise ValueError(
            "Raw bytes require width/height. Use a file path or "
            "call extract_palette_from_buffer directly."
        )
    else:
        return _lib.extract_palette_py(str(image), size, quality)


def extract_palette_from_buffer(
    pixels: bytes, width: int, height: int, size: int = 5, quality: int = 10
) -> List[Tuple[int, int, int]]:
    """Extract color palette from raw RGBA pixel buffer using GPU acceleration.

    Args:
        pixels: Raw RGBA pixel data (4 bytes per pixel).
        width: Image width in pixels.
        height: Image height in pixels.
        size: Number of colors to extract (default: 5).
        quality: Lower = higher quality, higher = faster (default: 10).

    Returns:
        List of (R, G, B) tuples.
    """
    return _lib.extract_palette_from_buffer_py(pixels, width, height, size, quality)


def extract_dominant_color(
    image: Union[str, bytes], quality: int = 10
) -> Tuple[int, int, int]:
    """Extract the dominant color from an image using GPU acceleration.

    Args:
        image: Path to image file.
        quality: Lower = higher quality, higher = faster (default: 10).

    Returns:
        (R, G, B) tuple of the dominant color.
    """
    if isinstance(image, str):
        return _lib.extract_dominant_color_py(image, quality)
    else:
        return _lib.extract_dominant_color_py(str(image), quality)


def extract_dominant_color_from_buffer(
    pixels: bytes, width: int, height: int, quality: int = 10
) -> Tuple[int, int, int]:
    """Extract the dominant color from raw RGBA pixel buffer using GPU acceleration.

    Args:
        pixels: Raw RGBA pixel data (4 bytes per pixel).
        width: Image width in pixels.
        height: Image height in pixels.
        quality: Lower = higher quality, higher = faster (default: 10).

    Returns:
        (R, G, B) tuple of the dominant color.
    """
    return _lib.extract_dominant_color_from_buffer_py(pixels, width, height, quality)


def list_gpus() -> List[dict]:
    """List available GPUs with Vulkan compute support.

    Returns:
        List of dicts with keys: index, name, device_type, vendor_name.
    """
    return _lib.list_gpus_py()
