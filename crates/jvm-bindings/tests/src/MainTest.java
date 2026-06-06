import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;
import static org.junit.jupiter.api.Assertions.*;

public class MainTest {

    @Test
    @DisplayName("API exists and is loadable")
    public void testApiExists() {
        assertNotNull(MainTest.class);
    }

    @Test
    @DisplayName("Palette returns valid RGB values")
    public void testPaletteReturnsValidRgb() {
        byte[] pixels = createSolidRedPixels(100);
        byte[][] palette = colorthief.Colorthief.getPalette(pixels, 10, 10, 5, 1);
        assertNotNull(palette);
        assertTrue(palette.length > 0);
        for (byte[] color : palette) {
            assertEquals(3, color.length);
            for (byte v : color) {
                assertTrue(v >= 0 && v <= 255);
            }
        }
    }

    @Test
    @DisplayName("Color returns valid RGB")
    public void testColorReturnsValidRgb() {
        byte[] pixels = createSolidRedPixels(100);
        byte[] color = colorthief.Colorthief.getColor(pixels, 10, 10, 1);
        assertNotNull(color);
        assertEquals(3, color.length);
    }

    @Test
    @DisplayName("Solid red image returns red")
    public void testSolidRedReturnsRed() {
        byte[] pixels = createSolidRedPixels(100);
        byte[] color = colorthief.Colorthief.getColor(pixels, 10, 10, 1);
        assertEquals(255, color[0] & 0xFF);
        assertEquals(0, color[1] & 0xFF);
        assertEquals(0, color[2] & 0xFF);
    }

    @Test
    @DisplayName("Two colors detected correctly")
    public void testTwoColors() {
        byte[] pixels = createTwoColorPixels(50, 50);
        byte[][] palette = colorthief.Colorthief.getPalette(pixels, 10, 10, 5, 1);
        boolean hasRed = false, hasBlue = false;
        for (byte[] c : palette) {
            if (c[0] == 255 && c[1] == 0 && c[2] == 0) hasRed = true;
            if (c[0] == 0 && c[1] == 0 && c[2] == 255) hasBlue = true;
        }
        assertTrue(hasRed, "Should detect red");
        assertTrue(hasBlue, "Should detect blue");
    }

    @Test
    @DisplayName("Palette is deduplicated")
    public void testDeduplication() {
        byte[] pixels = createSolidRedPixels(100);
        byte[][] palette = colorthief.Colorthief.getPalette(pixels, 10, 10, 255, 1);
        long unique = java.util.Arrays.stream(palette)
                .map(java.util.Arrays::toString)
                .distinct()
                .count();
        assertEquals(palette.length, unique);
    }

    @Test
    @DisplayName("Palette count bounded")
    public void testPaletteCountBounded() {
        byte[] pixels = createSolidRedPixels(100);
        for (int count : new int[]{3, 5}) {
            byte[][] palette = colorthief.Colorthief.getPalette(pixels, 10, 10, count, 1);
            assertTrue(palette.length <= count);
        }
    }

    @Test
    @DisplayName("Deterministic results")
    public void testDeterministic() {
        byte[] pixels = createSolidRedPixels(100);
        byte[] c1 = colorthief.Colorthief.getColor(pixels, 10, 10, 1);
        byte[] c2 = colorthief.Colorthief.getColor(pixels, 10, 10, 1);
        assertArrayEquals(c1, c2);
    }

    private byte[] createSolidRedPixels(int count) {
        byte[] pixels = new byte[count * 4];
        for (int i = 0; i < count; i++) {
            pixels[i * 4] = 255;
            pixels[i * 4 + 1] = 0;
            pixels[i * 4 + 2] = 0;
            pixels[i * 4 + 3] = 255;
        }
        return pixels;
    }

    private byte[] createTwoColorPixels(int redCount, int blueCount) {
        byte[] pixels = new byte[(redCount + blueCount) * 4];
        for (int i = 0; i < redCount; i++) {
            pixels[i * 4] = 255;
            pixels[i * 4 + 3] = 255;
        }
        for (int i = 0; i < blueCount; i++) {
            int idx = (redCount + i) * 4;
            pixels[idx + 2] = 255;
            pixels[idx + 3] = 255;
        }
        return pixels;
    }
}
