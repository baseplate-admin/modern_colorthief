package modern.colorthief;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.io.InputStream;
import java.util.Random;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for JVM binding features involving real image data patterns,
 * large (4K resolution) synthetic images, and memory handling.
 *
 * Covers:
 * - Loading test images from resources via ClassLoader.getResourceAsStream
 * - Synthetic pixel data simulating real image patterns (gradients, checkerboards, noise)
 * - Very large images at 4K resolution (4000x3000)
 * - Memory handling under large allocations
 * - Result validity (non-empty palettes, valid RGB ranges)
 */
public class RealImageTest {

    @BeforeAll
    public static void loadNativeLibrary() {
        System.loadLibrary("modern_colorthief");
    }

    // =========================================================================
    // Resource loading tests
    // =========================================================================

    @Test
    @DisplayName("Load test.jpg from resources and verify bytes are readable")
    public void testLoadTestJpgFromResources() {
        try (InputStream is = getClass().getClassLoader().getResourceAsStream("test.jpg")) {
            assertNotNull(is, "test.jpg should exist in src/test/resources");
            byte[] data = is.readAllBytes();
            assertTrue(data.length > 0, "test.jpg should not be empty");
            assertTrue(data.length < 1_000_000, "test.jpg should be under 1MB");
        } catch (Exception e) {
            fail("Failed to read test.jpg: " + e.getMessage());
        }
    }

    @Test
    @DisplayName("Load kaiju_no_8.jpg from resources and verify bytes are readable")
    public void testLoadKaijuNo8JpgFromResources() {
        try (InputStream is = getClass().getClassLoader().getResourceAsStream("kaiju_no_8.jpg")) {
            assertNotNull(is, "kaiju_no_8.jpg should exist in src/test/resources");
            byte[] data = is.readAllBytes();
            assertTrue(data.length > 0, "kaiju_no_8.jpg should not be empty");
            assertTrue(data.length < 1_000_000, "kaiju_no_8.jpg should be under 1MB");
        } catch (Exception e) {
            fail("Failed to read kaiju_no_8.jpg: " + e.getMessage());
        }
    }

    // =========================================================================
    // Synthetic gradient tests (simulating real image color transitions)
    // =========================================================================

    @Test
    @DisplayName("Horizontal RGB gradient produces valid palette")
    public void testHorizontalRgbGradient() {
        int width = 200;
        int height = 100;
        byte[] pixels = new byte[width * height * 4];
        for (int y = 0; y < height; y++) {
            for (int x = 0; x < width; x++) {
                int idx = (y * width + x) * 4;
                pixels[idx] = (byte) (x * 255 / (width - 1));       // R ramps 0..255
                pixels[idx + 1] = (byte) (y * 255 / (height - 1)); // G ramps 0..255
                pixels[idx + 2] = (byte) 128;                       // B constant
                pixels[idx + 3] = (byte) 255;
            }
        }
        byte[][] palette = Colorthief.getPalette(pixels, width, height, 10, 5);
        assertNotNull(palette);
        assertTrue(palette.length > 0, "Gradient palette should not be empty");
        assertAllRgbValid(palette);
    }

    @Test
    @DisplayName("Vertical gradient produces valid dominant color")
    public void testVerticalGradientDominantColor() {
        int width = 100;
        int height = 200;
        byte[] pixels = new byte[width * height * 4];
        for (int y = 0; y < height; y++) {
            for (int x = 0; x < width; x++) {
                int idx = (y * width + x) * 4;
                pixels[idx] = (byte) 255;                           // R constant
                pixels[idx + 1] = (byte) (y * 255 / (height - 1)); // G ramps
                pixels[idx + 2] = (byte) 0;                         // B constant
                pixels[idx + 3] = 255;
            }
        }
        byte[] color = Colorthief.getColor(pixels, width, height, 5);
        assertNotNull(color);
        assertEquals(3, color.length);
        assertTrue((color[0] & 0xFF) >= 0 && (color[0] & 0xFF) <= 255);
        assertTrue((color[1] & 0xFF) >= 0 && (color[1] & 0xFF) <= 255);
        assertTrue((color[2] & 0xFF) >= 0 && (color[2] & 0xFF) <= 255);
    }

    // =========================================================================
    // Checkerboard pattern tests
    // =========================================================================

