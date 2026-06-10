"""GPU-accelerated palette extraction tests."""
import pytest

# Try importing the GPU module; skip all tests if unavailable
try:
    from modern_colorthief_gpu import (
        extract_palette_from_buffer,
        extract_dominant_color_from_buffer,
    )
    gpu_available = True
except ImportError:
    gpu_available = False


@pytest.fixture
def solid_red_pixels():
    """100x100 solid red RGBA pixels."""
    return bytes([255, 0, 0, 255] * 10_000)


@pytest.fixture
def solid_green_pixels():
    """100x100 solid green RGBA pixels."""
    return bytes([0, 255, 0, 255] * 10_000)


@pytest.fixture
def solid_blue_pixels():
    """100x100 solid blue RGBA pixels."""
    return bytes([0, 0, 255, 255] * 10_000)


@pytest.mark.skipif(not gpu_available, reason="GPU module not available")
class TestGpuPalette:
    """Tests for GPU palette extraction."""

    def test_palette_not_empty(self, solid_red_pixels):
        palette = extract_palette_from_buffer(solid_red_pixels, 100, 100)
        assert len(palette) > 0

    def test_palette_valid_rgb(self, solid_red_pixels):
        palette = extract_palette_from_buffer(solid_red_pixels, 100, 100)
        for r, g, b in palette:
            assert 0 <= r <= 255
            assert 0 <= g <= 255
            assert 0 <= b <= 255

    def test_solid_red_dominant(self, solid_red_pixels):
        palette = extract_palette_from_buffer(solid_red_pixels, 100, 100)
        r, g, b = palette[0]
        assert r > 200
        assert g < 55
        assert b < 55

    def test_solid_green_dominant(self, solid_green_pixels):
        palette = extract_palette_from_buffer(solid_green_pixels, 100, 100)
        r, g, b = palette[0]
        assert r < 55
        assert g > 200
        assert b < 55

    def test_solid_blue_dominant(self, solid_blue_pixels):
        palette = extract_palette_from_buffer(solid_blue_pixels, 100, 100)
        r, g, b = palette[0]
        assert r < 55
        assert g < 55
        assert b > 200

    def test_color_count_bound(self, solid_red_pixels):
        palette = extract_palette_from_buffer(solid_red_pixels, 100, 100, size=5)
        assert len(palette) <= 5

    def test_no_duplicate_colors(self, solid_red_pixels):
        palette = extract_palette_from_buffer(solid_red_pixels, 100, 100, size=10)
        assert len(palette) == len(set(palette))

    def test_deterministic(self, solid_red_pixels):
        p1 = extract_palette_from_buffer(solid_red_pixels, 100, 100)
        p2 = extract_palette_from_buffer(solid_red_pixels, 100, 100)
        assert p1 == p2

    def test_quality_parameter(self, solid_red_pixels):
        for q in (1, 5, 10):
            palette = extract_palette_from_buffer(solid_red_pixels, 100, 100, quality=q)
            assert len(palette) > 0

    def test_error_on_empty_pixels(self):
        with pytest.raises((ValueError, RuntimeError)):
            extract_palette_from_buffer(b"", 0, 0)

    def test_error_on_zero_width(self, solid_red_pixels):
        with pytest.raises((ValueError, RuntimeError)):
            extract_palette_from_buffer(solid_red_pixels, 0, 100)

    def test_error_on_zero_height(self, solid_red_pixels):
        with pytest.raises((ValueError, RuntimeError)):
            extract_palette_from_buffer(solid_red_pixels, 100, 0)


@pytest.mark.skipif(not gpu_available, reason="GPU module not available")
class TestGpuColor:
    """Tests for GPU dominant color extraction."""

    def test_color_is_tuple_of_3(self, solid_red_pixels):
        color = extract_dominant_color_from_buffer(solid_red_pixels, 100, 100)
        assert len(color) == 3

    def test_color_valid_rgb(self, solid_red_pixels):
        r, g, b = extract_dominant_color_from_buffer(solid_red_pixels, 100, 100)
        assert 0 <= r <= 255
        assert 0 <= g <= 255
        assert 0 <= b <= 255

    def test_solid_red_color(self, solid_red_pixels):
        r, g, b = extract_dominant_color_from_buffer(solid_red_pixels, 100, 100)
        assert r > 200
        assert g < 55
        assert b < 55

    def test_deterministic(self, solid_red_pixels):
        c1 = extract_dominant_color_from_buffer(solid_red_pixels, 100, 100)
        c2 = extract_dominant_color_from_buffer(solid_red_pixels, 100, 100)
        assert c1 == c2

    def test_error_on_empty_pixels(self):
        with pytest.raises((ValueError, RuntimeError)):
            extract_dominant_color_from_buffer(b"", 0, 0)
