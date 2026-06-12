require_relative '../lib/colorthief_ruby'

RSpec.describe 'Consistency with Python test suite' do
  # Build a 10x10 image: 80% red pixels, 20% blue pixels
  let(:red_dominant_pixels) do
    pixels = []
    80.times { pixels.push(255, 0, 0, 255) }
    20.times { pixels.push(0, 0, 255, 255) }
    pixels.pack('C*')
  end

  let(:green_pixels) do
    pixels = []
    100.times { pixels.push(0, 255, 0, 255) }
    pixels.pack('C*')
  end

  describe '.get_color vs .get_palette consistency' do
    it 'get_color result is in the palette' do
      color = described_class.get_color(red_dominant_pixels, 10, 10, 1)
      palette = described_class.get_palette(red_dominant_pixels, 10, 10, 5, 1)
      expect(palette).to include(color)
    end

    it 'get_color matches first palette entry' do
      color = described_class.get_color(red_dominant_pixels, 10, 10, 1)
      palette = described_class.get_palette(red_dominant_pixels, 10, 10, 5, 1)
      expect(palette.first).to eq(color)
    end
  end

  describe '.get_color dominance' do
    it 'returns the most frequent color' do
      color = described_class.get_color(red_dominant_pixels, 10, 10, 1)
      expect(color).to eq([255, 0, 0])
    end
  end

  describe 'Different images produce different colors' do
    it 'red image returns red, green image returns green' do
      require 'stringio'
      red_pixels = []
      100.times { red_pixels.push(255, 0, 0, 255) }
      red_pixels = red_pixels.pack('C*')

      red_color = described_class.get_color(red_pixels, 10, 10, 1)
      green_color = described_class.get_color(green_pixels, 10, 10, 1)
      expect(red_color).not_to eq(green_color)
      expect(red_color).to eq([255, 0, 0])
      expect(green_color).to eq([0, 255, 0])
    end
  end

  describe 'Quality boundary' do
    it 'works for all quality values 1-10' do
      (1..10).each do |q|
        palette = described_class.get_palette(red_dominant_pixels, 10, 10, 5, q)
        expect(palette).not_to be_empty
        expect(palette.length).to be <= 5
      end
    end
  end

  describe 'Thread safety' do
    it 'handles concurrent calls without data races' do
      require 'thread'

      pixels = []
      100.times { pixels.push(255, 128, 64, 255) }
      pixels = pixels.pack('C*')

      results = []
      threads = []

      4.times do
        t = Thread.new do
          10.times do
            color = described_class.get_color(pixels, 10, 10, 1)
            results << color
          end
        end
        threads << t
      end

      threads.each(&:join)

      expect(results.length).to eq(40)
      expect(results.uniq).to eq([[255, 128, 64]])
    end

    it 'handles concurrent palette calls' do
      require 'thread'

      results = []
      threads = []

      3.times do
        t = Thread.new do
                    5.times do
              palette = described_class.get_palette(red_dominant_pixels, 10, 10, 3, 1)
              results << palette
            end
          end
          threads << t
        end

      threads.each(&:join)

      expect(results.length).to eq(15)
      results.each do |palette|
        expect(palette.length).to be <= 3
        expect(palette).not_to be_empty
      end
    end
  end
end
