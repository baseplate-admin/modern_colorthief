import io
from PIL import Image

img = Image.open('test.jpg', mode='r')

img_byte_arr = io.BytesIO()
img.save(img_byte_arr, format='PNG')
img_byte_arr = img_byte_arr.getvalue()

print(img_byte_arr)