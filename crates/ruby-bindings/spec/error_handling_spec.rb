require_relative '../lib/colorthief_ruby'

RSpec.describe Colorthief do
  describe '.get_palette error handling' do
    it 'raises on empty pixels' do
      expect {
        described_class.get_palette('', 10, 10, 5, 1)
      }.to raise_error(RuntimeError)
    end

    it 'raises on mismatched pixel data length (too few bytes)' do
      pixels = []
      50.times { pixels.push(255, 0, 0, 255) } # 50 pixels = 200 bytes
      packed = pixels.pack('C*')
      # claim width=10, height=10 => expects 400 bytes
      expect {
        described_class.get_palette(packed, 10, 10, 5, 1)
      }.to raise_error(RuntimeError)
    end

    it 'raises on mismatched pixel data length (too many bytes)' do
      pixels = []
      150.times { pixels.push(0, 255, 0, 255) } # 150 pixels = 600 bytes
      packed = pixels.pack('C*')
      # claim width=10, height=10 => expects 400 bytes
      expect {
        described_class.get_palette(packed, 10, 10, 5, 1)
      }.to raise_error(RuntimeError)
    end

    it 'raises when width is zero' do
      pixels = []
      100.times { pixels.push(255, 0, 0, 255) }
      packed = pixels.pack('C*')
      expect {
        described_class.get_palette(packed, 0, 10, 5, 1)
      }.to raise_error(RuntimeError)
    end

    it 'raises when height is zero' do
      pixels = []
      100.times { pixels.push(255, 0, 0, 255) }
      packed = pixels.pack('C*')
      expect {
        described_class.get_palette(packed, 10, 0, 5, 1)
      }.to raise_error(RuntimeError)
    end

    it 'raises when both dimensions are zero' do
      expect {
        described_class.get_palette('', 0, 0, 5, 1)
      }.to raise_error(RuntimeError)
    end
  end

  describe '.get_palette with quality=0' do
    it 'raises on quality zero (boundary value)' do
      pixels = []
      100.times { pixels.push(255, 0, 0, 255) }
      packed = pixels.pack('C*')
      expect {
        described_class.get_palette(packed, 10, 10, 5, 0)
      }.to raise_error(RuntimeError)
    end
  end

  describe '.get_color error handling' do
    it 'raises on empty pixels' do
      expect {
        described_class.get_color('', 10, 10, 1)
      }.to raise_error(RuntimeError)
    end

    it 'raises on mismatched pixel data length (too few bytes)' do
      pixels = []
      50.times { pixels.push(255, 0, 0, 255) }
      packed = pixels.pack('C*')
      expect {
        described_class.get_color(packed, 10, 10, 1)
      }.to raise_error(RuntimeError)
    end

    it 'raises on mismatched pixel data length (too many bytes)' do
      pixels = []
      150.times { pixels.push(0, 255, 0, 255) }
      packed = pixels.pack('C*')
      expect {
        described_class.get_color(packed, 10, 10, 1)
      }.to raise_error(RuntimeError)
    end

    it 'raises when width is zero' do
      pixels = []
      100.times { pixels.push(255, 0, 0, 255) }
      packed = pixels.pack('C*')
      expect {
        described_class.get_color(packed, 0, 10, 1)
      }.to raise_error(RuntimeError)
    end

    it 'raises when height is zero' do
      pixels = []
      100.times { pixels.push(255, 0, 0, 255) }
      packed = pixels.pack('C*')
      expect {
        described_class.get_color(packed, 10, 0, 1)
      }.to raise_error(RuntimeError)
    end
  end
end
