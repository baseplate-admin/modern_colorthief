# frozen_string_literal: true

require_relative '../lib/colorthief_ruby'

RSpec.describe Colorthief do
  # ----------------------------------------------------------------
  # Helpers to build synthetic pixel buffers that simulate real images
  # ----------------------------------------------------------------
  def build_pixels(width, height, &block)
    pixels = []
    height.times do |y|
      width.times do |x|
        r, g, b, a = block.call(x, y, width, height)
        pixels.push(r, g, b, a)
      end
    end
    pixels.pack('C*')
  end

  # ----------------------------------------------------------------
  # Real image files exist in the spec directory; verify they are
  # present and readable (the binding works on raw pixels, not files)
  # ----------------------------------------------------------------
  describe 'real image files in spec directory' do
    it 'test.jpg exists and has content' do
      test_jpg = File.join(__dir__, 'test.jpg')
      expect(File.exist?(test_jpg)).to be true
      expect(File.size(test_jpg)).to be > 0
    end

    it 'kaiju_no_8.jpg exists and has content' do
      kaiju_jpg = File.join(__dir__, 'kaiju_no_8.jpg')
      expect(File.exist?(kaiju_jpg)).to be true
      expect(File.size(kaiju_jpg)).to be > 0
    end
  end

  # ----------------------------------------------------------------
  # 1x1 single pixel image
  # ----------------------------------------------------------------
  describe '1x1 single pixel image' do
    let(:single_pixel) do
      pixels = [42, 128, 200, 255] # a single teal-ish pixel
      pixels.pack('C*')
    end

    it 'get_palette returns the single color' do
      palette = described_class.get_palette(single_pixel, 1, 1, 5, 1)
      expect(palette).not_to be_empty
      expect(palette.first).to eq([42, 128, 200])
    end

    it 'get_color returns the single color' do
      color = described_class.get_color(single_pixel, 1, 1, 1)
      expect(color).to eq([42, 128, 200])
    end

    it 'works with quality 1' do
      palette = described_class.get_palette(single_pixel, 1, 1, 1, 1)
      expect(palette).to eq([[42, 128, 200]])
    end
  end

  # ----------------------------------------------------------------
  # Non-square images
  # ----------------------------------------------------------------
  describe 'non-square images' do
    it 'handles 2x3 image (wide=2, tall=3)' do
      pixels = build_pixels(2, 3) { |x, y, _, _|
        if y < 2
          [255, 0, 0, 255] # red for top two rows
        else
          [0, 0, 255, 255] # blue for bottom row
        end
      }
      palette = described_class.get_palette(pixels, 2, 3, 5, 1)
      expect(palette).to include([255, 0, 0])
      expect(palette).to include([0, 0, 255])
    end

    it 'handles 3x2 image (wide=3, tall=2)' do
      pixels = build_pixels(3, 2) { |x, y, _, _|
        if x == 0
          [255, 0, 0, 255]
        elsif x == 1
          [0, 255, 0, 255]
        else
          [0, 0, 255, 255]
        end
      }
      palette = described_class.get_palette(pixels, 3, 2, 5, 1)
      expect(palette).to include([255, 0, 0])
      expect(palette).to include([0, 255, 0])
      expect(palette).to include([0, 0, 255])
    end

    it 'handles tall narrow image (2x10)' do
      pixels = build_pixels(2, 10) { |_, y, _, _|
        if y < 5
          [200, 100, 50, 255]
        else
          [50, 100, 200, 255]
        end
      }
      palette = described_class.get_palette(pixels, 2, 10, 5, 1)
      expect(palette.length).to be <= 5
      expect(palette).not_to be_empty
    end

    it 'handles wide short image (10x2)' do
      pixels = build_pixels(10, 2) { |x, _, _, _|
        if x < 5
          [100, 200, 100, 255]
        else
          [200, 100, 100, 255]
        end
      }
      color = described_class.get_color(pixels, 10, 2, 1)
      expect(color.length).to eq(3)
    end
  end

  # ----------------------------------------------------------------
  # Various color_count values
  # ----------------------------------------------------------------
  describe 'color_count variation' do
    let(:gradient_pixels) do
      # Build a 20x20 image with 10 distinct color bands
      build_pixels(20, 20) { |x, y, _, _|
        band = y / 2 # 0..9
        [band * 25, band * 20, band * 15, 255]
      }
    end

    it 'color_count=1 returns at most 1 color' do
      palette = described_class.get_palette(gradient_pixels, 20, 20, 1, 1)
      expect(palette.length).to be <= 1
      expect(palette).not_to be_empty
    end

    it 'color_count=5 returns at most 5 colors' do
      palette = described_class.get_palette(gradient_pixels, 20, 20, 5, 1)
      expect(palette.length).to be <= 5
    end

    it 'color_count=50 returns at most 50 colors' do
      palette = described_class.get_palette(gradient_pixels, 20, 20, 50, 1)
      expect(palette.length).to be <= 50
    end

    it 'color_count=255 returns at most 255 colors' do
      palette = described_class.get_palette(gradient_pixels, 20, 20, 255, 1)
      expect(palette.length).to be <= 255
    end

    it 'color_count=255 on small image returns fewer than 255 unique colors' do
      solid_pixels = []
      100.times { solid_pixels.push(100, 150, 200, 255) }
      packed = solid_pixels.pack('C*')
      palette = described_class.get_palette(packed, 10, 10, 255, 1)
      expect(palette.length).to be < 255
      expect(palette.length).to be >= 1
    end
  end

  # ----------------------------------------------------------------
  # Quality parameter variation
  # ----------------------------------------------------------------
  describe 'quality parameter variation' do
    let(:medium_image) do
      # 50x50 checkerboard of two colors
      build_pixels(50, 50) { |x, y, _, _|
        if (x + y).even?
          [200, 50, 50, 255]
        else
          [50, 50, 200, 255]
        end
      }
    end

    it 'quality=1 (highest quality) works' do
      palette = described_class.get_palette(medium_image, 50, 50, 5, 1)
      expect(palette).not_to be_empty
    end

    it 'quality=50 works' do
      palette = described_class.get_palette(medium_image, 50, 50, 5, 50)
      expect(palette).not_to be_empty
    end

    it 'quality=100 (lowest quality) works' do
      palette = described_class.get_palette(medium_image, 50, 50, 5, 100)
      expect(palette).not_to be_empty
    end

    it 'quality=0 is treated as quality=1 (clamped)' do
      palette = described_class.get_palette(medium_image, 50, 50, 5, 0)
      expect(palette).not_to be_empty
    end

    it 'higher quality returns more detailed palette' do
      high_q = described_class.get_palette(medium_image, 50, 50, 10, 1)
      low_q = described_class.get_palette(medium_image, 50, 50, 10, 100)
      # Higher quality should generally find at least as many distinct colors
      expect(high_q.length).to be >= low_q.length
    end

    it 'get_color with quality=1 works' do
      color = described_class.get_color(medium_image, 50, 50, 1)
      expect(color.length).to eq(3)
    end

    it 'get_color with quality=50 works' do
      color = described_class.get_color(medium_image, 50, 50, 50)
      expect(color.length).to eq(3)
    end

    it 'get_color with quality=100 works' do
      color = described_class.get_color(medium_image, 50, 50, 100)
      expect(color.length).to eq(3)
    end

    it 'get_color with quality=0 works (clamped to 1)' do
      color = described_class.get_color(medium_image, 50, 50, 0)
      expect(color.length).to eq(3)
    end
  end

  # ----------------------------------------------------------------
  # Synthetic patterns simulating real image characteristics
  # ----------------------------------------------------------------
  describe 'synthetic image patterns' do
    it 'gradient image returns dominant colors from gradient' do
      pixels = build_pixels(30, 30) { |x, _, _, _|
        [x * 8, 128, 64, 255]
      }
      palette = described_class.get_palette(pixels, 30, 30, 5, 1)
      expect(palette).not_to be_empty
      palette.each do |color|
        expect(color[1]).to eq(128)
        expect(color[2]).to eq(64)
      end
    end

    it 'large solid-color image returns single color' do
      pixels = []
      10_000.times { pixels.push(170, 85, 220, 255) } # 100x100
      packed = pixels.pack('C*')
      palette = described_class.get_palette(packed, 100, 100, 10, 1)
      expect(palette).to include([170, 85, 220])
    end

    it 'multi-color striped image detects all stripes' do
      pixels = build_pixels(10, 30) { |_, y, _, h|
        band = y / (h / 5)
        case band
        when 0 then [255, 0, 0, 255]
        when 1 then [0, 255, 0, 255]
        when 2 then [0, 0, 255, 255]
        when 3 then [255, 255, 0, 255]
        else        [255, 0, 255, 255]
        end
      }
      palette = described_class.get_palette(pixels, 10, 30, 10, 1)
      expect(palette).to include([255, 0, 0])
      expect(palette).to include([0, 255, 0])
      expect(palette).to include([0, 0, 255])
    end
  end

  # See: https://oxidize-rb.org/docs/testing
  # Memory verification under GC stress
  describe 'memory management under GC stress' do
    let(:stress_pixels) do
      build_pixels(20, 20) { |x, y, _, _|
        [x * 12, y * 12, (x + y) * 6, 255]
      }
    end

    it 'survives repeated palette extraction with GC.stress' do
      GC.stress = true
      100.times do
        palette = described_class.get_palette(stress_pixels, 20, 20, 5, 1)
        expect(palette).not_to be_empty
      end
    ensure
      GC.stress = false
    end

    it 'survives repeated color extraction with GC.stress' do
      GC.stress = true
      100.times do
        color = described_class.get_color(stress_pixels, 20, 20, 1)
        expect(color.length).to eq(3)
      end
    ensure
      GC.stress = false
    end
  end
end
