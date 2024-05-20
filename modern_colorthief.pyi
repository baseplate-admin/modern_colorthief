__version__: str

def get_color(
    image: str,
    quality: int | None = 10,
) -> tuple[int, int, int]: ...
def get_palette(
    image: str,
    color_count: int | None = 10,
    quality: int | None = 10,
) -> list[tuple[int, int, int]]: ...
