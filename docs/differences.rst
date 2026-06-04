============
 Differences
============

.. rubric:: Comparison with Other Libraries

Modern Colorthief differs from similar libraries in several key ways.

.. rubric:: vs fast-colorthief

.. list-table::
   :widths: 50 50
   :header-rows: 1

   * - Feature
     - Detail
   * - Architecture Support
     - Broader support via PyO3 (vs pybind11)
   * - NumPy Dependency
     - No hard dependency on NumPy
   * - Codebase
     - Simpler, more maintainable Rust code vs C++ codebase
   * - Build Tooling
     - Automated via Maturin and GitHub Actions
   * - Package Size
     - 500-700 KB vs 52-60 KB for fast-colorthief

.. rubric:: vs color-thief-py

.. list-table::
   :widths: 50 50
   :header-rows: 1

   * - Feature
     - Detail
   * - Execution Speed
     - Nearly 100x faster
   * - Pillow Dependency
     - No hard dependency on Pillow
   * - Python Compatibility
     - Fully compatible with modern Python versions

.. rubric:: Achieving Parity with colorthief (Python)

If you need output that matches the original ``colorthief`` library:

#. Load the image with Pillow.
#. Save the loaded image to a ``BytesIO`` object.
#. Pass the ``BytesIO`` object to ``modern_colorthief``.

.. code-block:: python
   :linenos:

   import io
   from PIL import Image
   import modern_colorthief

   path = "photo.jpg"
   img = Image.open(path, mode="r")

   image_bytes = io.BytesIO()
   img.save(image_bytes, format="PNG")
   image_bytes.seek(0)

   dominant_color = modern_colorthief.get_color(image_bytes)
   dominant_palette = modern_colorthief.get_palette(image_bytes)

.. rubric:: Achieving Parity with fast-colorthief (C++)

The same approach works for matching ``fast-colorthief`` output:

.. code-block:: python
   :linenos:

   import io
   from PIL import Image
   import modern_colorthief

   path = "photo.jpg"
   img = Image.open(path, mode="r")

   image_bytes = io.BytesIO()
   img.save(image_bytes, format="PNG")
   image_bytes.seek(0)

   dominant_color = modern_colorthief.get_color(image_bytes)
   dominant_palette = modern_colorthief.get_palette(image_bytes)

.. note::

   The parity technique works because Pillow normalizes pixel data across
   image formats, ensuring consistent input to the quantization algorithm.

.. seealso::

   :doc:`api_differences` -- Detailed output comparison with color-thief.

   :doc:`benchmarks` -- Speed comparison across implementations.
