from modern_colorthief import _get_palette
import os

path = os.path.join(os.getcwd(),'test.jpg')

x = _get_palette(path)

print(x)