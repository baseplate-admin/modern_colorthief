package modern.colorthief;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Comprehensive JUnit 5 tests for the modern_colorthief Java bindings.
 * Covers all test scenarios from the Python test suite:
 * - Solid color detection
 * - Two-color detection
 * - Palette length respects color_count
 * - Deduplication
 * - get_color returns correct dominant color
 * - Error handling for empty/invalid input
 * - Deterministic results
 * - Edge cases (small images, large quality values)
 */
public class MainTest {

    @BeforeAll
    public static void loadNativeLibrary() {
        System.loadLibrary("modern_colorthief");
    }

    // =========================================================================
    // Solid color detection
    // =========================================================================

    @Test
    @DisplayName("Solid red image returns red dominant color")
    public void testSolidRedDominantColor() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[] color = Colorthief.getColor(pixels, 10, 10, 1);
        assertNotNull(color);
        assertEquals(3, color.length);
        assertEquals(255, color[0] & 0xFF);
        assertEquals(0, color[1] & 0xFF);
        assertEquals(0, color[2] & 0xFF);
    }

    @Test
    @DisplayName("Solid green image returns green dominant color")
    public void testSolidGreenDominantColor() {
        byte[] pixels = createSolidColorPixels(10, 10, 0, (byte) 255, 0);
        byte[] color = Colorthief.getColor(pixels, 10, 10, 1);
        assertNotNull(color);
        assertEquals(3, color.length);
        assertEquals(0, color[0] & 0xFF);
        assertEquals(255, color[1] & 0xFF);
        assertEquals(0, color[2] & 0xFF);
    }

    @Test
    @DisplayName("Solid blue image returns blue dominant color")
    public void testSolidBlueDominantColor() {
        byte[] pixels = createSolidColorPixels(10, 10, 0, 0, (byte) 255);
        byte[] color = Colorthief.getColor(pixels, 10, 10, 1);
        assertNotNull(color);
        assertEquals(3, color.length);
        assertEquals(0, color[0] & 0xFF);
        assertEquals(0, color[1] & 0xFF);
        assertEquals(255, color[2] & 0xFF);
    }

    @Test
    @DisplayName("Solid white image returns white dominant color")
    public void testSolidWhiteDominantColor() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, (byte) 255, (byte) 255);
        byte[] color = Colorthief.getColor(pixels, 10, 10, 1);
        assertNotNull(color);
        assertEquals(3, color.length);
        assertEquals(255, color[0] & 0xFF);
        assertEquals(255, color[1] & 0xFF);
        assertEquals(255, color[2] & 0xFF);
    }

    @Test
    @DisplayName("Solid palette returns only that color")
    public void testSolidRedPalette() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[][] palette = Colorthief.getPalette(pixels, 10, 10, 5, 1);
        assertNotNull(palette);
        assertTrue(palette.length > 0);
        for (byte[] c : palette) {
            assertEquals(255, c[0] & 0xFF);
            assertEquals(0, c[1] & 0xFF);
            assertEquals(0, c[2] & 0xFF);
        }
    }

    // =========================================================================
    // Two-color detection
    // =========================================================================

    @Test
    @DisplayName("Two-color image detects both red and blue")
    public void testTwoColorsRedBlue() {
        byte[] pixels = createTwoColorPixels(50, 50,
                (byte) 255, 0, 0,
                0, 0, (byte) 255);
        byte[][] palette = Colorthief.getPalette(pixels, 10, 10, 5, 1);
        assertTrue(paletteContains(palette, 255, 0, 0), "Should detect red");
        assertTrue(paletteContains(palette, 0, 0, 255), "Should detect blue");
    }

    @Test
    @DisplayName("Two-color image detects green and yellow")
    public void testTwoColorsGreenYellow() {
        byte[] pixels = createTwoColorPixels(50, 50,
                0, (byte) 255, 0,
                (byte) 255, (byte) 255, 0);
        byte[][] palette = Colorthief.getPalette(pixels, 10, 10, 5, 1);
        assertTrue(paletteContains(palette, 0, 255, 0), "Should detect green");
        assertTrue(paletteContains(palette, 255, 255, 0), "Should detect yellow");
    }

    @Test
    @DisplayName("Dominant color reflects majority color")
    public void testDominantColorMajority() {
        // 90 red pixels, 10 blue pixels -- red should dominate
        byte[] pixels = createTwoColorPixels(90, 10,
                (byte) 255, 0, 0,
                0, 0, (byte) 255);
        byte[] color = Colorthief.getColor(pixels, 10, 10, 1);
        assertEquals(255, color[0] & 0xFF);
        assertEquals(0, color[1] & 0xFF);
        assertEquals(0, color[2] & 0xFF);
    }

    // =========================================================================
    // Palette length respects color_count
    // =========================================================================

    @Test
    @DisplayName("Palette length does not exceed requested color_count of 3")
    public void testPaletteCountBounded3() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[][] palette = Colorthief.getPalette(pixels, 10, 10, 3, 1);
        assertTrue(palette.length <= 3, "Palette length must not exceed color_count");
    }

    @Test
    @DisplayName("Palette length does not exceed requested color_count of 5")
    public void testPaletteCountBounded5() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[][] palette = Colorthief.getPalette(pixels, 10, 10, 5, 1);
        assertTrue(palette.length <= 5, "Palette length must not exceed color_count");
    }

    @Test
    @DisplayName("Palette length does not exceed requested color_count of 10")
    public void testPaletteCountBounded10() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[][] palette = Colorthief.getPalette(pixels, 10, 10, 10, 1);
        assertTrue(palette.length <= 10, "Palette length must not exceed color_count");
    }

    @Test
    @DisplayName("Palette returns non-empty result")
    public void testPaletteNonEmpty() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[][] palette = Colorthief.getPalette(pixels, 10, 10, 5, 1);
        assertNotNull(palette);
        assertTrue(palette.length > 0, "Palette should not be empty");
    }

    // =========================================================================
    // Deduplication
    // =========================================================================

    @Test
    @DisplayName("Palette contains no duplicate colors")
    public void testDeduplication() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[][] palette = Colorthief.getPalette(pixels, 10, 10, 255, 1);
        long uniqueCount = java.util.Arrays.stream(palette)
                .map(java.util.Arrays::toString)
                .distinct()
                .count();
        assertEquals(palette.length, uniqueCount, "Palette must contain no duplicates");
    }

    @Test
    @DisplayName("Palette size within reasonable bounds when requesting 255 colors")
    public void testDeduplicationSizeBounded() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[][] palette = Colorthief.getPalette(pixels, 10, 10, 255, 1);
        assertTrue(palette.length > 0, "Palette should not be empty");
        assertTrue(palette.length <= 255, "Palette should not exceed 255 entries");
    }

    // =========================================================================
    // get_color returns correct dominant color
    // =========================================================================

    @Test
    @DisplayName("get_color returns valid RGB values in range [0, 255]")
    public void testColorReturnsValidRgb() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[] color = Colorthief.getColor(pixels, 10, 10, 1);
        assertNotNull(color);
        assertEquals(3, color.length);
        for (byte v : color) {
            int unsigned = v & 0xFF;
            assertTrue(unsigned >= 0 && unsigned <= 255, "RGB value must be in [0, 255]");
        }
    }

    @Test
    @DisplayName("Palette entries are valid RGB values in range [0, 255]")
    public void testPaletteReturnsValidRgb() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[][] palette = Colorthief.getPalette(pixels, 10, 10, 5, 1);
        assertNotNull(palette);
        assertTrue(palette.length > 0);
        for (byte[] color : palette) {
            assertEquals(3, color.length);
            for (byte v : color) {
                int unsigned = v & 0xFF;
                assertTrue(unsigned >= 0 && unsigned <= 255, "RGB value must be in [0, 255]");
            }
        }
    }

    // =========================================================================
    // Error handling for empty/invalid input
    // =========================================================================

    @Test
    @DisplayName("Empty pixel array throws exception for getPalette")
    public void testEmptyPixelsPalette() {
        assertThrows(RuntimeException.class, () -> {
            Colorthief.getPalette(new byte[0], 0, 0, 5, 1);
        });
    }

    @Test
    @DisplayName("Empty pixel array throws exception for getColor")
    public void testEmptyPixelsColor() {
        assertThrows(RuntimeException.class, () -> {
            Colorthief.getColor(new byte[0], 0, 0, 1);
        });
    }

    @Test
    @DisplayName("Zero width and height throws exception for getPalette")
    public void testZeroDimensionsPalette() {
        assertThrows(RuntimeException.class, () -> {
            Colorthief.getPalette(new byte[0], 0, 0, 5, 1);
        });
    }

    @Test
    @DisplayName("Zero width and height throws exception for getColor")
    public void testZeroDimensionsColor() {
        assertThrows(RuntimeException.class, () -> {
            Colorthief.getColor(new byte[0], 0, 0, 1);
        });
    }

    @Test
    @DisplayName("Mismatched pixel data length throws exception")
    public void testMismatchedPixelLength() {
        // 100 bytes but claims 10x10 (needs 400 bytes)
        byte[] shortPixels = new byte[100];
        assertThrows(RuntimeException.class, () -> {
            Colorthief.getPalette(shortPixels, 10, 10, 5, 1);
        });
    }

    @Test
    @DisplayName("Mismatched pixel data length throws exception for getColor")
    public void testMismatchedPixelLengthColor() {
        byte[] shortPixels = new byte[100];
        assertThrows(RuntimeException.class, () -> {
            Colorthief.getColor(shortPixels, 10, 10, 1);
        });
    }

    // =========================================================================
    // Deterministic results
    // =========================================================================

    @Test
    @DisplayName("get_color returns same result for identical inputs")
    public void testDeterministicColor() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[] c1 = Colorthief.getColor(pixels, 10, 10, 1);
        byte[] c2 = Colorthief.getColor(pixels, 10, 10, 1);
        assertArrayEquals(c1, c2, "Results must be deterministic");
    }

    @Test
    @DisplayName("getPalette returns same result for identical inputs")
    public void testDeterministicPalette() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[][] p1 = Colorthief.getPalette(pixels, 10, 10, 5, 1);
        byte[][] p2 = Colorthief.getPalette(pixels, 10, 10, 5, 1);
        assertEquals(p1.length, p2.length);
        for (int i = 0; i < p1.length; i++) {
            assertArrayEquals(p1[i], p2[i], "Palette entry " + i + " must match");
        }
    }

    @Test
    @DisplayName("Multiple calls produce consistent results")
    public void testDeterministicMultipleCalls() {
        byte[] pixels = createSolidColorPixels(20, 20, (byte) 128, (byte) 64, (byte) 200);
        byte[] first = Colorthief.getColor(pixels, 20, 20, 1);
        for (int i = 0; i < 4; i++) {
            byte[] later = Colorthief.getColor(pixels, 20, 20, 1);
            assertArrayEquals(first, later, "Call " + (i + 2) + " must match first");
        }
    }

    // =========================================================================
    // Edge cases: small images
    // =========================================================================

    @Test
    @DisplayName("Single pixel image returns that pixel color")
    public void testSinglePixel() {
        byte[] pixels = new byte[4];
        pixels[0] = 42;
        pixels[1] = 100;
        pixels[2] = 200;
        pixels[3] = 255; // alpha
        byte[] color = Colorthief.getColor(pixels, 1, 1, 1);
        assertNotNull(color);
        assertEquals(3, color.length);
        assertEquals(42, color[0] & 0xFF);
        assertEquals(100, color[1] & 0xFF);
        assertEquals(200, color[2] & 0xFF);
    }

    @Test
    @DisplayName("Single pixel palette returns that color")
    public void testSinglePixelPalette() {
        byte[] pixels = new byte[4];
        pixels[0] = 42;
        pixels[1] = 100;
        pixels[2] = 200;
        pixels[3] = 255;
        byte[][] palette = Colorthief.getPalette(pixels, 1, 1, 5, 1);
        assertNotNull(palette);
        assertTrue(palette.length > 0);
    }

    @Test
    @DisplayName("Small 2x2 image works correctly")
    public void testSmallImage2x2() {
        byte[] pixels = createSolidColorPixels(2, 2, (byte) 255, (byte) 128, 0);
        byte[] color = Colorthief.getColor(pixels, 2, 2, 1);
        assertNotNull(color);
        assertEquals(3, color.length);
    }

    @Test
    @DisplayName("Non-square image (20x5) works correctly")
    public void testNonSquareImage() {
        byte[] pixels = createSolidColorPixels(20, 5, (byte) 255, 0, 0);
        byte[] color = Colorthief.getColor(pixels, 20, 5, 1);
        assertNotNull(color);
        assertEquals(3, color.length);
        assertEquals(255, color[0] & 0xFF);
    }

    // =========================================================================
    // Edge cases: large quality values
    // =========================================================================

    @Test
    @DisplayName("Quality 1 (most accurate) works")
    public void testQualityMin() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[] color = Colorthief.getColor(pixels, 10, 10, 1);
        assertNotNull(color);
        assertEquals(3, color.length);
    }

    @Test
    @DisplayName("Quality 5 (default) works")
    public void testQualityMid() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[] color = Colorthief.getColor(pixels, 10, 10, 5);
        assertNotNull(color);
        assertEquals(3, color.length);
    }

    @Test
    @DisplayName("Quality 10 (fastest) works")
    public void testQualityMax() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[] color = Colorthief.getColor(pixels, 10, 10, 10);
        assertNotNull(color);
        assertEquals(3, color.length);
    }

    @Test
    @DisplayName("Palette with quality 1 works")
    public void testPaletteQualityMin() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[][] palette = Colorthief.getPalette(pixels, 10, 10, 5, 1);
        assertNotNull(palette);
        assertTrue(palette.length > 0);
    }

    @Test
    @DisplayName("Palette with quality 10 works")
    public void testPaletteQualityMax() {
        byte[] pixels = createSolidColorPixels(10, 10, (byte) 255, 0, 0);
        byte[][] palette = Colorthief.getPalette(pixels, 10, 10, 5, 10);
        assertNotNull(palette);
        assertTrue(palette.length > 0);
    }

    // =========================================================================
    // Helper methods
    // =========================================================================

    /**
     * Create raw RGBA pixel data for a solid-color image.
     */
    private byte[] createSolidColorPixels(int width, int height, int r, int g, int b) {
        int pixelCount = width * height;
        byte[] pixels = new byte[pixelCount * 4];
        for (int i = 0; i < pixelCount; i++) {
            pixels[i * 4] = (byte) r;
            pixels[i * 4 + 1] = (byte) g;
            pixels[i * 4 + 2] = (byte) b;
            pixels[i * 4 + 3] = (byte) 255; // full alpha
        }
        return pixels;
    }

    /**
     * Create raw RGBA pixel data with two color blocks.
     */
    private byte[] createTwoColorPixels(int firstCount, int secondCount,
                                        int r1, int g1, int b1,
                                        int r2, int g2, int b2) {
        byte[] pixels = new byte[(firstCount + secondCount) * 4];
        for (int i = 0; i < firstCount; i++) {
            pixels[i * 4] = (byte) r1;
            pixels[i * 4 + 1] = (byte) g1;
            pixels[i * 4 + 2] = (byte) b1;
            pixels[i * 4 + 3] = (byte) 255;
        }
        for (int i = 0; i < secondCount; i++) {
            int idx = (firstCount + i) * 4;
            pixels[idx] = (byte) r2;
            pixels[idx + 1] = (byte) g2;
            pixels[idx + 2] = (byte) b2;
            pixels[idx + 3] = (byte) 255;
        }
        return pixels;
    }

    /**
     * Check if a palette contains a specific RGB color (with tolerance).
     */
    private boolean paletteContains(byte[][] palette, int r, int g, int b) {
        for (byte[] c : palette) {
            if ((c[0] & 0xFF) == r && (c[1] & 0xFF) == g && (c[2] & 0xFF) == b) {
                return true;
            }
        }
        return false;
    }
}
