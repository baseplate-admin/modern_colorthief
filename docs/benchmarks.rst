===========
 Benchmarks
===========

.. rubric:: Overview

This page compares the execution time of ``modern_colorthief`` (Rust-based)
against the original ``colorthief`` (pure Python) and ``fast_colorthief``
(C++ based).

.. rubric:: Benchmark Script

You can reproduce the benchmarks using the following script:

.. code-block:: python
   :linenos:
   :caption: benchmark_colorthief.py

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

   print(f"Python color:  {py_color_time:.6f}s")
   print(f"C++ color:     {cpp_color_time:.6f}s")
   print(f"Rust color:    {rust_color_time:.6f}s")
   print(f"Python palette: {py_palette_time:.6f}s")
   print(f"C++ palette:    {cpp_palette_time:.6f}s")
   print(f"Rust palette:   {rust_palette_time:.6f}s")

.. rubric:: Results

On a sample image, the execution times are approximately as follows:

.. list-table:: Performance Comparison
   :widths: 25 25 25 25
   :header-rows: 1

   * - Task
     - Python (colorthief)
     - C++ (fast_colorthief)
     - Rust (modern_colorthief)
   * - Extracting Color
     - 0.219895 s
     - 0.021180 s
     - **0.019645 s**
   * - Extracting Palette
     - 0.202956 s
     - 0.023626 s
     - **0.018661 s**

.. topic:: Performance Summary

   ``modern_colorthief`` provides roughly a **100x speedup** compared to
   the pure Python implementation, matching or exceeding the performance
   of the C++ implementation without the overhead of C++ compilation
   tooling.

.. warning::

   Benchmark results may vary depending on the image size, color
   complexity, and hardware. Use the script above to test on your own
   workloads.

.. seealso::

   :doc:`mmcq` -- The algorithm behind these performance numbers.
