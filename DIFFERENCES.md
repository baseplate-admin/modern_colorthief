The difference between `color-thief` and `modern_colorthief` is very negligible

For example with this [code](https://github.com/baseplate-admin/modern_colorthief/blob/04831648767f09295abda3cdba723d6f9673e202/examples/test.py),

```python
from modern_colorthief import get_color, get_palette
from colorthief import ColorThief
from pathlib import Path
import os

BASE_DIR = Path(__file__).resolve().parent
path = os.path.join(BASE_DIR, "test.jpg")

print(path)
x = get_color(path)
y = ColorThief(path).get_color()

print(x)
print(y)

m = get_palette(path)
n = ColorThief(path).get_palette()

print(m)
print(n)
```

I get this as output

```python
(201, 160, 118)
(202, 160, 118)


[(30, 169, 166), (179, 51, 55), (219, 176, 127), (248, 233, 225), (160, 98, 87), (63, 47, 42), (131, 163, 107), (179, 119, 52), (237, 220, 155)]
[(31, 167, 164), (179, 51, 55), (219, 176, 127), (248, 233, 225), (160, 98, 87), (62, 44, 38), (131, 162, 106), (178, 118, 51), (242, 220, 157)]
```
