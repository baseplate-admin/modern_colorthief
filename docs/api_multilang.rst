======================
 Multi-Language API
======================

.. rubric:: Overview

Modern Colorthief provides native bindings for multiple programming languages,
all backed by the same Rust Median Cut Color Quantization core.  Every language
exposes two primary operations:

* **``get_palette()``** -- Extract a color palette (list of RGB triples)
* **``get_color()``**   -- Extract the single dominant color (one RGB triple)

.. rubric:: Parameter Reference

.. list-table:: Common Parameters
   :widths: 20 10 70
   :header-rows: 1

   * - Parameter
     - Type
     - Description
   * - ``pixels``
     - byte array
     - Raw RGBA pixel data (4 bytes per pixel, row-major order)
   * - ``width``
     - integer
     - Image width in pixels
   * - ``height``
     - integer
     - Image height in pixels
   * - ``color_count``
     - integer (optional)
     - Maximum number of colors to extract (default: 10, range: 1..255)
   * - ``quality``
     - integer (optional)
     - Sampling quality: 1 = every pixel, higher = faster but less accurate (default: 10)

.. note::

   The ``quality`` parameter controls subsampling.  A value of ``1`` examines
   every pixel; ``10`` examines every 10th pixel.  Higher values are faster
   but may miss rare colors.  Values outside ``1..10`` are clamped.

.. rubric:: Return Types

.. list-table:: Return Values
   :widths: 25 75
   :header-rows: 1

   * - Function
     - Return
   * - ``get_palette()``
     - List/array of RGB triples ``[[R, G, B], ...]``
   * - ``get_color()``
     - Single RGB triple ``[R, G, B]``

All color channel values are integers in ``0..=255``.

.. rubric:: Installation by Language

.. tabs::

   .. code-tab:: py Python

      .. code-block:: bash

         pip install modern_colorthief

   .. code-tab:: rb Ruby

      .. code-block:: bash

         gem install modern_colorthief

   .. code-tab:: js Node.js

      .. code-block:: bash

         npm install modern-colorthief

   .. code-tab:: bash Java

      .. code-block:: bash

         # Add to your Maven/Gradle dependencies
         # JAR available from the release assets

   .. code-tab:: php PHP

      .. code-block:: bash

         # Load as a PHP extension in php.ini
         extension=modern_colorthief.so

.. rubric:: Getting a Color Palette

Extract up to ``N`` dominant colors from raw pixel data.

.. tabs::

   .. code-tab:: py Python

      .. code-block:: python
         :linenos:

         from modern_colorthief import get_palette

         # From a file path
         palette = get_palette("photo.jpg", color_count=5)
         print(palette)
         # [(139, 69, 19), (220, 20, 60), (255, 250, 240), ...]

         # From raw bytes
         palette = get_palette(image_bytes, color_count=10)

   .. code-tab:: rb Ruby

      .. code-block:: ruby
         :linenos:

         require "colorthief_ruby"

         # From raw RGBA pixel data
         pixels = "\xFF\x00\x00\xFF" * 100  # 10x10 solid red
         palette = Colorthief.get_palette(pixels, 10, 10, 5, 1)
         puts palette.inspect
         # [[255, 0, 0], ...]

   .. code-tab:: js Node.js

      .. code-block:: javascript
         :linenos:

         const { getPalette } = require("modern-colorthief");

         // From raw RGBA pixel Uint8Array
         const pixels = new Uint8Array([255, 0, 0, 255]); // 1 red pixel
         const palette = getPalette(pixels, 1, 1, 5, 1);
         console.log(palette);
         // [[255, 0, 0], ...]

   .. code-tab:: java Java

      .. code-block:: java
         :linenos:

         import io.baseplate_admin.modern_colorthief.Colorthief;

         public class Example {
             public static void main(String[] args) {
                 byte[] pixels = new byte[]{(byte)255, 0, 0, -1}; // 1 red pixel (RGBA, Java byte is signed)
                 byte[][] palette = Colorthief.getPalette(pixels, 1, 1, 5, 1);
                 for (byte[] color : palette) {
                     System.out.println(Arrays.toString(color));
                 }
             }
         }

   .. code-tab:: php PHP

      .. code-block:: php
         :linenos:

         <?php
         // Raw RGBA pixel data as integer array
         $pixels = [255, 0, 0, 255]; // 1 red pixel
         $palette = get_palette($pixels, 1, 1, 5, 1);
         print_r($palette);
         // Array([[255, 0, 0], ...])

.. rubric:: Getting the Dominant Color

Extract the single most prominent color from raw pixel data.

