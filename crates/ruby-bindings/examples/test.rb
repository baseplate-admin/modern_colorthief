require 'colorthief_ruby'

image_path = File.join(__dir__, 'test.jpg')
pixels = File.binread(image_path)

# Note: This expects raw RGBA pixel data. In practice, use RMagick or MiniMagick to decode.
# palette = Colorthief.get_palette(pixels, width, height, 10, 10)
# puts "Palette: #{palette}"
#
# color = Colorthief.get_color(pixels, width, height, 10)
# puts "Dominant color: #{color}"
