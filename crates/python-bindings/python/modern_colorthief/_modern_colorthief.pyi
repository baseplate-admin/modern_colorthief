__version__: str
"""The version of the modern_colorthief package."""

def _get_palette_given_pixels(
    pixels: bytes, width: int, height: int, color_count: int | None = None, quality: int | None = None
) -> list[tuple[int, int, int]]:
    """Extract a color palette from raw RGBA pixel bytes."""
    ...

def _get_color_given_pixels(
    pixels: bytes, width: int, height: int, quality: int | None = None
) -> tuple[int, int, int]:
    """Extract the dominant color from raw RGBA pixel bytes."""
    ...
