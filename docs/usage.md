# Usage

`modern_colorthief` exposes two functions `get_color` and `get_palette`

## Image Path

How to use with image path:

```python
import modern_colorthief

# Path to any image
path = ...

print(modern_colorthief.get_palette(path)) # returns list[tuple[int,int,int]]
print(modern_colorthief.get_color(path)) # returns tuple[int,int,int]
```

## BytesIO

How to use `modern_colorthief` with a `BytesIO` object:

```python
import io

import modern_colorthief

path = ...

image_bytes = io.BytesIO()

dominant_color = modern_colorthief.get_color(image_bytes)
dominant_palette = modern_colorthief.get_palette(image_bytes)
```

## Pillow Object

How to use `modern_colorthief` with a `Pillow` object:

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

## Numpy Array

How to use `modern_colorthief` with `numpy` array (needs `Pillow`):

```python
import numpy as np
from PIL import Image

arr = ... # Numpy Array

img = Image.fromarray(arr)

image_bytes = io.BytesIO()
img.save(image_bytes, format="PNG")

dominant_color = modern_colorthief.get_color(image_bytes)
dominant_palette = modern_colorthief.get_palette(image_bytes)
```

```{eval-rst}
If there is a difference in the colors returned by `modern_colorthief` check the :doc:`parity <../parity>` documentation
```