.. tabs::

   .. code-tab:: py Python

      .. code-block:: python
         :linenos:

         from modern_colorthief import get_color

         # From a file path
         color = get_color("photo.jpg")
         print(color)
         # (139, 69, 19)

         # From raw bytes
         color = get_color(image_bytes)

   .. code-tab:: rb Ruby

      .. code-block:: ruby
         :linenos:

         require "colorthief_ruby"

         pixels = "\xFF\x00\x00\xFF" * 100  # 10x10 solid red
         color = Colorthief.get_color(pixels, 10, 10, 1)
         puts color.inspect
         # [255, 0, 0]

   .. code-tab:: js Node.js

      .. code-block:: javascript
         :linenos:

         const { getColor } = require("modern-colorthief");

         const pixels = new Uint8Array([255, 0, 0, 255]); // 1 red pixel
         const color = getColor(pixels, 1, 1, 1);
         console.log(color);
         // [255, 0, 0]

   .. code-tab:: java Java

      .. code-block:: java
         :linenos:

         import io.baseplate_admin.modern_colorthief.Colorthief;

         public class Example {
             public static void main(String[] args) {
                 byte[] pixels = new byte[]{(byte)255, 0, 0, -1};
                 byte[] color = Colorthief.getColor(pixels, 1, 1, 1);
                 System.out.println(Arrays.toString(color));
                 // [255, 0, 0]
             }
         }

   .. code-tab:: php PHP

      .. code-block:: php
         :linenos:

         <?php
         $pixels = [255, 0, 0, 255]; // 1 red pixel
         $color = get_color($pixels, 1, 1, 1);
         print_r($color);
         // Array([255, 0, 0])

.. rubric:: GPU-Accelerated API

The GPU variants use Vulkan compute shaders for palette extraction on
supported hardware.  The API is identical to the CPU version but operates
on raw pixel buffers.

.. note::

   GPU bindings require a Vulkan-capable device.  If no GPU is available,
   the call falls back to the CPU implementation.

.. tabs::

   .. code-tab:: py Python (GPU)

      .. code-block:: python
         :linenos:

         from modern_colorthief_gpu import get_palette_gpu, get_color_gpu

         palette = get_palette_gpu(pixels, width, height, color_count=5)
         color   = get_color_gpu(pixels, width, height)

   .. code-tab:: rb Ruby (GPU)

      .. code-block:: ruby
         :linenos:

         require "colorthief_gpu"

         palette = ColorthiefGpu.get_palette(pixels, 10, 10, 5, 1)
         color   = ColorthiefGpu.get_color(pixels, 10, 10, 1)

   .. code-tab:: js Node.js (GPU)

      .. code-block:: javascript
         :linenos:

         const { getPaletteGpu, getColorGpu } = require("modern-colorthief-gpu");

         const palette = getPaletteGpu(pixels, width, height, 5, 1);
         const color   = getColorGpu(pixels, width, height, 1);

   .. code-tab:: java Java (GPU)

      .. code-block:: java
         :linenos:

         import modern.colorthief.ColorthiefGpu;

         byte[][] palette = ColorthiefGpu.getPalette(pixels, w, h, 5, 1);
         byte[] color    = ColorthiefGpu.getColor(pixels, w, h, 1);

   .. code-tab:: php PHP (GPU)

      .. code-block:: php
         :linenos:

         <?php
         // Same function names, GPU backend
         $palette = get_palette($pixels, $w, $h, 5, 1);
         $color   = get_color($pixels, $w, $h, 1);

.. rubric:: WebAssembly (Browser JavaScript)

For browser-based usage, Modern Colorthief compiles to WebAssembly via
``wasm-bindgen``.  Install from npm:

.. code-block:: bash

   npm install modern-colorthief-wasm

.. code-block:: javascript
   :linenos:

   import init, { get_palette, get_color } from 'modern-colorthief-wasm';

   await init();

   // pixels: Uint8Array of RGBA data
   const palette = get_palette(pixels, width, height, color_count, quality);
   const color   = get_color(pixels, width, height, quality);

.. seealso::

   :doc:`webassembly` -- Detailed WebAssembly setup and examples.

   :doc:`api` -- Python-specific autodoc API reference.

.. rubric:: Source Code References

The binding implementations are in these crate directories:

* **Python (CPU):**  ``crates/python-bindings/``
* **Python (GPU):**  ``crates/python-bindings-gpu/``
* **Ruby (CPU):**    ``crates/ruby-bindings/ext/``
* **Ruby (GPU):**    ``crates/ruby-bindings-gpu/ext/``
* **Node.js (CPU):** ``crates/node-bindings/src/``
* **Node.js (GPU):** ``crates/node-bindings-gpu/src/``
* **Java (CPU):**    ``crates/jvm-bindings/``
* **Java (GPU):**    ``crates/jvm-bindings-gpu/``
* **PHP (CPU):**     ``crates/php-bindings/src/``
* **PHP (GPU):**     ``crates/php-bindings-gpu/src/``
* **WASM (CPU):**    ``crates/wasm-bindings/``
* **WASM (GPU):**    ``crates/wasm-bindings-webgpu/``
