package modern.colorthief

import io.baseplate_admin.modern_colorthief.Colorthief
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import java.util.Arrays
import kotlin.test.assertEquals
import kotlin.test.assertContentEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

/**
 * Comprehensive JUnit 5 tests for the modern_colorthief Java bindings.
 * Covers core functionality:
 * - Solid color detection
 * - Two-color detection
 * - Palette length respects color_count
 * - Deduplication
 * - get_color returns correct dominant color
 * - Error handling for empty/invalid input
 * - Deterministic results
 * - Edge cases (small images, large quality values)
 */
class MainTest {

    companion object {
        @BeforeAll
        @JvmStatic
        fun loadNativeLibrary() {
            System.loadLibrary("modern_colorthief")
        }
    }

    // =========================================================================
    // Solid color detection
    // =========================================================================

    @Test
    @DisplayName("Solid red image returns red dominant color")
    fun testSolidRedDominantColor() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val color = Colorthief.getColor(pixels, 10, 10, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
        assertEquals(255, color[0].toInt() and 0xFF)
        assertEquals(0, color[1].toInt() and 0xFF)
        assertEquals(0, color[2].toInt() and 0xFF)
    }

    @Test
    @DisplayName("Solid green image returns green dominant color")
    fun testSolidGreenDominantColor() {
        val pixels = createSolidColorPixels(10, 10, 0.toByte(), 255.toByte(), 0.toByte())
        val color = Colorthief.getColor(pixels, 10, 10, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
        assertEquals(0, color[0].toInt() and 0xFF)
        assertEquals(255, color[1].toInt() and 0xFF)
        assertEquals(0, color[2].toInt() and 0xFF)
    }

    @Test
    @DisplayName("Solid blue image returns blue dominant color")
    fun testSolidBlueDominantColor() {
        val pixels = createSolidColorPixels(10, 10, 0.toByte(), 0.toByte(), 255.toByte())
        val color = Colorthief.getColor(pixels, 10, 10, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
        assertEquals(0, color[0].toInt() and 0xFF)
        assertEquals(0, color[1].toInt() and 0xFF)
        assertEquals(255, color[2].toInt() and 0xFF)
    }

    @Test
    @DisplayName("Solid white image returns white dominant color")
    fun testSolidWhiteDominantColor() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 255.toByte(), 255.toByte())
        val color = Colorthief.getColor(pixels, 10, 10, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
        assertEquals(255, color[0].toInt() and 0xFF)
        assertEquals(255, color[1].toInt() and 0xFF)
        assertEquals(255, color[2].toInt() and 0xFF)
    }

    @Test
    @DisplayName("Solid palette returns only that color")
    fun testSolidRedPalette() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = Colorthief.getPalette(pixels, 10, 10, 5, 1)
        assertNotNull(palette)
        assertTrue(palette.isNotEmpty())
        for (c in palette) {
            assertEquals(255, c[0].toInt() and 0xFF)
            assertEquals(0, c[1].toInt() and 0xFF)
            assertEquals(0, c[2].toInt() and 0xFF)
        }
    }

    // =========================================================================
    // Two-color detection
    // =========================================================================

    @Test
    @DisplayName("Two-color image detects both red and blue")
    fun testTwoColorsRedBlue() {
        val pixels = createTwoColorPixels(50, 50,
            255.toByte(), 0.toByte(), 0.toByte(),
            0.toByte(), 0.toByte(), 255.toByte())
        val palette = Colorthief.getPalette(pixels, 10, 10, 5, 1)
        assertTrue(paletteContains(palette, 255, 0, 0), "Should detect red")
        assertTrue(paletteContains(palette, 0, 0, 255), "Should detect blue")
    }

