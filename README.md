# Introduction :

[`ColorThief`](https://github.com/fengsp/color-thief-py) reimagined

## Why use `modern_colorthief` ?

-   It is built on modern standards (`colorthief` uses [custom `cached_property`](https://github.com/fengsp/color-thief-py/blob/3e96a52abfa34323c798a691b2970c6df3059fda/colorthief.py#L18-L27))
-   I want innovations to happen in `colorthief` (optimize algorithms? make everything faster?)
-   I want to optimize parts of `colorthief` (maybe rewrite in a compiled language?)
-   Automated [test](https://github.com/baseplate-admin/modern_colorthief/blob/7f1025c853bf9458e123a43d284099523a8a587b/tests/test_modern_colortheif_with_colorthief.py#L10-L16) to make sure `colorthief` and `modern_colorthief` gives the same output

## Why shouldn't you use `modern_colorthief` ?

-   `modern_colorthief` does not support EOL python versions.

## Requirements :

-   [`Pillow`](https://pypi.org/project/Pillow/)
-   Python 3

## Examples :

Here is a minimal example :

```python
from modern_colorthief import ColorThief

image = '' # Path to a image

ColorThief(image).get_color()

```

<center><sub> If you want a comprehensive example please visit the <a href="https://github.com/baseplate-admin/modern_colorthief/blob/7b1a02ca44ca1c7b8e63cd4818caf1a506c18fde/tests/test_modern_colortheif_with_colorthief.py">test</a> file</sub></center>

## Migration from `colorthief` :

```diff
- from colorthief import ColorThief
+ from modern_colorthief import ColorThief


image = '' # Path to a image

ColorThief(image).get_color()

```

## Used Internally by :

-   [`coreproject`](https://github.com/baseplate-admin/coreproject)

## Contributing :

If you like this project add a star.
If you have problems or suggestions please put them in the [Issue Tracker](https://github.com/baseplate-admin/modern_colorthief/issues)
If you like to add features. Fork this repo and submit a Pull Request. 😛

# Roadmap :

You tell me. If i have free time, I will implement it.
