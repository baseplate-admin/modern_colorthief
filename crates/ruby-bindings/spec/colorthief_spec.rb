# frozen_string_literal: true

require 'spec_helper'

RSpec.describe "Colorthief" do
  skip "CPU extension not available" unless defined?(Colorthief)

  let(:colorthief) { Colorthief }

  # ---------------------------------------------------------------------------
  # Pixel helpers  (RGBA -> binary blob, matching the Rust &[u8] contract)
  # ---------------------------------------------------------------------------

  def solid_pixels(r, g, b, count)
    pixels = []
    count.times { pixels.push(r, g, b, 255) }
    pixels.pack('C*')
  end

  def two_color_pixels(r1, g1, b1, r2, g2, b2, count1: 50, count2: 50)
    pixels = []
    count1.times { pixels.push(r1, g1, b1, 255) }
    count2.times { pixels.push(r2, g2, b2, 255) }
    pixels.pack('C*')
  end

  def build_image(width, height, &pixel_fn)
    pixels = []
    (0...height).each do |y|
      (0...width).each do |x|
        r, g, b = pixel_fn.call(x, y)
        pixels.push(r, g, b, 255)
      end
    end
    pixels.pack('C*')
  end

  # 10x10 solid red
  let(:solid_red_pixels) { solid_pixels(255, 0, 0, 100) }
  # 10x10 solid green
  let(:solid_green_pixels) { solid_pixels(0, 255, 0, 100) }
  # 10x10 solid blue
  let(:solid_blue_pixels) { solid_pixels(0, 0, 255, 100) }
  # 10x10 solid white
  let(:solid_white_pixels) { solid_pixels(255, 255, 255, 100) }
  # 10x10 split red / blue
  let(:red_blue_pixels) { two_color_pixels(255, 0, 0, 0, 0, 255) }
  # 10x10 split red / green
  let(:red_green_pixels) { two_color_pixels(255, 0, 0, 0, 255, 0) }
  # 100x100 solid purple
  let(:large_solid_pixels) { solid_pixels(170, 85, 220, 10_000) }
  # 30x30 horizontal gradient
  let(:gradient_pixels) { build_image(30, 30) { |x, _| [(x * 8) % 256, 128, 64] } }
  # 50x50 checkerboard
  let(:checkerboard_pixels) { build_image(50, 50) { |x, y| ((x + y) % 2 == 0) ? [200, 50, 50] : [50, 50, 200] } }
  # 10x2 wide non-square
  let(:wide_pixels) { build_image(10, 2) { |x, _| x < 5 ? [255, 0, 0] : [0, 0, 255] } }
  # 2x10 tall non-square
  let(:tall_pixels) { build_image(2, 10) { |_, y| y < 5 ? [200, 100, 50] : [50, 100, 200] } }

  # ===================================================================
  # Module existence
  # ===================================================================

  describe 'module' do
    it 'defines Colorthief' do
      expect(colorthief).to be_a(Module)
    end

    it 'responds to get_palette' do
      expect(colorthief).to respond_to(:get_palette)
    end

    it 'responds to get_color' do
      expect(colorthief).to respond_to(:get_color)
    end
  end

  # ===================================================================
  # get_palette
  # ===================================================================

  describe '.get_palette' do
    # --- Solid color detection ---

    describe 'solid color detection' do
      it 'detects solid red' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 5, 1)
        expect(palette).to include([255, 0, 0])
      end

      it 'detects solid green' do
        palette = colorthief.get_palette(solid_green_pixels, 10, 10, 5, 1)
        expect(palette).to include([0, 255, 0])
      end

      it 'detects solid blue' do
        palette = colorthief.get_palette(solid_blue_pixels, 10, 10, 5, 1)
        expect(palette).to include([0, 0, 255])
      end

      it 'detects solid white' do
        palette = colorthief.get_palette(solid_white_pixels, 10, 10, 5, 1)
        expect(palette).to include([255, 255, 255])
      end

      it 'detects solid purple in large image' do
        palette = colorthief.get_palette(large_solid_pixels, 100, 100, 10, 1)
        expect(palette).to include([170, 85, 220])
      end
    end

    # --- Two-color detection ---

    describe 'two-color detection' do
      it 'finds both red and blue' do
        palette = colorthief.get_palette(red_blue_pixels, 10, 10, 5, 1)
        expect(palette).to include([255, 0, 0])
        expect(palette).to include([0, 0, 255])
      end

      it 'finds both red and green' do
        palette = colorthief.get_palette(red_green_pixels, 10, 10, 5, 1)
        expect(palette).to include([255, 0, 0])
        expect(palette).to include([0, 255, 0])
      end
    end

    # --- Return value structure ---

    describe 'return value structure' do
      it 'returns a non-empty palette' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 5, 1)
        expect(palette).to be_an(Array)
        expect(palette).not_to be_empty
      end

      it 'returns valid RGB arrays of length 3' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 5, 1)
        palette.each do |color|
          expect(color).to be_an(Array)
          expect(color.length).to eq(3)
          color.each do |v|
            expect(v).to be_a(Integer)
            expect(v).to be >= 0
            expect(v).to be <= 255
          end
        end
      end
    end

    # --- Palette length respects color_count ---

    describe 'color_count bound' do
      it 'returns at most color_count entries for count=1' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 1, 1)
        expect(palette.length).to be >= 1
        expect(palette.length).to be <= 1
      end

      it 'returns at most color_count entries for count=3' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 3, 1)
        expect(palette.length).to be <= 3
      end

      it 'returns at most color_count entries for count=5' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 5, 1)
        expect(palette.length).to be <= 5
      end

      it 'returns at most color_count entries for count=50' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 50, 1)
        expect(palette.length).to be <= 50
      end

      it 'returns at most color_count entries for count=255' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 255, 1)
        expect(palette.length).to be <= 255
      end
    end

    # --- Deduplication ---

    describe 'deduplication' do
      it 'contains no duplicate colors even with high color_count' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 255, 1)
        expect(palette.length).to eq(palette.uniq.length)
      end

      it 'returns a reasonable palette size for large color_count' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 255, 1)
        expect(palette.length).to be > 0
        expect(palette.length).to be <= 255
      end
    end

    # --- Quality parameter ---

    describe 'quality parameter' do
      it 'works with quality=1 (most accurate)' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 5, 1)
        expect(palette).not_to be_empty
      end

      it 'works with quality=10 (fastest)' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 5, 10)
        expect(palette).not_to be_empty
      end

      it 'works with quality=5 (middle)' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 5, 5)
        expect(palette).not_to be_empty
      end

      it 'quality=0 is clamped to valid' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 5, 0)
        expect(palette).not_to be_empty
      end

      it 'quality=100 works' do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 5, 100)
        expect(palette).not_to be_empty
      end
    end

    # --- Deterministic results ---

    describe 'determinism' do
      it 'returns the same palette for identical inputs' do
        p1 = colorthief.get_palette(solid_red_pixels, 10, 10, 5, 1)
        p2 = colorthief.get_palette(solid_red_pixels, 10, 10, 5, 1)
        expect(p1).to eq(p2)
      end

      it 'returns the same palette for two-color input' do
        p1 = colorthief.get_palette(red_blue_pixels, 10, 10, 5, 1)
        p2 = colorthief.get_palette(red_blue_pixels, 10, 10, 5, 1)
        expect(p1).to eq(p2)
      end
    end

    # --- Different images produce different palettes ---

    describe 'different images' do
      it 'solid red palette differs from solid green palette' do
        red_palette   = colorthief.get_palette(solid_red_pixels, 10, 10, 5, 1)
        green_palette = colorthief.get_palette(solid_green_pixels, 10, 10, 5, 1)
        expect(red_palette).not_to eq(green_palette)
      end
    end

    # --- Consistency ---

    describe 'consistency' do
      it 'dominant color appears in palette' do
        color   = colorthief.get_color(red_blue_pixels, 10, 10, 1)
        palette = colorthief.get_palette(red_blue_pixels, 10, 10, 5, 1)
        expect(palette).to include(color)
      end
    end

    # --- Edge cases ---

    describe 'edge cases' do
      it 'handles small 1x1 image' do
        pixel = [255, 128, 64, 255].pack('C*')
        palette = colorthief.get_palette(pixel, 1, 1, 5, 1)
        expect(palette).not_to be_empty
        expect(palette[0]).to eq([255, 128, 64])
      end

      it 'handles large color_count request on small image' do
        pixel = [100, 150, 200, 255].pack('C*')
        palette = colorthief.get_palette(pixel, 1, 1, 100, 1)
        expect(palette.length).to be <= 100
      end

      it 'handles non-square wide image' do
        palette = colorthief.get_palette(wide_pixels, 10, 2, 5, 1)
        expect(palette).not_to be_empty
      end

      it 'handles non-square tall image' do
        palette = colorthief.get_palette(tall_pixels, 2, 10, 5, 1)
        expect(palette).not_to be_empty
        expect(palette.length).to be <= 5
      end

      it 'handles gradient image' do
        palette = colorthief.get_palette(gradient_pixels, 30, 30, 5, 1)
        expect(palette).not_to be_empty
      end

      it 'handles checkerboard image' do
        palette = colorthief.get_palette(checkerboard_pixels, 50, 50, 5, 1)
        expect(palette).not_to be_empty
      end
    end
  end

  # ===================================================================
  # get_color
  # ===================================================================

  describe '.get_color' do
    # --- Solid color detection ---

    describe 'solid color detection' do
      it 'returns red for solid red image' do
        color = colorthief.get_color(solid_red_pixels, 10, 10, 1)
        expect(color).to eq([255, 0, 0])
      end

      it 'returns green for solid green image' do
        color = colorthief.get_color(solid_green_pixels, 10, 10, 1)
        expect(color).to eq([0, 255, 0])
      end

      it 'returns blue for solid blue image' do
        color = colorthief.get_color(solid_blue_pixels, 10, 10, 1)
        expect(color).to eq([0, 0, 255])
      end

      it 'returns white for solid white image' do
        color = colorthief.get_color(solid_white_pixels, 10, 10, 1)
        expect(color).to eq([255, 255, 255])
      end
    end

    # --- Return value structure ---

    describe 'return value structure' do
      it 'returns a valid RGB array of length 3' do
        color = colorthief.get_color(solid_red_pixels, 10, 10, 1)
        expect(color).to be_an(Array)
        expect(color.length).to eq(3)
        color.each do |v|
          expect(v).to be_a(Integer)
          expect(v).to be >= 0
          expect(v).to be <= 255
        end
      end
    end

    # --- Quality parameter ---

    describe 'quality parameter' do
      it 'works with quality=1 (most accurate)' do
        color = colorthief.get_color(solid_red_pixels, 10, 10, 1)
        expect(color.length).to eq(3)
      end

      it 'works with quality=10 (fastest)' do
        color = colorthief.get_color(solid_red_pixels, 10, 10, 10)
        expect(color.length).to eq(3)
      end

      it 'works with quality=5 (middle)' do
        color = colorthief.get_color(solid_red_pixels, 10, 10, 5)
        expect(color.length).to eq(3)
      end

      it 'quality=0 is clamped to valid' do
        color = colorthief.get_color(solid_red_pixels, 10, 10, 0)
        expect(color.length).to eq(3)
      end

      it 'quality=100 works' do
        color = colorthief.get_color(solid_red_pixels, 10, 10, 100)
        expect(color.length).to eq(3)
      end
    end

    # --- Deterministic results ---

    describe 'determinism' do
      it 'returns the same color for identical inputs' do
        c1 = colorthief.get_color(solid_red_pixels, 10, 10, 1)
        c2 = colorthief.get_color(solid_red_pixels, 10, 10, 1)
        expect(c1).to eq(c2)
      end
    end

    # --- Different images produce different dominant colors ---

    describe 'different images' do
      it 'solid red dominant color differs from solid green' do
        red   = colorthief.get_color(solid_red_pixels, 10, 10, 1)
        green = colorthief.get_color(solid_green_pixels, 10, 10, 1)
        expect(red).not_to eq(green)
      end
    end

    # --- Edge cases ---

    describe 'edge cases' do
      it 'handles small 1x1 image' do
        pixel = [200, 100, 50, 255].pack('C*')
        color = colorthief.get_color(pixel, 1, 1, 1)
        expect(color).to eq([200, 100, 50])
      end

      it 'handles non-square wide image' do
        color = colorthief.get_color(wide_pixels, 10, 2, 1)
        expect(color.length).to eq(3)
      end

      it 'handles non-square tall image' do
        color = colorthief.get_color(tall_pixels, 2, 10, 1)
        expect(color.length).to eq(3)
      end
    end

    # --- Error handling for empty/invalid input ---

    describe 'error handling' do
      it 'raises RuntimeError for empty pixel data' do
        empty = ''.b
        expect { colorthief.get_color(empty, 0, 0, 1) }.to raise_error(RuntimeError)
      end

      it 'raises RuntimeError for zero dimensions' do
        empty = ''.b
        expect { colorthief.get_palette(empty, 0, 0, 5, 1) }.to raise_error(RuntimeError)
      end
    end
  end

  # ===================================================================
  # GC stress — repeated calls should not leak or crash
  # ===================================================================

  describe 'GC stress' do
    it 'survives 50 repeated palette calls' do
      50.times do
        palette = colorthief.get_palette(solid_red_pixels, 10, 10, 5, 1)
        expect(palette).not_to be_empty
      end
    end

    it 'survives 50 repeated color calls' do
      50.times do
        color = colorthief.get_color(solid_red_pixels, 10, 10, 1)
        expect(color.length).to eq(3)
      end
    end

    it 'survives 25 mixed palette + color calls' do
      25.times do
        palette = colorthief.get_palette(red_green_pixels, 10, 10, 5, 1)
        color   = colorthief.get_color(red_green_pixels, 10, 10, 1)
        expect(palette).not_to be_empty
        expect(color.length).to eq(3)
      end
    end
  end
end
