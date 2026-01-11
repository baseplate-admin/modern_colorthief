# Command Line Interface

`modern_colorthief` comes with a built-in command line interface.

## Usage

```bash
modern-colorthief [OPTIONS] FILE
```

## Arguments

- `FILE`: Path to the image file.

## Options

- `--palette`: Get the palette of dominant colors instead of a single dominant color.
- `--quality`: Quality (default: 10). Higher is faster but less accurate.
- `--count`: Color count for palette (default: 5). Only used with `--palette`.

## Examples

**Get dominant color:**

```bash
modern-colorthief path/to/image.png
# Output: #f0e68c
```

**Get palette:**

```bash
modern-colorthief path/to/image.png --palette
# Output:
# #f0e68c
# #556b2f
# ...
```