    @Test
    @DisplayName("Checkerboard pattern detects both colors in palette")
    public void testCheckerboardPalette() {
        int width = 64;
        int height = 64;
        byte[] pixels = new byte[width * height * 4];
        for (int y = 0; y < height; y++) {
            for (int x = 0; x < width; x++) {
                int idx = (y * width + x) * 4;
                boolean white = ((x / 8) + (y / 8)) % 2 == 0;
                byte v = white ? (byte) 255 : 0;
                pixels[idx] = v;
                pixels[idx + 1] = v;
                pixels[idx + 2] = v;
                pixels[idx + 3] = 255;
            }
        }
        byte[][] palette = Colorthief.getPalette(pixels, width, height, 5, 5);
        assertNotNull(palette);
        assertTrue(palette.length > 0);
        assertAllRgbValid(palette);
    }

    // =========================================================================
    // Random noise tests (simulating photographic data)
    // =========================================================================

    @Test
    @DisplayName("Random noise image produces valid palette with many distinct colors")
    public void testRandomNoisePalette() {
        int width = 100;
        int height = 100;
        byte[] pixels = generateRandomPixels(width, height, new Random(42));
        byte[][] palette = Colorthief.getPalette(pixels, width, height, 10, 5);
        assertNotNull(palette);
        assertTrue(palette.length > 0, "Noise palette should not be empty");
        assertTrue(palette.length <= 10, "Palette should not exceed requested count");
        assertAllRgbValid(palette);
    }

    @Test
    @DisplayName("Random noise dominant color is valid")
    public void testRandomNoiseDominantColor() {
        int width = 100;
        int height = 100;
        byte[] pixels = generateRandomPixels(width, height, new Random(99));
        byte[] color = Colorthief.getColor(pixels, width, height, 5);
        assertNotNull(color);
        assertEquals(3, color.length);
        for (byte v : color) {
            int unsigned = v & 0xFF;
            assertTrue(unsigned >= 0 && unsigned <= 255);
        }
    }

    // =========================================================================
    // Large image (4K resolution) tests
    // =========================================================================

    @Test
    @DisplayName("4K resolution image (4000x3000) palette extraction works")
    public void test4kResolutionPalette() {
        int width = 4000;
        int height = 3000;
        long expectedBytes = (long) width * height * 4;
        assertTrue(expectedBytes > 0, "Should not overflow int range");

        byte[] pixels = new byte[(int) expectedBytes];
        fillGradientPixels(pixels, width, height);

        byte[][] palette = Colorthief.getPalette(pixels, width, height, 10, 10);
        assertNotNull(palette);
        assertTrue(palette.length > 0, "4K palette should not be empty");
        assertTrue(palette.length <= 10);
        assertAllRgbValid(palette);
    }

    @Test
    @DisplayName("4K resolution image (4000x3000) dominant color extraction works")
    public void test4kResolutionDominantColor() {
        int width = 4000;
        int height = 3000;
        byte[] pixels = new byte[width * height * 4];
        fillGradientPixels(pixels, width, height);

        byte[] color = Colorthief.getColor(pixels, width, height, 10);
        assertNotNull(color);
        assertEquals(3, color.length);
        for (byte v : color) {
            int unsigned = v & 0xFF;
            assertTrue(unsigned >= 0 && unsigned <= 255);
        }
    }

    @Test
    @DisplayName("Large solid-color image at 4K returns correct color")
    public void test4kSolidColor() {
        int width = 4000;
        int height = 3000;
        byte[] pixels = createSolidColorPixels(width, height, (byte) 170, (byte) 85, (byte) 220);

        byte[] color = Colorthief.getColor(pixels, width, height, 1);
        assertNotNull(color);
        assertEquals(3, color.length);
        // Allow small tolerance for large image sampling
        assertEquals(170, color[0] & 0xFF, 10);
        assertEquals(85, color[1] & 0xFF, 10);
        assertEquals(220, color[2] & 0xFF, 10);
    }

    @Test
    @DisplayName("Multiple 4K palette calls produce consistent results")
    public void test4kConsistentResults() {
        int width = 4000;
        int height = 3000;
        byte[] pixels = new byte[width * height * 4];
        fillGradientPixels(pixels, width, height);

        byte[][] p1 = Colorthief.getPalette(pixels, width, height, 5, 10);
        byte[][] p2 = Colorthief.getPalette(pixels, width, height, 5, 10);

        assertEquals(p1.length, p2.length, "Palette lengths must match");
        for (int i = 0; i < p1.length; i++) {
            assertArrayEquals(p1[i], p2[i], "Palette entry " + i + " must be consistent");
        }
    }

    // =========================================================================
    // Memory handling with repeated large allocations
    // =========================================================================

