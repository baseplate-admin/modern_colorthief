=============
 API Reference
=============

.. rubric:: Package Overview

.. module:: modern_colorthief

Modern Colorthief provides two main functions for color extraction,
both backed by a Rust implementation of the Median Cut Color
Quantization algorithm.

.. note::

   Looking for bindings in another language?  See the
   :doc:`api_multilang` page for Ruby, Node.js, Java, PHP, and WebAssembly examples.

.. rubric:: Functions

.. autofunction:: modern_colorthief.get_palette

.. autofunction:: modern_colorthief.get_color

.. rubric:: Version

.. data:: modern_colorthief.__version__

   The current version string of the package.

.. rubric:: Accepted Input Types

Both :func:`get_color` and :func:`get_palette` accept the following
input types:

.. list-table:: Supported Inputs
   :widths: 25 75
   :header-rows: 1

   * - Type
     - Description
   * - ``str``
     - A file path pointing to a supported image format (JPEG, PNG, WebP, GIF, BMP, TIFF)
   * - ``bytes``
     - Raw image data in memory
   * - ``io.BytesIO``
     - A binary I/O stream containing image data

.. rubric:: Return Types

.. list-table:: Return Values
   :widths: 25 75
   :header-rows: 1

   * - Function
     - Return Type
   * - ``get_color()``
     - ``tuple[int, int, int]`` -- an RGB triple
   * - ``get_palette()``
     - ``list[tuple[int, int, int]]`` -- a list of RGB triples

.. note::

   All color values are integers in the range ``0..=255`` representing
   the Red, Green, and Blue channels respectively.

.. seealso::

   :doc:`usage` -- Practical examples of using the API.

   :doc:`mmcq` -- Details on the underlying algorithm.
