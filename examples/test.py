from python.modern_colorthief.modern_colorthief import get_color, get_palette
from colorthief import ColorThief
from pathlib import Path
import os

BASE_DIR = Path(__file__).resolve().parent
path = os.path.join(BASE_DIR, "test.jpg")

print(path)
x = get_color(path)
y = ColorThief(path).get_color()

print(x)
print(y)

m = get_palette(path)
n = ColorThief(path).get_palette()

print(m)
print(n)
