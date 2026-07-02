"""GPU-accelerated palette extraction tests."""
from pathlib import Path

import pytest

# Try importing the GPU module; skip all tests if unavailable
try:
    from modern_colorthief_gpu import (
        extract_palette_from_buffer,
        extract_dominant_color_from_buffer,
        extract_palette,
        extract_dominant_color,
        list_gpus,
    )
    gpu_available = True
except ImportError:
    gpu_available = False


# ---------------------------------------------------------------------------
# Pixel fixtures
# ---------------------------------------------------------------------------

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


@pytest.fixture
def two_color_pixels():
    """10x10 split red/blue (50 red, 50 blue)."""
    return bytes([255, 0, 0, 255] * 50 + [0, 0, 255, 255] * 50)


@pytest.fixture
def gradient_pixels():
    """20x10 horizontal gradient."""
    pixels = []
    for x in range(20):
        for _ in range(10):
            pixels.extend([(x * 13) % 256, (x * 7) % 256, (x * 5) % 256, 255])
    return bytes(pixels)


@pytest.fixture
def checkerboard_pixels():
    """10x10 checkerboard red/blue."""
    pixels = []
    for y in range(10):
        for x in range(10):
            if (x + y) % 2 == 0:
                pixels.extend([200, 50, 50, 255])
            else:
                pixels.extend([50, 50, 200, 255])
    return bytes(pixels)


# ---------------------------------------------------------------------------
# Palette tests
# ---------------------------------------------------------------------------

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

    # -- Error handling --

    def test_error_on_empty_pixels(self):
        with pytest.raises((ValueError, RuntimeError)):
            extract_palette_from_buffer(b"", 0, 0)

    def test_error_on_zero_width(self, solid_red_pixels):
        with pytest.raises((ValueError, RuntimeError)):
            extract_palette_from_buffer(solid_red_pixels, 0, 100)

    def test_error_on_zero_height(self, solid_red_pixels):
        with pytest.raises((ValueError, RuntimeError)):
            extract_palette_from_buffer(solid_red_pixels, 100, 0)

    # -- Two-color detection --

    def test_two_color_red_blue(self, two_color_pixels):
        palette = extract_palette_from_buffer(two_color_pixels, 10, 10, size=5, quality=1)
        rgb_set = set(palette)
        assert any(r > 200 and g < 55 and b < 55 for r, g, b in rgb_set), "should detect red"
        assert any(r < 55 and g < 55 and b > 200 for r, g, b in rgb_set), "should detect blue"

    # -- Gradient image --

    def test_gradient_returns_multiple_colors(self, gradient_pixels):
        palette = extract_palette_from_buffer(gradient_pixels, 20, 10, size=10, quality=1)
        assert len(palette) > 1, "gradient should produce >1 color"

    # -- Checkerboard --

    def test_checkerboard(self, checkerboard_pixels):
        palette = extract_palette_from_buffer(checkerboard_pixels, 10, 10, size=5, quality=1)
        assert len(palette) > 0

    # -- 1x1 single pixel --

    def test_single_pixel(self):
        pixel = bytes([42, 100, 200, 255])
        palette = extract_palette_from_buffer(pixel, 1, 1, size=5, quality=1)
        assert len(palette) > 0
        assert (42, 100, 200) in palette

    # -- Non-square wide --

    def test_wide_image(self, solid_red_pixels):
        palette = extract_palette_from_buffer(solid_red_pixels, 200, 50)
        assert len(palette) > 0

    # -- Non-square tall --

    def test_tall_image(self, solid_red_pixels):
        palette = extract_palette_from_buffer(solid_red_pixels, 50, 200)
        assert len(palette) > 0

    # -- Quality=0 clamped --

    def test_quality_zero_clamped(self, solid_red_pixels):
        palette = extract_palette_from_buffer(solid_red_pixels, 100, 100, quality=0)
        assert len(palette) > 0

    # -- Quality=100 --

    def test_quality_100(self, solid_red_pixels):
        palette = extract_palette_from_buffer(solid_red_pixels, 100, 100, quality=100)
        assert len(palette) > 0

    # -- Different images produce different palettes --

    def test_different_images_different_palette(self, solid_red_pixels, solid_blue_pixels):
        p1 = extract_palette_from_buffer(solid_red_pixels, 100, 100)
        p2 = extract_palette_from_buffer(solid_blue_pixels, 100, 100)
        assert p1 != p2

    # -- Dominant color appears in palette --

    def test_dominant_in_palette(self, two_color_pixels):
        color = extract_dominant_color_from_buffer(two_color_pixels, 10, 10, quality=1)
        palette = extract_palette_from_buffer(two_color_pixels, 10, 10, size=5, quality=1)
        assert color in palette

    # -- GC stress --

    def test_gc_stress_palette(self, solid_red_pixels):
        for _ in range(50):
            palette = extract_palette_from_buffer(solid_red_pixels, 100, 100)
            assert len(palette) > 0

    def test_gc_stress_color(self, solid_red_pixels):
        for _ in range(50):
            color = extract_dominant_color_from_buffer(solid_red_pixels, 100, 100)
            assert len(color) == 3

    def test_gc_stress_mixed(self, solid_red_pixels):
        for _ in range(25):
            palette = extract_palette_from_buffer(solid_red_pixels, 100, 100)
            color = extract_dominant_color_from_buffer(solid_red_pixels, 100, 100)
            assert len(palette) > 0
            assert len(color) == 3


