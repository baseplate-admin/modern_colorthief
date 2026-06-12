# frozen_string_literal: true

require_relative '../lib/colorthief_ruby'

RSpec.describe Colorthief do
  describe '.get_palette error handling' do
    it 'raises on empty pixels' do
      expect {
        described_class.get_palette('', 10, 10, 5, 1)
      }.to raise_error(RuntimeError)
    end

    it 'raises when both dimensions are zero with empty buffer' do
      expect {
        described_class.get_palette('', 0, 0, 5, 1)
      }.to raise_error(RuntimeError)
    end
  end

  describe '.get_color error handling' do
    it 'raises on empty pixels' do
      expect {
        described_class.get_color('', 10, 10, 1)
      }.to raise_error(RuntimeError)
    end
  end
end
