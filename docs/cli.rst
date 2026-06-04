==========================
 Command Line Interface
==========================

.. rubric:: Overview

``modern_colorthief`` ships with a built-in command-line interface
invoked via the ``modern-colorthief`` command.

.. rubric:: Usage

.. code-block:: bash

   modern-colorthief [OPTIONS] FILE

.. rubric:: Arguments

.. describe:: FILE

   Path to the image file to analyze.

.. rubric:: Options

.. describe:: --palette

   Extract a palette of dominant colors instead of a single color.

.. describe:: --quality

   Quality setting (default: ``10``). Higher values are faster but less
   accurate. Valid range is ``1`` to ``100``.

.. describe:: --count

   Number of colors in the palette (default: ``5``). Only used with
   ``--palette``.

.. rubric:: Examples

**Get the dominant color:**

.. code-block:: bash

   $ modern-colorthief path/to/image.png
   # Output: #f0e68c

**Get a color palette:**

.. code-block:: bash

   $ modern-colorthief path/to/image.png --palette
   # Output:
   # #f0e68c
   # #556b2f
   # ...

**Custom quality and count:**

.. code-block:: bash

   $ modern-colorthief photo.jpg --palette --quality 5 --count 8
   # Output: 8 colors with higher accuracy

.. tip::

   Use a lower ``--quality`` value (e.g., ``1``) for the most accurate
   results when processing images where color precision matters.

.. seealso::

   :doc:`api` -- The CLI wraps the same functions as the Python API.
