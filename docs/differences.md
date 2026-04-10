# Differences

## With `fast-colorthief`

- Supports more architectures. ( `pybind11` vs `pyo3` )
- Doesn't have a hard dependency on `numpy`
- Code is simple compared to `fast-colorthief`'s CPP codebase
- Automated tooling powered by `maturin` and `github-actions`
- The size of `fast-colorthief` is 52kb-60kb,compared to 500kb-700kb for `modern_colorthief`

## With `color-thief-py`

- Superior execution time (nearly 100x)
- Doesn't have a hard dependency on `pillow`
- `color-thief`'s codebase is not in par with modern python versions

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

## With `fast-colorthief` ( c++ )

If you want to get the same output as `fast-colorthief`

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

dominant_color = modern_colorthief.get_color(image_bytes) # method same as `get_dominant_color`
dominant_palette = modern_colorthief.get_palette(image_bytes)
```
