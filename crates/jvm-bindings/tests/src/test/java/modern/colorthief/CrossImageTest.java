package modern.colorthief;

import io.baseplate_admin.modern_colorthief.Colorthief;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Cross-image comparison and input consistency tests:
 * - Different images produce different dominant colors
 * - Input consistency: separate pixel arrays with same data yield same results
 * - Color distance calculations between results
 */
public class CrossImageTest {

    @BeforeAll
    public static void loadNativeLibrary() {
        System.loadLibrary("modern_colorthief");
    }

    // =========================================================================
    // Different images produce different colors
    // =========================================================================

    @Test
    @DisplayName("Different pixel data produces different dominant colors")
    public void testDifferentImagesDifferentColors() {
        byte[] redPixels = createSolidPixels(50, 50, 255, 0, 0);
        byte[] bluePixels = createSolidPixels(50, 50, 0, 0, 255);

        byte[] c1 = Colorthief.getColor(redPixels, 50, 50, 1);
        byte[] c2 = Colorthief.getColor(bluePixels, 50, 50, 1);

        assertNotEquals(c1[0] & 0xFF, c2[0] & 0xFF, "Red image R channel must differ from blue image R channel");
        assertNotEquals(c1[2] & 0xFF, c2[2] & 0xFF, "Red image B channel must differ from blue image B channel");
    }

    @Test
    @DisplayName("Different gradient patterns produce different palettes")
    public void testDifferentGradientsDifferentPalettes() {
        byte[] hGrad = createHorizontalGradient(200, 100);
        byte[] vGrad = createVerticalGradient(100, 200);

        byte[][] p1 = Colorthief.getPalette(hGrad, 200, 100, 10, 5);
        byte[][] p2 = Colorthief.getPalette(vGrad, 100, 200, 10, 5);

        // Palettes should differ in at least one entry
        boolean differs = p1.length != p2.length;
        if (!differs) {
            for (int i = 0; i < p1.length; i++) {
                if (!java.util.Arrays.equals(p1[i], p2[i])) {
                    differs = true;
                    break;
                }
            }
        }
        assertTrue(differs, "Different gradient patterns should produce different palettes");
    }

    // =========================================================================
    // Input consistency: separate arrays with same data yield same results
    // =========================================================================

    @Test
    @DisplayName("Separate pixel arrays with same data yield same color")
    public void testConsistentAcrossSeparateArrays() {
        byte[] a = createSolidPixels(100, 100, 180, 100, 220);
        byte[] b = createSolidPixels(100, 100, 180, 100, 220);

        byte[] c1 = Colorthief.getColor(a, 100, 100, 1);
        byte[] c2 = Colorthief.getColor(b, 100, 100, 1);

        assertArrayEquals(c1, c2, "Same pixel data in different arrays must yield same color");
    }

    @Test
    @DisplayName("Separate pixel arrays with same data yield same palette")
    public void testConsistentPaletteAcrossSeparateArrays() {
        byte[] a = createSolidPixels(100, 100, 180, 100, 220);
        byte[] b = createSolidPixels(100, 100, 180, 100, 220);

        byte[][] p1 = Colorthief.getPalette(a, 100, 100, 5, 1);
        byte[][] p2 = Colorthief.getPalette(b, 100, 100, 5, 1);

        assertEquals(p1.length, p2.length);
        for (int i = 0; i < p1.length; i++) {
            assertArrayEquals(p1[i], p2[i], "Palette entry " + i);
        }
    }

    @Test
    @DisplayName("Color distance between identical inputs is zero")
    public void testColorDistanceZero() {
        byte[] pixels = createSolidPixels(200, 200, 128, 64, 192);
        byte[] c1 = Colorthief.getColor(pixels, 200, 200, 5);
        byte[] c2 = Colorthief.getColor(pixels, 200, 200, 5);

        double dist = colorDistance(c1, c2);
        assertEquals(0, dist, 0.001, "Identical inputs must have zero color distance");
    }

    @Test
    @DisplayName("Color distance between different inputs is non-zero")
    public void testColorDistanceNonZero() {
        byte[] red = createSolidPixels(100, 100, 255, 0, 0);
        byte[] cyan = createSolidPixels(100, 100, 0, 255, 255);

        byte[] c1 = Colorthief.getColor(red, 100, 100, 1);
        byte[] c2 = Colorthief.getColor(cyan, 100, 100, 1);

        double dist = colorDistance(c1, c2);
        assertTrue(dist > 100, "Red vs cyan should have large color distance, got " + dist);
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

    private byte[] createHorizontalGradient(int width, int height) {
        byte[] pixels = new byte[width * height * 4];
        for (int y = 0; y < height; y++) {
            for (int x = 0; x < width; x++) {
                int idx = (y * width + x) * 4;
                pixels[idx] = (byte) (x * 255 / (width - 1));
                pixels[idx + 1] = (byte) (y * 255 / (height - 1));
                pixels[idx + 2] = (byte) 128;
                pixels[idx + 3] = (byte) 255;
            }
        }
        return pixels;
    }

    private byte[] createVerticalGradient(int width, int height) {
        byte[] pixels = new byte[width * height * 4];
        for (int y = 0; y < height; y++) {
            for (int x = 0; x < width; x++) {
                int idx = (y * width + x) * 4;
                pixels[idx] = (byte) 255;
                pixels[idx + 1] = (byte) (y * 255 / (height - 1));
                pixels[idx + 2] = (byte) 0;
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
}
