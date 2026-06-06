require_relative '../lib/colorthief_ruby'

RSpec.describe Colorthief do
  let(:solid_red_pixels) do
    pixels = []
    100.times { pixels.push(255, 0, 0, 255) }
    pixels.pack('C*')
  end

  let(:two_color_pixels) do
    pixels = []
    50.times { pixels.push(255, 0, 0, 255) }
    50.times { pixels.push(0, 0, 255, 255) }
    pixels.pack('C*')
  end

  describe '.get_palette' do
    it 'returns a non-empty palette' do
      palette = described_class.get_palette(solid_red_pixels, 10, 10, 5, 1)
      expect(palette).not_to be_empty
    end

    it 'returns valid RGB arrays' do
      palette = described_class.get_palette(solid_red_pixels, 10, 10, 5, 1)
      palette.each do |color|
        expect(color).to all(be_a(Integer))
        expect(color.length).to eq(3)
        color.each do |v|
          expect(v).to be >= 0 && be <= 255
        end
      end
    end

    it 'detects solid red' do
      palette = described_class.get_palette(solid_red_pixels, 10, 10, 5, 1)
      expect(palette).to include([255, 0, 0])
    end

    it 'detects two colors' do
      palette = described_class.get_palette(two_color_pixels, 10, 10, 5, 1)
      expect(palette).to include([255, 0, 0])
      expect(palette).to include([0, 0, 255])
    end

    it 'is deduplicated' do
      palette = described_class.get_palette(solid_red_pixels, 10, 10, 255, 1)
      expect(palette.length).to eq(palette.uniq.length)
    end

    it 'respects color_count' do
      [3, 5].each do |count|
        palette = described_class.get_palette(solid_red_pixels, 10, 10, count, 1)
        expect(palette.length).to be <= count
      end
    end
  end

  describe '.get_color' do
    it 'returns a valid RGB array' do
      color = described_class.get_color(solid_red_pixels, 10, 10, 1)
      expect(color).to all(be_a(Integer))
      expect(color.length).to eq(3)
    end

    it 'returns red for solid red' do
      color = described_class.get_color(solid_red_pixels, 10, 10, 1)
      expect(color).to eq([255, 0, 0])
    end

    it 'is deterministic' do
      c1 = described_class.get_color(solid_red_pixels, 10, 10, 1)
      c2 = described_class.get_color(solid_red_pixels, 10, 10, 1)
      expect(c1).to eq(c2)
    end
  end
end
