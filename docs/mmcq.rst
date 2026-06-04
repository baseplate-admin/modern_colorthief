======================================
 Median Cut Color Quantization (MMCQ)
======================================

.. rubric:: Overview

Median Cut Color Quantization is an algorithm used to reduce the number
of colors in an image while preserving visual similarity. It is widely
used in image processing and color palette extraction.

Modern Colorthief implements a highly optimized version of this
algorithm in Rust.

.. rubric:: How It Works

The algorithm works by recursively dividing the RGB color space into
smaller "boxes" and selecting representative colors.

.. rubric:: Algorithm Steps

#. **Collect Pixels** -- Gather all pixels from the image.

#. **Initial Box** -- Place all pixels into a single 3D RGB box.

#. **Find Longest Channel** -- Identify the color channel (R, G, or B)
   with the largest range within the current box.

#. **Sort** -- Sort pixels along that channel.

#. **Split** -- Divide the box at the median point into two sub-boxes.

#. **Recurse** -- Repeat steps 3-5 until the desired number of color
   boxes is reached.

#. **Compute Averages** -- Calculate the average color of each box to
   form the final palette.

.. topic:: Visual Explanation

   Imagine a 3D cube where each axis represents a color channel (Red,
   Green, Blue). Each pixel is a point in this cube. The algorithm
   repeatedly cuts the most "spread out" dimension in half, creating
   smaller boxes. The center of each final box becomes a palette color.

.. rubric:: Advantages

- **Simple and Efficient** -- O(n log k) complexity where n is the
  number of pixels and k is the target palette size.
- **Visually Good Palettes** -- Produces colors that well represent the
  image's dominant hues.
- **Widely Used** -- Industry-standard algorithm for color quantization.

.. rubric:: Parameters

The ``quality`` parameter controls the sampling rate. A higher quality
value means fewer pixels are analyzed, resulting in faster execution but
slightly less accurate results.

.. list-table:: Quality Guide
   :widths: 25 75
   :header-rows: 1

   * - Quality
     - Effect
   * - 1
     - Every pixel analyzed (slowest, most accurate)
   * - 10
     - Every 10th pixel (default, balanced)
   * - 100
     - Every 100th pixel (fastest, least accurate)

.. rubric:: Reference

:download:`Detailed PDF explanation <_static/pdfs/mediancut.pdf>`

.. seealso::

   :doc:`benchmarks` -- Performance benchmarks of the Rust implementation.

   :doc:`api` -- API reference for get_color and get_palette.
