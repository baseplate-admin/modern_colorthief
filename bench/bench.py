import os
import timeit
from pathlib import Path

from tabulate import tabulate

from colorthief import ColorThief
from fast_colorthief import get_dominant_color, get_palette as f_get_palette
from modern_colorthief import get_color, get_palette

BASE_DIR = Path(__file__).resolve().parent
path = os.path.join(BASE_DIR, "test.jpg")


def benchmark(func, *args, **kwargs):
    """Run a single benchmark and return elapsed seconds."""
    start = timeit.default_timer()
    func(*args, **kwargs)
    return timeit.default_timer() - start


# --- Extracting Color ---
py_color_time = benchmark(ColorThief(path).get_color)
cpp_color_time = benchmark(get_dominant_color, path, 10)
rust_color_time = benchmark(get_color, path)

# --- Extracting Palette ---
py_palette_time = benchmark(ColorThief(path).get_palette)
cpp_palette_time = benchmark(f_get_palette, path)
rust_palette_time = benchmark(get_palette, path)

# --- Table ---
headers = ["Task", "Python (colorthief)", "CPP (fast_colorthief)", "Rust (modern_colorthief)"]
rows = [
    ["Extract Color", f"{py_color_time:.6f}s", f"{cpp_color_time:.6f}s", f"{rust_color_time:.6f}s"],
    ["Extract Palette", f"{py_palette_time:.6f}s", f"{cpp_palette_time:.6f}s", f"{rust_palette_time:.6f}s"],
]

print(tabulate(rows, headers=headers, tablefmt="grid"))