# ---------------------------------------------------------------------------
# Dominant color tests
# ---------------------------------------------------------------------------

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

    def test_solid_green_color(self, solid_green_pixels):
        r, g, b = extract_dominant_color_from_buffer(solid_green_pixels, 100, 100)
        assert r < 55
        assert g > 200
        assert b < 55

    def test_solid_blue_color(self, solid_blue_pixels):
        r, g, b = extract_dominant_color_from_buffer(solid_blue_pixels, 100, 100)
        assert r < 55
        assert g < 55
        assert b > 200

    def test_deterministic(self, solid_red_pixels):
        c1 = extract_dominant_color_from_buffer(solid_red_pixels, 100, 100)
        c2 = extract_dominant_color_from_buffer(solid_red_pixels, 100, 100)
        assert c1 == c2

    def test_error_on_empty_pixels(self):
        with pytest.raises((ValueError, RuntimeError)):
            extract_dominant_color_from_buffer(b"", 0, 0)

    # -- 1x1 single pixel --

    def test_single_pixel_color(self):
        pixel = bytes([200, 100, 50, 255])
        color = extract_dominant_color_from_buffer(pixel, 1, 1)
        assert color == (200, 100, 50)

    # -- Different images produce different dominant colors --

    def test_different_images_different_color(self, solid_red_pixels, solid_green_pixels):
        c1 = extract_dominant_color_from_buffer(solid_red_pixels, 100, 100)
        c2 = extract_dominant_color_from_buffer(solid_green_pixels, 100, 100)
        assert c1 != c2

    # -- API surface tests --

    def test_module_exports_palette(self):
        import modern_colorthief_gpu as m
        assert hasattr(m, "extract_palette_from_buffer")
        assert callable(m.extract_palette_from_buffer)

    def test_module_exports_color(self):
        import modern_colorthief_gpu as m
        assert hasattr(m, "extract_dominant_color_from_buffer")
        assert callable(m.extract_dominant_color_from_buffer)


    # -- Solid white detection --

    def test_solid_white_dominant(self):
        pixels = bytes([255, 255, 255, 255] * 100)
        palette = extract_palette_from_buffer(pixels, 10, 10)
        r, g, b = palette[0]
        assert r > 200
        assert g > 200
        assert b > 200

    # -- Solid black detection --

    def test_solid_black_dominant(self):
        pixels = bytes([0, 0, 0, 255] * 100)
        palette = extract_palette_from_buffer(pixels, 10, 10)
        r, g, b = palette[0]
        assert r < 55
        assert g < 55
        assert b < 55

    # -- Dominant color reflects majority --

    def test_dominant_reflects_majority(self):
        pixels = bytes([255, 0, 0, 255] * 90 + [0, 0, 255, 255] * 10)
        color = extract_dominant_color_from_buffer(pixels, 10, 10, quality=1)
        assert color[0] > 200, "dominant should be red for 90/10 split"

    # -- Palette with quality maximum --

    def test_palette_quality_max(self, solid_red_pixels):
        palette = extract_palette_from_buffer(solid_red_pixels, 100, 100, quality=10)
        assert len(palette) > 0

    # -- Color count=0 edge case --

    def test_color_count_zero(self, solid_red_pixels):
        palette = extract_palette_from_buffer(solid_red_pixels, 100, 100, size=0)
        assert len(palette) == 0

    # -- __version__ attribute --

    def test_version_exists(self):
        import modern_colorthief_gpu as m
        assert hasattr(m, "__version__")

    def test_version_is_string(self):
        import modern_colorthief_gpu as m
        assert isinstance(m.__version__, str)

    def test_version_not_empty(self):
        import modern_colorthief_gpu as m
        assert len(m.__version__) > 0

    def test_version_format_semver_like(self):
        import modern_colorthief_gpu as m
        parts = m.__version__.split(".")
        assert len(parts) >= 2
        assert all(p.isdigit() for p in parts[:2])

    def test_version_no_whitespace(self):
        import modern_colorthief_gpu as m
        assert m.__version__.strip() == m.__version__


