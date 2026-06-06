# frozen_string_literal: true

require 'spec_helper'

RSpec.describe ColorthiefGpu do
  # ---------------------------------------------------------------------------
  # Pixel helpers  (RGBA -> binary blob, matching the Rust &[u8] contract)
  # ---------------------------------------------------------------------------

  # 10x10 solid red (100 pixels of RGBA)
  let(:solid_red_pixels) do
    pixels = []
    100.times { pixels.push(255, 0, 0, 255) }
    pixels.pack('C*')
  end

  # 10x10 split red / blue (50 red, 50 blue)
  let(:two_color_pixels) do
    pixels = []
    50.times { pixels.push(255, 0, 0, 255) }
    50.times { pixels.push(0, 0, 255, 255) }
    pixels.pack('C*')
  end

  # 5x5 solid green
  let(:solid_green_pixels) do
    pixels = []
    25.times { pixels.push(0, 128, 0, 255) }
    pixels.pack('C*')
  end

  # 3x3 white
  let(:solid_white_pixels) do
    pixels = []
    9.times { pixels.push(255, 255, 255, 255) }
    pixels.pack('C*')
  end

  # 4x4 multi-color gradient (16 unique-ish colors)
  let(:multi_color_pixels) do
    pixels = []
    16.times { |i| pixels.push((i * 16) % 256, (i * 32) % 256, (i * 48) % 256, 255) }
    pixels.pack('C*')
  end

  # ---------------------------------------------------------------------------
  # Helper: run a block, skip gracefully when GPU is unavailable
  # ---------------------------------------------------------------------------

  def with_gpu
    yield
  rescue RuntimeError => e
    skip "GPU unavailable: #{e.message}"
  end

  # ===================================================================
  # Module existence
  # ===================================================================

  describe 'module' do
    it 'defines ColorthiefGpu' do
      expect(described_class).to be_a(Module)
    end

    it 'responds to get_palette' do
      expect(described_class).to respond_to(:get_palette)
    end

    it 'responds to get_color' do
      expect(described_class).to respond_to(:get_color)
    end
  end

  # ===================================================================
  # get_palette
  # ===================================================================

  describe '.get_palette' do
    # --- Solid color detection ---

    describe 'solid color detection' do
      it 'detects solid red' do
        with_gpu do
          palette = described_class.get_palette(solid_red_pixels, 10, 10, 5, 1)
          expect(palette).to include([255, 0, 0])
        end
      end

      it 'detects solid green' do
        with_gpu do
          palette = described_class.get_palette(solid_green_pixels, 5, 5, 5, 1)
          expect(palette).to include([0, 128, 0])
        end
      end

      it 'detects solid white' do
        with_gpu do
          palette = described_class.get_palette(solid_white_pixels, 3, 3, 5, 1)
          expect(palette).to include([255, 255, 255])
        end
      end
    end

    # --- Two-color detection ---

    describe 'two-color detection' do
      it 'finds both red and blue' do
        with_gpu do
          palette = described_class.get_palette(two_color_pixels, 10, 10, 5, 1)
          expect(palette).to include([255, 0, 0])
          expect(palette).to include([0, 0, 255])
        end
      end
    end

    # --- Return value structure ---

    describe 'return value structure' do
      it 'returns a non-empty palette' do
        with_gpu do
          palette = described_class.get_palette(solid_red_pixels, 10, 10, 5, 1)
          expect(palette).to be_an(Array)
          expect(palette).not_to be_empty
        end
      end

      it 'returns valid RGB arrays of length 3' do
        with_gpu do
          palette = described_class.get_palette(solid_red_pixels, 10, 10, 5, 1)
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
    end

    # --- Palette length respects color_count ---

    describe 'color_count bound' do
      it 'returns at most color_count entries for count=3' do
        with_gpu do
          palette = described_class.get_palette(solid_red_pixels, 10, 10, 3, 1)
          expect(palette.length).to be <= 3
        end
      end

      it 'returns at most color_count entries for count=5' do
        with_gpu do
          palette = described_class.get_palette(solid_red_pixels, 10, 10, 5, 1)
          expect(palette.length).to be <= 5
        end
      end
    end

    # --- Deduplication ---

    describe 'deduplication' do
      it 'contains no duplicate colors even with high color_count' do
        with_gpu do
          palette = described_class.get_palette(solid_red_pixels, 10, 10, 255, 1)
          expect(palette.length).to eq(palette.uniq.length)
        end
      end

      it 'returns a reasonable palette size for large color_count' do
        with_gpu do
          palette = described_class.get_palette(solid_red_pixels, 10, 10, 255, 1)
          expect(palette.length).to be > 0
          expect(palette.length).to be <= 255
        end
      end
    end

    # --- Quality parameter ---

    describe 'quality parameter' do
      it 'works with quality=1 (most accurate)' do
        with_gpu do
          palette = described_class.get_palette(solid_red_pixels, 10, 10, 5, 1)
          expect(palette).not_to be_empty
        end
      end

      it 'works with quality=10 (fastest)' do
        with_gpu do
          palette = described_class.get_palette(solid_red_pixels, 10, 10, 5, 10)
          expect(palette).not_to be_empty
        end
      end

      it 'works with quality=5 (middle)' do
        with_gpu do
          palette = described_class.get_palette(solid_red_pixels, 10, 10, 5, 5)
          expect(palette).not_to be_empty
        end
      end
    end

    # --- Deterministic results ---

    describe 'determinism' do
      it 'returns the same palette for identical inputs' do
        with_gpu do
          p1 = described_class.get_palette(solid_red_pixels, 10, 10, 5, 1)
          p2 = described_class.get_palette(solid_red_pixels, 10, 10, 5, 1)
          expect(p1).to eq(p2)
        end
      end

      it 'returns the same palette for two-color input' do
        with_gpu do
          p1 = described_class.get_palette(two_color_pixels, 10, 10, 5, 1)
          p2 = described_class.get_palette(two_color_pixels, 10, 10, 5, 1)
          expect(p1).to eq(p2)
        end
      end
    end

    # --- Different images produce different palettes ---

    describe 'different images' do
      it 'solid red palette differs from solid green palette' do
        with_gpu do
          red_palette   = described_class.get_palette(solid_red_pixels, 10, 10, 5, 1)
          green_palette = described_class.get_palette(solid_green_pixels, 5, 5, 5, 1)
          expect(red_palette).not_to eq(green_palette)
        end
      end
    end

    # --- Edge cases ---

    describe 'edge cases' do
      it 'handles small 1x1 image' do
        with_gpu do
          pixel = [255, 128, 64, 255].pack('C*')
          palette = described_class.get_palette(pixel, 1, 1, 5, 1)
          expect(palette).not_to be_empty
        end
      end

      it 'handles large color_count request on small image' do
        with_gpu do
          palette = described_class.get_palette(solid_white_pixels, 3, 3, 100, 1)
          expect(palette.length).to be <= 100
        end
      end

      it 'handles minimum color_count of 1' do
        with_gpu do
          palette = described_class.get_palette(solid_red_pixels, 10, 10, 1, 1)
          expect(palette.length).to be >= 1
          expect(palette.length).to be <= 1
        end
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
        with_gpu do
          color = described_class.get_color(solid_red_pixels, 10, 10, 1)
          expect(color).to eq([255, 0, 0])
        end
      end

      it 'returns green for solid green image' do
        with_gpu do
          color = described_class.get_color(solid_green_pixels, 5, 5, 1)
          expect(color).to eq([0, 128, 0])
        end
      end

      it 'returns white for solid white image' do
        with_gpu do
          color = described_class.get_color(solid_white_pixels, 3, 3, 1)
          expect(color).to eq([255, 255, 255])
        end
      end
    end

    # --- Return value structure ---

    describe 'return value structure' do
      it 'returns a valid RGB array of length 3' do
        with_gpu do
          color = described_class.get_color(solid_red_pixels, 10, 10, 1)
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

    # --- Quality parameter ---

    describe 'quality parameter' do
      it 'works with quality=1 (most accurate)' do
        with_gpu do
          color = described_class.get_color(solid_red_pixels, 10, 10, 1)
          expect(color.length).to eq(3)
        end
      end

      it 'works with quality=10 (fastest)' do
        with_gpu do
          color = described_class.get_color(solid_red_pixels, 10, 10, 10)
          expect(color.length).to eq(3)
        end
      end

      it 'works with quality=5 (middle)' do
        with_gpu do
          color = described_class.get_color(solid_red_pixels, 10, 10, 5)
          expect(color.length).to eq(3)
        end
      end
    end

    # --- Deterministic results ---

    describe 'determinism' do
      it 'returns the same color for identical inputs' do
        with_gpu do
          c1 = described_class.get_color(solid_red_pixels, 10, 10, 1)
          c2 = described_class.get_color(solid_red_pixels, 10, 10, 1)
          expect(c1).to eq(c2)
        end
      end
    end

    # --- Different images produce different dominant colors ---

    describe 'different images' do
      it 'solid red dominant color differs from solid green' do
        with_gpu do
          red   = described_class.get_color(solid_red_pixels, 10, 10, 1)
          green = described_class.get_color(solid_green_pixels, 5, 5, 1)
          expect(red).not_to eq(green)
        end
      end
    end

    # --- Edge cases ---

    describe 'edge cases' do
      it 'handles small 1x1 image' do
        with_gpu do
          pixel = [200, 100, 50, 255].pack('C*')
          color = described_class.get_color(pixel, 1, 1, 1)
          expect(color.length).to eq(3)
        end
      end
    end

    # --- Error handling for empty/invalid input ---

    describe 'error handling' do
      it 'raises RuntimeError for empty pixel data' do
        with_gpu do
          empty = ''.b
          expect { described_class.get_color(empty, 0, 0, 1) }.to raise_error(RuntimeError)
        end
      end

      it 'raises RuntimeError for zero dimensions' do
        with_gpu do
          expect { described_class.get_color(solid_red_pixels, 0, 0, 1) }.to raise_error(RuntimeError)
        end
      end
    end
  end
end
