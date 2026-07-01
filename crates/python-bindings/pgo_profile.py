from modern_colorthief import get_color, get_palette
from PIL import Image

img = Image.new("RGB", (100, 100), color=(200, 100, 50))
get_palette(img, 5)
get_color(img)
