__version__: str
"""The version of the modern_colorthief package."""

def _get_palette_given_bytes(
    image: bytes, color_count: int | None = None, quality: int | None = None
) -> list[tuple[int, int, int]]:
    """
    Returns the palette given a bytes object.

    Args:
        image (bytes): The image data.
        color_count (Optional[int]): The number of colors to return. Defaults to None (which implies 10 in Rust logic, but type is Option).
        quality (Optional[int]): The quality of the palette. Defaults to None (which implies 10 in Rust logic).

    Returns:
        List[Tuple[int, int, int]]: A list of RGB tuples.
    """
    ...

def _get_palette_given_location(
    image: str, color_count: int | None = None, quality: int | None = None
) -> list[tuple[int, int, int]]:
    """
    Returns the palette given an image path.

    Args:
        image (str): The path to the image.
        color_count (Optional[int]): The number of colors to return. Defaults to None.
        quality (Optional[int]): The quality of the palette. Defaults to None.

    Returns:
        List[Tuple[int, int, int]]: A list of RGB tuples.
    """
    ...

def _get_color_given_bytes(
    image: bytes, quality: int | None = None
) -> tuple[int, int, int]:
    """
    Gets the dominant color given an image bytes object.

    Args:
        image (bytes): The image data.
        quality (Optional[int]): The quality of the color extraction. Defaults to None.

    Returns:
        Tuple[int, int, int]: The dominant RGB color.
    """
    ...

def _get_color_given_location(
    image: str, quality: int | None = None
) -> tuple[int, int, int]:
    """
    Gets the dominant color given an image path.

    Args:
        image (str): The path to the image.
        quality (Optional[int]): The quality of the color extraction. Defaults to None.

    Returns:
        Tuple[int, int, int]: The dominant RGB color.
    """
    ...
