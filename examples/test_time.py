from python.modern_colorthief.modern_colorthief import get_color, get_palette
from fast_colorthief import get_dominant_color, get_palette as f_get_palette
import timeit
from colorthief import ColorThief
import os
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent
path = os.path.join(BASE_DIR, "test.jpg")


start_time = timeit.default_timer()
y = ColorThief(path).get_color()
elapsed = timeit.default_timer() - start_time
print(f"Python Took:\t\t{elapsed}")

start_time = timeit.default_timer()
z = get_dominant_color(path, 10)
elapsed = timeit.default_timer() - start_time
print(f"CPP Took:\t\t{elapsed}")

start_time = timeit.default_timer()
x = get_color(path)
elapsed = timeit.default_timer() - start_time
print(f"RUST Took:\t\t{elapsed}")

print("\n\n")

start_time = timeit.default_timer()
m = get_palette(path)
elapsed = timeit.default_timer() - start_time
print(f"RUST Took:\t\t{elapsed}")

start_time = timeit.default_timer()
n = ColorThief(path).get_palette()
elapsed = timeit.default_timer() - start_time
print(f"Python Took:\t\t{elapsed}")

start_time = timeit.default_timer()
o = f_get_palette(path)
elapsed = timeit.default_timer() - start_time
print(f"CPP Took:\t\t{elapsed}")