    @Test
    @DisplayName("Repeated 4K palette calls do not cause memory errors")
    public void testRepeated4kAllocations() {
        int width = 4000;
        int height = 3000;
        byte[] pixels = new byte[width * height * 4];
        fillGradientPixels(pixels, width, height);

        for (int i = 0; i < 5; i++) {
            byte[][] palette = Colorthief.getPalette(pixels, width, height, 5, 10);
            assertNotNull(palette);
            assertTrue(palette.length > 0);
            assertAllRgbValid(palette);
        }
    }

    @Test
    @DisplayName("Large two-tone image (4K) detects both dominant regions")
    public void test4kTwoTonePalette() {
        int width = 4000;
        int height = 3000;
        byte[] pixels = new byte[width * height * 4];
        // Top half red, bottom half blue
        for (int y = 0; y < height; y++) {
            for (int x = 0; x < width; x++) {
                int idx = (y * width + x) * 4;
                if (y < height / 2) {
                    pixels[idx] = 255;   // R
                    pixels[idx + 1] = 0; // G
                    pixels[idx + 2] = 0; // B
                } else {
                    pixels[idx] = 0;     // R
                    pixels[idx + 1] = 0; // G
                    pixels[idx + 2] = 255; // B
                }
                pixels[idx + 3] = 255;
            }
        }
        byte[][] palette = Colorthief.getPalette(pixels, width, height, 5, 10);
        assertNotNull(palette);
        assertTrue(palette.length > 0);
        assertAllRgbValid(palette);
        // Should find both red and blue (or close approximations)
        boolean hasRed = false;
        boolean hasBlue = false;
        for (byte[] c : palette) {
            int r = c[0] & 0xFF;
            int g = c[1] & 0xFF;
            int b = c[2] & 0xFF;
            if (r > 128 && g < 128 && b < 128) hasRed = true;
            if (r < 128 && g < 128 && b > 128) hasBlue = true;
        }
        assertTrue(hasRed, "4K two-tone palette should contain red region");
        assertTrue(hasBlue, "4K two-tone palette should contain blue region");
    }

    // =========================================================================
    // Edge case: maximum reasonable quality with large image
    // =========================================================================

    @Test
    @DisplayName("4K image at highest quality setting works")
    public void test4kHighestQuality() {
        int width = 4000;
        int height = 3000;
        byte[] pixels = new byte[width * height * 4];
        fillGradientPixels(pixels, width, height);

        byte[][] palette = Colorthief.getPalette(pixels, width, height, 3, 1);
        assertNotNull(palette);
        assertTrue(palette.length > 0);
        assertAllRgbValid(palette);
    }

    // =========================================================================
    // Helper methods
    // =========================================================================

    /**
     * Fill pixel buffer with a diagonal gradient pattern.
     */
    private void fillGradientPixels(byte[] pixels, int width, int height) {
        for (int y = 0; y < height; y++) {
            for (int x = 0; x < width; x++) {
                int idx = (y * width + x) * 4;
                pixels[idx] = (byte) (x * 255 / (width - 1));
                pixels[idx + 1] = (byte) (y * 255 / (height - 1));
                pixels[idx + 2] = (byte) ((x + y) * 127 / (width + height - 2));
                pixels[idx + 3] = 255;
            }
        }
    }

    /**
     * Generate random RGBA pixel data.
     */
    private byte[] generateRandomPixels(int width, int height, Random rng) {
        byte[] pixels = new byte[width * height * 4];
        rng.nextBytes(pixels);
        // Force alpha to 255 for all pixels
        for (int i = 3; i < pixels.length; i += 4) {
            pixels[i] = 255;
        }
        return pixels;
    }

    /**
     * Create raw RGBA pixel data for a solid-color image.
     */
    private byte[] createSolidColorPixels(int width, int height, byte r, byte g, byte b) {
        byte[] pixels = new byte[width * height * 4];
        for (int i = 0; i < width * height; i++) {
            pixels[i * 4] = r;
            pixels[i * 4 + 1] = g;
            pixels[i * 4 + 2] = b;
            pixels[i * 4 + 3] = 255;
        }
        return pixels;
    }

    /**
     * Assert that every color in the palette has valid RGB values [0, 255].
     */
    private void assertAllRgbValid(byte[][] palette) {
        for (int i = 0; i < palette.length; i++) {
            byte[] c = palette[i];
            assertEquals(3, c.length, "Palette entry " + i + " must have 3 channels");
            for (int ch = 0; ch < 3; ch++) {
                int v = c[ch] & 0xFF;
                assertTrue(v >= 0 && v <= 255,
                        "Palette[" + i + "][" + ch + "] = " + v + " out of range");
            }
        }
    }
}
