import timeit
from modern_colorthief import get_color, get_palette
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent
path = BASE_DIR / "test.jpg"

color_time = timeit.timeit(lambda: get_color(path), number=100)
print(f"get_color (100 runs): {color_time:.4f}s")

palette_time = timeit.timeit(lambda: get_palette(path), number=100)
print(f"get_palette (100 runs): {palette_time:.4f}s")
