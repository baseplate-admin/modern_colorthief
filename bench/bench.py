import os
import timeit
from pathlib import Path

from colorthief import ColorThief
from fast_colorthief import get_dominant_color, get_palette as f_get_palette
from modern_colorthief import get_color, get_palette

BASE_DIR = Path(__file__).resolve().parent
path = os.path.join(BASE_DIR, "test.jpg")

# --- Extracting Color ---
start_time = timeit.default_timer()
ColorThief(path).get_color()
py_color_time = timeit.default_timer() - start_time

start_time = timeit.default_timer()
get_dominant_color(path, 10)
cpp_color_time = timeit.default_timer() - start_time

start_time = timeit.default_timer()
get_color(path)
rust_color_time = timeit.default_timer() - start_time


# --- Extracting Palette ---
start_time = timeit.default_timer()
ColorThief(path).get_palette()
py_palette_time = timeit.default_timer() - start_time

start_time = timeit.default_timer()
f_get_palette(path)
cpp_palette_time = timeit.default_timer() - start_time

start_time = timeit.default_timer()
get_palette(path)
rust_palette_time = timeit.default_timer() - start_time

print("| Task | Python (`colorthief`) | CPP (`fast_colorthief`) | Rust (`modern_colorthief`) |")
print("|---|---|---|---|")
print(f"| Extracting Color | {py_color_time:.6f}s | {cpp_color_time:.6f}s | {rust_color_time:.6f}s |")
print(f"| Extracting Palette | {py_palette_time:.6f}s | {cpp_palette_time:.6f}s | {rust_palette_time:.6f}s |")