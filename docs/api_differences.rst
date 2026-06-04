================
 API Differences
================

.. rubric:: Overview

This page documents the output differences between ``modern_colorthief``
and the original ``color-thief`` (JavaScript/Python) library.

.. rubric:: Comparison with color-thief

The difference between ``color-thief`` and ``modern_colorthief`` is very
negligible. Both use the same Median Cut algorithm, so results are nearly
identical.

.. rubric:: Example Comparison

Using the following test code:

.. code-block:: python
   :linenos:
   :caption: Comparison test

   from modern_colorthief import get_color, get_palette
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

The output is:

.. code-block:: python
   :linenos:
   :caption: Output comparison

   # modern_colorthief dominant color
   (201, 160, 118)

   # colorthief dominant color
   (202, 160, 118)

   # modern_colorthief palette
   [(30, 169, 166), (179, 51, 55), (219, 176, 127), (248, 233, 225),
    (160, 98, 87), (63, 47, 42), (131, 163, 107), (179, 119, 52),
    (237, 220, 155)]

   # colorthief palette
   [(31, 167, 164), (179, 51, 55), (219, 176, 127), (248, 233, 225),
    (160, 98, 87), (62, 44, 38), (131, 162, 106), (178, 118, 51),
    (242, 220, 157)]

.. topic:: Analysis

   As you can see, the differences are minimal -- typically within 1-3
   units per channel. This is expected because both implementations use
   the same algorithm, but minor floating-point rounding differences and
   image decoding variations cause slight deviations.

.. note::

   For most practical purposes (UI theming, color analysis, design tools),
   these differences are imperceptible.

.. seealso::

   :doc:`differences` -- Broader comparison with other libraries.