# ---------------------------------------------------------------------------
# File path API tests
# ---------------------------------------------------------------------------

@pytest.mark.skipif(not gpu_available, reason="GPU module not available")
class TestGpuFilePath:
    """Tests for GPU file-path extraction functions."""

    def test_extract_palette_from_file(self):
        palette = extract_palette(str(Path(__file__).parent / "test.jpg"))
        assert len(palette) > 0
        for r, g, b in palette:
            assert 0 <= r <= 255
            assert 0 <= g <= 255
            assert 0 <= b <= 255

    def test_extract_dominant_color_from_file(self):
        color = extract_dominant_color(str(Path(__file__).parent / "test.jpg"))
        assert len(color) == 3
        assert 0 <= color[0] <= 255
        assert 0 <= color[1] <= 255
        assert 0 <= color[2] <= 255

    def test_extract_palette_different_images(self):
        p1 = extract_palette(str(Path(__file__).parent / "test.jpg"))
        p2 = extract_palette(str(Path(__file__).parent / "kaiju_no_8.jpg"))
        assert p1 != p2

    def test_extract_dominant_color_different_images(self):
        c1 = extract_dominant_color(str(Path(__file__).parent / "test.jpg"))
        c2 = extract_dominant_color(str(Path(__file__).parent / "kaiju_no_8.jpg"))
        assert c1 != c2

    def test_extract_palette_quality(self):
        for q in (1, 5, 10):
            palette = extract_palette(str(Path(__file__).parent / "test.jpg"), quality=q)
            assert len(palette) > 0

    def test_extract_palette_color_count(self):
        palette = extract_palette(str(Path(__file__).parent / "test.jpg"), color_count=3)
        assert len(palette) <= 3

    def test_nonexistent_file_palette(self):
        with pytest.raises(ValueError):
            extract_palette("does_not_exist.jpg")

    def test_nonexistent_file_color(self):
        with pytest.raises(ValueError):
            extract_dominant_color("does_not_exist.jpg")


# ---------------------------------------------------------------------------
# list_gpus tests
# ---------------------------------------------------------------------------

@pytest.mark.skipif(not gpu_available, reason="GPU module not available")
class TestGpuList:
    """Tests for list_gpus function."""

    def test_list_gpus_returns_list(self):
        gpus = list_gpus()
        assert isinstance(gpus, list)

    def test_list_gpus_gpu_entries(self):
        gpus = list_gpus()
        for gpu in gpus:
            assert "index" in gpu
            assert "name" in gpu
            assert "device_type" in gpu
            assert "vendor_name" in gpu
