.. WebAssembly bindings

===========================================
WebAssembly
===========================================

Modern Colorthief runs in the browser via WebAssembly, delivering native-speed
color extraction with zero server-side dependencies.

.. seealso::

   :doc:`api_multilang` -- Multi-language API reference with examples for
   Python, Ruby, Node.js, Java, PHP, and WebAssembly.

.. rubric:: Installation

Via npm:

.. code-block:: bash

   npm install colorthief-wasm

Via yarn:

.. code-block:: bash

   yarn add colorthief-wasm

Via pnpm:

.. code-block:: bash

   pnpm add colorthief-wasm

.. rubric:: Quick Start

.. code-block:: javascript

   import { getPalette, getColor } from 'colorthief-wasm';

   // From a URL
   const palette = await getPalette('photo.jpg', 10, 10);
   console.log(palette); // [[139,69,19], [220,20,60], ...]

   const dominant = await getColor('photo.jpg', 10);
   console.log(dominant); // [139, 69, 19]

.. rubric:: API Reference

getPalette(source, count, quality)
===========================================

Extract a color palette from an image.

**Parameters**

- ``source`` - An image URL (string), a ``Uint8Array`` of image bytes, or raw ``Uint8ClampedArray`` pixels with width/height
- ``count`` - Number of colors to extract (default: 10, max: 255)
- ``quality`` - Sampling quality 1-10 (default: 10, lower = faster)

**Returns** ``Promise<number[][]>`` - Array of RGB color tuples

.. code-block:: javascript

   const palette = await getPalette('photo.jpg', 5, 10);
   // [[139,69,19], [220,20,60], [255,250,240], [34,139,34], [70,130,180]]


getColor(source, quality)
===========================================

Extract the dominant color from an image.

**Parameters**

- ``source`` - An image URL (string), a ``Uint8Array`` of image bytes, or raw ``Uint8ClampedArray`` pixels with width/height
- ``quality`` - Sampling quality 1-10 (default: 10, lower = faster)

**Returns** ``Promise<number[]>`` - RGB color tuple

.. code-block:: javascript

   const color = await getColor('photo.jpg', 10);
   // [139, 69, 19]


decodeImage(url)
===========================================

Decode an image to raw pixels for custom processing.

**Parameters**

- ``url`` - Image URL

**Returns** ``Promise<{pixels: Uint8ClampedArray, width: number, height: number}>``

.. code-block:: javascript

   const { pixels, width, height } = await decodeImage('photo.jpg');


.. rubric:: Input Types

The WASM API accepts three input formats:

**URL (string)**

.. code-block:: javascript

   const palette = await getPalette('https://example.com/photo.jpg', 10);

**Image bytes (Uint8Array)**

.. code-block:: javascript

   const response = await fetch('photo.jpg');
   const bytes = new Uint8Array(await response.arrayBuffer());
   const palette = await getPalette(bytes, 10);

**Raw pixels (Uint8ClampedArray)**

.. code-block:: javascript

   // Use decodeImage for raw pixel access
   const { pixels, width, height } = await decodeImage('photo.jpg');
   // Now pass pixels to colorthief-core directly

.. rubric:: Framework Examples

React
===========================================

.. code-block:: jsx

   import { useState, useEffect } from 'react';
   import { getColor } from 'colorthief-wasm';

   function DominantColor({ src }) {
       const [color, setColor] = useState(null);

       useEffect(() => {
           getColor(src).then(setColor);
       }, [src]);

       if (!color) return <div>Loading...</div>;

       return (
           <div style={{ backgroundColor: `rgb(${color.join(',')})`, padding: '2rem' }}>
               Dominant: rgb({color.join(', ')})
           </div>
       );
   }

Vue
===========================================

.. code-block:: javascript

   import { ref, onMounted } from 'vue';
   import { getPalette } from 'colorthief-wasm';

   export default {
       setup(props) {
           const palette = ref([]);

           onMounted(async () => {
               palette.value = await getPalette(props.src, 8);
           });

           return { palette };
       }
   }

.. rubric:: Browser Compatibility

The WASM bindings work in all modern browsers that support:

- WebAssembly
- ``ImageBitmap`` / Canvas API
- ES2020 modules

Tested on Chrome 90+, Firefox 88+, Safari 14+, Edge 90+.

.. rubric:: Performance

The WASM module achieves near-native performance through:

- **LTO Fat optimization** - Whole-program optimization across all crates
- **Single codegen unit** - Maximum inlining and dead code elimination
- **Canvas-based decoding** - Uses browser-native image decoding
- **Shared core** - Same quantization algorithm as the Python bindings

.. rubric:: Build from Source

.. code-block:: bash

   # Install wasm-pack
   cargo install wasm-pack

   # Build
   cd crates/wasm-bindings
   wasm-pack build --release

   # The compiled module is in pkg/
   # pkg/colorthief_wasm.js  - JS glue
   # pkg/colorthief_wasm_bg.wasm - WASM binary
   # pkg/colorthief_wasm.d.ts   - TypeScript types

.. rubric:: Notes

- Images from URLs must be CORS-accessible or same-origin
- Large images may take longer to decode via Canvas
- The quality parameter controls sampling speed vs accuracy (1-10, default 10)
