# Usage

`modern_colorthief` exposes two functions `get_color` and `get_palette`

Here is how to use `get_color`:

```python
from modern_colorthief import get_color

# Path to any image
path = ...

print(get_color(path)) # returns tuple[int,int,int]
```

Here is how to use `get_palette`:

```python
from modern_colorthief import get_color

# Path to any image
path = ...

print(get_palette(path)) # returns list[tuple[int,int,int]]
```
