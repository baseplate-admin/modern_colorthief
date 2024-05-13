def get_color(
    location: str,
    quality: int | None = 10,
) -> tuple[int, int, int]: ...
def get_palette(
    location: str,
    color_count: int | None = 10,
    quality: int | None = 10,
) -> list[tuple[int, int, int]]: ...
