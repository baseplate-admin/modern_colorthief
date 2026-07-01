import tempfile

from PIL import Image

from modern_colorthief_gpu import (
    extract_dominant_color,
    extract_dominant_color_from_buffer,
    extract_palette,
    extract_palette_from_buffer,
)

img = Image.new("RGB", (100, 100), color=(200, 100, 50))

with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as tmp:
    img.save(tmp.name, format="PNG")
    extract_palette(tmp.name, 5)
    extract_dominant_color(tmp.name)
    tmp.close()
    import os
    os.unlink(tmp.name)

pixels = img.convert("RGBA").tobytes()
extract_palette_from_buffer(pixels, 100, 100, 5)
extract_dominant_color_from_buffer(pixels, 100, 100)
