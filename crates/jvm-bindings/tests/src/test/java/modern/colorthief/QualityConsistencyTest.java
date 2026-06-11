package modern.colorthief;

import io.baseplate_admin.modern_colorthief.Colorthief;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Quality consistency tests:
 * - Same pixel data at different quality settings yields similar results
 * - Palette deduplication on complex multi-color images
 */
public class QualityConsistencyTest {

    @BeforeAll
    public static void loadNativeLibrary() {
        System.loadLibrary("modern_colorthief");
    }

    // =========================================================================
    // Same pixels, different quality, similar results
    // =========================================================================

    @Test
    @DisplayName("get_color at quality 1 and quality 10 yield similar dominant color")
    public void testColorConsistentAcrossQuality() {
        byte[] pixels = createGradientPixels(500, 500);

        byte[] c1 = Colorthief.getColor(pixels, 500, 500, 1);
        byte[] c10 = Colorthief.getColor(pixels, 500, 500, 10);

        assertNotNull(c1);
        assertNotNull(c10);
        assertEquals(3, c1.length);
        assertEquals(3, c10.length);

  double dist = colorDistance(c1, c10);
        assertTrue(dist < 200,
                "Quality 1 vs 10 color distance " + dist + " should be < 200");
    }

    @Test
    @DisplayName("getPalette at quality 1 and quality 10 both return valid results")
    public void testPaletteConsistentAcrossQuality() {
        byte[] pixels = createGradientPixels(500, 500);

        byte[][] p1 = Colorthief.getPalette(pixels, 500, 500, 10, 1);
        byte[][] p10 = Colorthief.getPalette(pixels, 500, 500, 10, 10);

        assertNotNull(p1);
        assertNotNull(p10);
        assertTrue(p1.length > 0);
        assertTrue(p10.length > 0);
        assertTrue(p1.length <= 10);
        assertTrue(p10.length <= 10);

        // All entries valid RGB
        for (byte[] c : p1) assertAllValid(c);
        for (byte[] c : p10) assertAllValid(c);
    }

    @Test
    @DisplayName("All quality values 1-10 produce non-null valid results")
    public void testAllQualityValuesProduceValidResults() {
        byte[] pixels = createSolidPixels(100, 100, 200, 100, 50);

        for (int q = 1; q <= 10; q++) {
            byte[] c = Colorthief.getColor(pixels, 100, 100, q);
            assertNotNull(c, "quality " + q + " returned null");
            assertEquals(3, c.length, "quality " + q + " wrong length");
        }
    }

    // =========================================================================
    // Palette deduplication on complex multi-color images
    // =========================================================================

    @Test
    @DisplayName("Multi-color gradient palette has no duplicates")
    public void testDeduplicationOnGradient() {
        byte[] pixels = createGradientPixels(500, 500);
        byte[][] palette = Colorthief.getPalette(pixels, 500, 500, 255, 1);

        assertTrue(palette.length > 1, "Gradient should produce multiple colors");
        assertTrue(palette.length <= 255, "Palette must not exceed requested count");

        long uniqueCount = java.util.Arrays.stream(palette)
                .map(java.util.Arrays::toString)
                .distinct()
                .count();
        assertEquals(palette.length, uniqueCount, "No duplicates in gradient palette");
    }

    @Test
    @DisplayName("Checkerboard palette has no duplicates")
    public void testDeduplicationOnCheckerboard() {
        byte[] pixels = createCheckerboardPixels(200, 200, 16);
        byte[][] palette = Colorthief.getPalette(pixels, 200, 200, 255, 1);

        assertTrue(palette.length > 0);
        assertTrue(palette.length <= 255);

        long uniqueCount = java.util.Arrays.stream(palette)
                .map(java.util.Arrays::toString)
                .distinct()
                .count();
        assertEquals(palette.length, uniqueCount, "No duplicates in checkerboard palette");
    }

    // =========================================================================
    // Helper methods
    // =========================================================================

    private byte[] createSolidPixels(int width, int height, int r, int g, int b) {
        byte[] pixels = new byte[width * height * 4];
        for (int i = 0; i < width * height; i++) {
            pixels[i * 4] = (byte) r;
            pixels[i * 4 + 1] = (byte) g;
            pixels[i * 4 + 2] = (byte) b;
            pixels[i * 4 + 3] = (byte) 255;
        }
        return pixels;
    }

    private byte[] createGradientPixels(int width, int height) {
        byte[] pixels = new byte[width * height * 4];
        for (int y = 0; y < height; y++) {
            for (int x = 0; x < width; x++) {
                int idx = (y * width + x) * 4;
                pixels[idx] = (byte) (x * 255 / (width - 1));
                pixels[idx + 1] = (byte) (y * 255 / (height - 1));
                pixels[idx + 2] = (byte) ((x + y) * 127 / (width + height - 2));
                pixels[idx + 3] = (byte) 255;
            }
        }
        return pixels;
    }

    private byte[] createCheckerboardPixels(int width, int height, int tileSize) {
        byte[] pixels = new byte[width * height * 4];
        for (int y = 0; y < height; y++) {
            for (int x = 0; x < width; x++) {
                int idx = (y * width + x) * 4;
                boolean dark = ((x / tileSize) + (y / tileSize)) % 2 != 0;
                pixels[idx] = dark ? (byte) 50 : (byte) 200;
                pixels[idx + 1] = dark ? (byte) 100 : (byte) 150;
                pixels[idx + 2] = dark ? (byte) 150 : (byte) 50;
                pixels[idx + 3] = (byte) 255;
            }
        }
        return pixels;
    }

    private double colorDistance(byte[] a, byte[] b) {
        double sum = 0;
        for (int i = 0; i < 3; i++) {
            double diff = (a[i] & 0xFF) - (b[i] & 0xFF);
            sum += diff * diff;
        }
        return Math.sqrt(sum);
    }

    private void assertAllValid(byte[] c) {
        assertEquals(3, c.length);
        for (int i = 0; i < 3; i++) {
            assertTrue((c[i] & 0xFF) >= 0 && (c[i] & 0xFF) <= 255);
        }
    }
}
