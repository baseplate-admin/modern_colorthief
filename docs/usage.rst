=====
 Usage
=====

.. rubric:: Overview

Modern Colorthief exposes two primary functions:

- :func:`modern_colorthief.get_color` -- extract the single dominant color
- :func:`modern_colorthief.get_palette` -- extract a palette of dominant colors

Both functions accept a file path (``str``), raw bytes (``bytes``), or a
``io.BytesIO`` object.

.. versionadded:: 0.4.0

   Support for ``bytes`` and ``io.BytesIO`` input.

.. rubric:: Using a File Path

The simplest way to use the library is to pass a path string:

.. code-block:: python

   import modern_colorthief

   # Path to any supported image file
   path = "photo.jpg"

   # Get the dominant color
   color = modern_colorthief.get_color(path)
   print(color)  # e.g. (139, 69, 19)

   # Get a palette of 10 colors
   palette = modern_colorthief.get_palette(path)
   print(palette)  # e.g. [(139, 69, 19), (220, 20, 60), ...]

.. rubric:: Using BytesIO

When working with image data in memory, pass a ``BytesIO`` object:

.. code-block:: python

   import io
   import modern_colorthief

   # Load image bytes from a source (network, database, etc.)
   image_bytes = io.BytesIO(b"\x89PNG\r\n...")

   dominant_color = modern_colorthief.get_color(image_bytes)
   dominant_palette = modern_colorthief.get_palette(image_bytes)

.. rubric:: Using a Pillow Image

To use a Pillow ``Image`` object, save it to a ``BytesIO`` buffer first:

.. code-block:: python

   import io
   from PIL import Image
   import modern_colorthief

   img = Image.open("photo.jpg")
   img = img.convert("RGB")

   image_bytes = io.BytesIO()
   img.save(image_bytes, format="PNG")
   image_bytes.seek(0)

   dominant_color = modern_colorthief.get_color(image_bytes)
   dominant_palette = modern_colorthief.get_palette(image_bytes)

.. rubric:: Using a NumPy Array

You can also work with NumPy arrays by converting through Pillow:

.. code-block:: python

   import io
   import numpy as np
   from PIL import Image
   import modern_colorthief

   # Your numpy array of shape (H, W, 3) with RGB values
   arr = np.random.randint(0, 256, (480, 640, 3), dtype=np.uint8)

   img = Image.fromarray(arr)

   image_bytes = io.BytesIO()
   img.save(image_bytes, format="PNG")
   image_bytes.seek(0)

   dominant_color = modern_colorthief.get_color(image_bytes)
   dominant_palette = modern_colorthief.get_palette(image_bytes)

.. note::

   ``modern_colorthief`` does not require Pillow or NumPy as a hard
   dependency. These are only needed when you want to pre-process images
   before passing them to the library.

.. seealso::

   :doc:`differences` -- If the colors returned differ from other
   libraries, see the parity notes.

   :doc:`api` -- Full API reference with parameter details.
