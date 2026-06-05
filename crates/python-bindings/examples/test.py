from modern_colorthief import get_color, get_palette
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent
path = BASE_DIR / "test.jpg"

color = get_color(path)
print(f"Dominant color: {color}")

palette = get_palette(path, color_count=5)
print(f"Palette: {palette}")
