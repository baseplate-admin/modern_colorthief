Median Cut Color Quantization (MMCQ)
====================================

Median Cut Color Quantization is an algorithm used to reduce the number of colors
in an image while preserving visual similarity. It is widely used in image
processing and color palette extraction.

Overview
--------

The algorithm works by recursively dividing the RGB color space into smaller
"boxes" and selecting representative colors.

Steps
-----

1. Collect all pixels from the image.
2. Place them into a single 3D RGB box.
3. Find the color channel (R, G, or B) with the largest range.
4. Sort pixels along that channel.
5. Split the box at the median point.
6. Repeat until the desired number of colors is reached.
7. Compute the average color of each box to form the palette.

Advantages
----------

- Simple and efficient
- Produces visually good palettes
- Widely used 

Reference
---------

:download:`Detailed PDF explanation <_static/pdfs/mediancut.pdf>`