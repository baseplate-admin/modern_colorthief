from modern_colorthief import get_color
from colorthief import ColorThief

import os

path = os.path.join(os.getcwd(), "test.jpg")

x = get_color(path, 10)
y = ColorThief(path).get_color()

print(x)
print(y)
