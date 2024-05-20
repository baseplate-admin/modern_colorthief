# Parity

## With `colorthief` ( python )

If you want to get the same output as `colorthief`.

1. Load the image with `Pillow`.
2. Save the loaded image in a `BytesIO` object.
3. Pass the `BytesIO` object to `modern_colorthief`

Code example:

```python
import io
from PIL import Image

import modern_colorthief

path = ...
img = Image.open(path, mode="r")

image_bytes = io.BytesIO()
img.save(image_bytes, format="PNG")

dominant_color = modern_colorthief.get_color(image_bytes)
dominant_palette = modern_colorthief.get_palette(image_bytes)

```