    @Test
    @DisplayName("Two-color image detects green and yellow")
    fun testTwoColorsGreenYellow() {
        val pixels = createTwoColorPixels(50, 50,
            0.toByte(), 255.toByte(), 0.toByte(),
            255.toByte(), 255.toByte(), 0.toByte())
        val palette = Colorthief.getPalette(pixels, 10, 10, 5, 1)
        assertTrue(paletteContains(palette, 0, 255, 0), "Should detect green")
        assertTrue(paletteContains(palette, 255, 255, 0), "Should detect yellow")
    }

    @Test
    @DisplayName("Dominant color reflects majority color")
    fun testDominantColorMajority() {
        // 90 red pixels, 10 blue pixels -- red should dominate
        val pixels = createTwoColorPixels(90, 10,
            255.toByte(), 0.toByte(), 0.toByte(),
            0.toByte(), 0.toByte(), 255.toByte())
        val color = Colorthief.getColor(pixels, 10, 10, 1)
        assertEquals(255, color[0].toInt() and 0xFF)
        assertEquals(0, color[1].toInt() and 0xFF)
        assertEquals(0, color[2].toInt() and 0xFF)
    }

    // =========================================================================
    // Palette length respects color_count
    // =========================================================================

    @Test
    @DisplayName("Palette length does not exceed requested color_count of 3")
    fun testPaletteCountBounded3() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = Colorthief.getPalette(pixels, 10, 10, 3, 1)
        assertTrue(palette.size <= 3, "Palette length must not exceed color_count")
    }

    @Test
    @DisplayName("Palette length does not exceed requested color_count of 5")
    fun testPaletteCountBounded5() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = Colorthief.getPalette(pixels, 10, 10, 5, 1)
        assertTrue(palette.size <= 5, "Palette length must not exceed color_count")
    }

    @Test
    @DisplayName("Palette length does not exceed requested color_count of 10")
    fun testPaletteCountBounded10() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = Colorthief.getPalette(pixels, 10, 10, 10, 1)
        assertTrue(palette.size <= 10, "Palette length must not exceed color_count")
    }

    @Test
    @DisplayName("Palette returns non-empty result")
    fun testPaletteNonEmpty() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = Colorthief.getPalette(pixels, 10, 10, 5, 1)
        assertNotNull(palette)
        assertTrue(palette.isNotEmpty(), "Palette should not be empty")
    }

    // =========================================================================
    // Deduplication
    // =========================================================================

    @Test
    @DisplayName("Palette contains no duplicate colors")
    fun testDeduplication() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = Colorthief.getPalette(pixels, 10, 10, 255, 1)
        val uniqueCount = palette.map { Arrays.toString(it) }.distinct().count()
        assertEquals(palette.size.toLong(), uniqueCount.toLong(), "Palette must contain no duplicates")
    }

    @Test
    @DisplayName("Palette size within reasonable bounds when requesting 255 colors")
    fun testDeduplicationSizeBounded() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = Colorthief.getPalette(pixels, 10, 10, 255, 1)
        assertTrue(palette.isNotEmpty(), "Palette should not be empty")
        assertTrue(palette.size <= 255, "Palette should not exceed 255 entries")
    }

    // =========================================================================
    // get_color returns correct dominant color
    // =========================================================================

    @Test
    @DisplayName("get_color returns valid RGB values in range [0, 255]")
    fun testColorReturnsValidRgb() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val color = Colorthief.getColor(pixels, 10, 10, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
        for (v in color) {
            val unsigned = v.toInt() and 0xFF
            assertTrue(unsigned in 0..255, "RGB value must be in [0, 255]")
        }
    }

    @Test
    @DisplayName("Palette entries are valid RGB values in range [0, 255]")
    fun testPaletteReturnsValidRgb() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = Colorthief.getPalette(pixels, 10, 10, 5, 1)
        assertNotNull(palette)
        assertTrue(palette.isNotEmpty())
        for (color in palette) {
            assertEquals(3, color.size)
            for (v in color) {
                val unsigned = v.toInt() and 0xFF
                assertTrue(unsigned in 0..255, "RGB value must be in [0, 255]")
            }
        }
    }

    // =========================================================================
    // Error handling for empty/invalid input
    // =========================================================================

    @Test
    @DisplayName("Empty pixel array throws exception for getPalette")
    fun testEmptyPixelsPalette() {
        val ex = assertFailsWith<RuntimeException> {
            Colorthief.getPalette(ByteArray(0), 0, 0, 5, 1)
        }
        assertNotNull(ex)
    }

    @Test
    @DisplayName("Empty pixel array throws exception for getColor")
    fun testEmptyPixelsColor() {
        val ex = assertFailsWith<RuntimeException> {
            Colorthief.getColor(ByteArray(0), 0, 0, 1)
        }
        assertNotNull(ex)
    }

    @Test
    @DisplayName("Zero width and height throws exception for getPalette")
    fun testZeroDimensionsPalette() {
        val ex = assertFailsWith<RuntimeException> {
            Colorthief.getPalette(ByteArray(0), 0, 0, 5, 1)
        }
        assertNotNull(ex)
    }

    @Test
    @DisplayName("Zero width and height throws exception for getColor")
    fun testZeroDimensionsColor() {
        val ex = assertFailsWith<RuntimeException> {
            Colorthief.getColor(ByteArray(0), 0, 0, 1)
        }
        assertNotNull(ex)
    }

    @Test
    @DisplayName("Mismatched pixel data length throws exception")
    fun testMismatchedPixelLength() {
        // 100 bytes but claims 10x10 (needs 400 bytes)
        val shortPixels = ByteArray(100)
        val ex = assertFailsWith<RuntimeException> {
            Colorthief.getPalette(shortPixels, 10, 10, 5, 1)
        }
        assertNotNull(ex)
    }

    @Test
    @DisplayName("Mismatched pixel data length throws exception for getColor")
    fun testMismatchedPixelLengthColor() {
        val shortPixels = ByteArray(100)
        val ex = assertFailsWith<RuntimeException> {
            Colorthief.getColor(shortPixels, 10, 10, 1)
        }
        assertNotNull(ex)
    }

    // =========================================================================
    // Deterministic results
    // =========================================================================

    @Test
    @DisplayName("get_color returns same result for identical inputs")
    fun testDeterministicColor() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val c1 = Colorthief.getColor(pixels, 10, 10, 1)
        val c2 = Colorthief.getColor(pixels, 10, 10, 1)
        assertContentEquals(c1, c2)
    }

    @Test
    @DisplayName("getPalette returns same result for identical inputs")
    fun testDeterministicPalette() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val p1 = Colorthief.getPalette(pixels, 10, 10, 5, 1)
        val p2 = Colorthief.getPalette(pixels, 10, 10, 5, 1)
        assertEquals(p1.size, p2.size)
        for (i in p1.indices) {
            assertContentEquals(p1[i], p2[i])
        }
    }

    @Test
    @DisplayName("Multiple calls produce consistent results")
    fun testDeterministicMultipleCalls() {
        val pixels = createSolidColorPixels(20, 20, 128.toByte(), 64.toByte(), 200.toByte())
        val first = Colorthief.getColor(pixels, 20, 20, 1)
        for (i in 0..3) {
            val later = Colorthief.getColor(pixels, 20, 20, 1)
            assertContentEquals(first, later)
        }
    }

    // =========================================================================
    // Edge cases: small images
    // =========================================================================

    @Test
    @DisplayName("Single pixel image returns that pixel color")
    fun testSinglePixel() {
        val pixels = byteArrayOf(42.toByte(), 100.toByte(), 200.toByte(), 255.toByte())
        val color = Colorthief.getColor(pixels, 1, 1, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
        assertEquals(42, color[0].toInt() and 0xFF)
        assertEquals(100, color[1].toInt() and 0xFF)
        assertEquals(200, color[2].toInt() and 0xFF)
    }

    @Test
    @DisplayName("Single pixel palette returns that color")
    fun testSinglePixelPalette() {
        val pixels = byteArrayOf(42.toByte(), 100.toByte(), 200.toByte(), 255.toByte())
        val palette = Colorthief.getPalette(pixels, 1, 1, 5, 1)
        assertNotNull(palette)
        assertTrue(palette.isNotEmpty())
    }

    @Test
    @DisplayName("Small 2x2 image works correctly")
    fun testSmallImage2x2() {
        val pixels = createSolidColorPixels(2, 2, 255.toByte(), 128.toByte(), 0.toByte())
        val color = Colorthief.getColor(pixels, 2, 2, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
    }

    @Test
    @DisplayName("Non-square image (20x5) works correctly")
    fun testNonSquareImage() {
        val pixels = createSolidColorPixels(20, 5, 255.toByte(), 0.toByte(), 0.toByte())
        val color = Colorthief.getColor(pixels, 20, 5, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
        assertEquals(255, color[0].toInt() and 0xFF)
    }

    // =========================================================================
    // Edge cases: large quality values
    // =========================================================================

    @Test
    @DisplayName("Quality 1 (most accurate) works")
    fun testQualityMin() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val color = Colorthief.getColor(pixels, 10, 10, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
    }

    @Test
    @DisplayName("Quality 5 (default) works")
    fun testQualityMid() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val color = Colorthief.getColor(pixels, 10, 10, 5)
        assertNotNull(color)
        assertEquals(3, color.size)
    }

    @Test
    @DisplayName("Quality 10 (fastest) works")
    fun testQualityMax() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val color = Colorthief.getColor(pixels, 10, 10, 10)
        assertNotNull(color)
        assertEquals(3, color.size)
    }

    @Test
    @DisplayName("Palette with quality 1 works")
    fun testPaletteQualityMin() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = Colorthief.getPalette(pixels, 10, 10, 5, 1)
        assertNotNull(palette)
        assertTrue(palette.isNotEmpty())
    }

    @Test
    @DisplayName("Palette with quality 10 works")
    fun testPaletteQualityMax() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = Colorthief.getPalette(pixels, 10, 10, 5, 10)
        assertNotNull(palette)
        assertTrue(palette.isNotEmpty())
    }

    // =========================================================================
    // Helper methods
    // =========================================================================

    /**
     * Create raw RGBA pixel data for a solid-color image.
     */
    private fun createSolidColorPixels(width: Int, height: Int, r: Byte, g: Byte, b: Byte): ByteArray {
        val pixelCount = width * height
        val pixels = ByteArray(pixelCount * 4)
        for (i in 0 until pixelCount) {
            pixels[i * 4] = r
            pixels[i * 4 + 1] = g
            pixels[i * 4 + 2] = b
            pixels[i * 4 + 3] = 255.toByte() // full alpha
        }
        return pixels
    }

    /**
     * Create raw RGBA pixel data with two color blocks.
     */
    private fun createTwoColorPixels(
        firstCount: Int, secondCount: Int,
        r1: Byte, g1: Byte, b1: Byte,
        r2: Byte, g2: Byte, b2: Byte
    ): ByteArray {
        val pixels = ByteArray((firstCount + secondCount) * 4)
        for (i in 0 until firstCount) {
            pixels[i * 4] = r1
            pixels[i * 4 + 1] = g1
            pixels[i * 4 + 2] = b1
            pixels[i * 4 + 3] = 255.toByte()
        }
        for (i in 0 until secondCount) {
            val idx = (firstCount + i) * 4
            pixels[idx] = r2
            pixels[idx + 1] = g2
            pixels[idx + 2] = b2
            pixels[idx + 3] = 255.toByte()
        }
        return pixels
    }

    /**
     * Check if a palette contains a specific RGB color (with tolerance).
     */
    private fun paletteContains(palette: Array<ByteArray>, r: Int, g: Int, b: Int): Boolean {
        for (c in palette) {
            if ((c[0].toInt() and 0xFF) == r && (c[1].toInt() and 0xFF) == g && (c[2].toInt() and 0xFF) == b) {
                return true
            }
        }
        return false
    }
}
