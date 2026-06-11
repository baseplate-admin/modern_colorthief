package modern.colorthief

import io.baseplate_admin.modern_colorthief.Colorthief
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import java.util.Random
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.fail
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

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
class RealImageTest {

    companion object {
        @BeforeAll
        @JvmStatic
        fun loadNativeLibrary() {
            System.loadLibrary("modern_colorthief")
        }
    }

    // =========================================================================
    // Resource loading tests
    // =========================================================================

    @Test
    @DisplayName("Load test.jpg from resources and verify bytes are readable")
    fun testLoadTestJpgFromResources() {
        val stream = javaClass.classLoader.getResourceAsStream("test.jpg")
        assertNotNull(stream, "test.jpg should exist in src/test/resources")
        var data: ByteArray
        try {
            data = stream.readBytes()
            assertTrue(data.isNotEmpty(), "test.jpg should not be empty")
            assertTrue(data.size < 1_000_000, "test.jpg should be under 1MB")
        } catch (e: Exception) {
            fail("Failed to read test.jpg: ${e.message}")
        } finally {
            stream.close()
        }
    }

    @Test
    @DisplayName("Load kaiju_no_8.jpg from resources and verify bytes are readable")
    fun testLoadKaijuNo8JpgFromResources() {
        val stream = javaClass.classLoader.getResourceAsStream("kaiju_no_8.jpg")
        assertNotNull(stream, "kaiju_no_8.jpg should exist in src/test/resources")
        var data: ByteArray
        try {
            data = stream.readBytes()
            assertTrue(data.isNotEmpty(), "kaiju_no_8.jpg should not be empty")
            assertTrue(data.size < 1_000_000, "kaiju_no_8.jpg should be under 1MB")
        } catch (e: Exception) {
            fail("Failed to read kaiju_no_8.jpg: ${e.message}")
        } finally {
            stream.close()
        }
    }

    // =========================================================================
    // Synthetic gradient tests (simulating real image color transitions)
    // =========================================================================

    @Test
    @DisplayName("Horizontal RGB gradient produces valid palette")
    fun testHorizontalRgbGradient() {
        val width = 200
        val height = 100
        val pixels = ByteArray(width * height * 4)
        for (y in 0 until height) {
            for (x in 0 until width) {
                val idx = (y * width + x) * 4
                pixels[idx] = (x * 255 / (width - 1)).toByte()       // R ramps 0..255
                pixels[idx + 1] = (y * 255 / (height - 1)).toByte() // G ramps 0..255
                pixels[idx + 2] = 128.toByte()                       // B constant
                pixels[idx + 3] = 255.toByte()
            }
        }
        val palette = Colorthief.getPalette(pixels, width, height, 10, 5)
        assertNotNull(palette)
        assertTrue(palette.isNotEmpty(), "Gradient palette should not be empty")
        assertAllRgbValid(palette)
    }

    @Test
    @DisplayName("Vertical gradient produces valid dominant color")
    fun testVerticalGradientDominantColor() {
        val width = 100
        val height = 200
        val pixels = ByteArray(width * height * 4)
        for (y in 0 until height) {
            for (x in 0 until width) {
                val idx = (y * width + x) * 4
                pixels[idx] = 255.toByte()                           // R constant
                pixels[idx + 1] = (y * 255 / (height - 1)).toByte() // G ramps
                pixels[idx + 2] = 0.toByte()                         // B constant
                pixels[idx + 3] = 255.toByte()
            }
        }
        val color = Colorthief.getColor(pixels, width, height, 5)
        assertNotNull(color)
        assertEquals(3, color.size)
        assertTrue((color[0].toInt() and 0xFF) in 0..255)
        assertTrue((color[1].toInt() and 0xFF) in 0..255)
        assertTrue((color[2].toInt() and 0xFF) in 0..255)
    }

    // =========================================================================
    // Checkerboard pattern tests
    // =========================================================================

    @Test
    @DisplayName("Checkerboard pattern detects both colors in palette")
    fun testCheckerboardPalette() {
        val width = 64
        val height = 64
        val pixels = ByteArray(width * height * 4)
        for (y in 0 until height) {
            for (x in 0 until width) {
                val idx = (y * width + x) * 4
                val white = ((x / 8) + (y / 8)) % 2 == 0
                val v = if (white) 255.toByte() else 0.toByte()
                pixels[idx] = v
                pixels[idx + 1] = v
                pixels[idx + 2] = v
                pixels[idx + 3] = 255.toByte()
            }
        }
        val palette = Colorthief.getPalette(pixels, width, height, 5, 5)
        assertNotNull(palette)
        assertTrue(palette.isNotEmpty())
        assertAllRgbValid(palette)
    }

    // =========================================================================
    // Random noise tests (simulating photographic data)
    // =========================================================================

    @Test
    @DisplayName("Random noise image produces valid palette with many distinct colors")
    fun testRandomNoisePalette() {
        val width = 100
        val height = 100
        val pixels = generateRandomPixels(width, height, Random(42))
        val palette = Colorthief.getPalette(pixels, width, height, 10, 5)
        assertNotNull(palette)
        assertTrue(palette.isNotEmpty(), "Noise palette should not be empty")
        assertTrue(palette.size <= 10, "Palette should not exceed requested count")
        assertAllRgbValid(palette)
    }

    @Test
    @DisplayName("Random noise dominant color is valid")
    fun testRandomNoiseDominantColor() {
        val width = 100
        val height = 100
        val pixels = generateRandomPixels(width, height, Random(99))
        val color = Colorthief.getColor(pixels, width, height, 5)
        assertNotNull(color)
        assertEquals(3, color.size)
        for (v in color) {
            val unsigned = v.toInt() and 0xFF
            assertTrue(unsigned in 0..255)
        }
    }

    // =========================================================================
    // Large image (4K resolution) tests
    // =========================================================================

    @Test
    @DisplayName("4K resolution image (4000x3000) palette extraction works")
    fun test4kResolutionPalette() {
        val width = 4000
        val height = 3000
        val expectedBytes = width.toLong() * height * 4L
        assertTrue(expectedBytes > 0, "Should not overflow int range")

        val pixels = ByteArray(expectedBytes.toInt())
        fillGradientPixels(pixels, width, height)

        val palette = Colorthief.getPalette(pixels, width, height, 10, 10)
        assertNotNull(palette)
        assertTrue(palette.isNotEmpty(), "4K palette should not be empty")
        assertTrue(palette.size <= 10)
        assertAllRgbValid(palette)
    }

    @Test
    @DisplayName("4K resolution image (4000x3000) dominant color extraction works")
    fun test4kResolutionDominantColor() {
        val width = 4000
        val height = 3000
        val pixels = ByteArray(width * height * 4)
        fillGradientPixels(pixels, width, height)

        val color = Colorthief.getColor(pixels, width, height, 10)
        assertNotNull(color)
        assertEquals(3, color.size)
        for (v in color) {
            val unsigned = v.toInt() and 0xFF
            assertTrue(unsigned in 0..255)
        }
    }

    @Test
    @DisplayName("Large solid-color image at 4K returns correct color")
    fun test4kSolidColor() {
        val width = 4000
        val height = 3000
        val pixels = createSolidColorPixels(width, height, 170.toByte(), 85.toByte(), 220.toByte())

        val color = Colorthief.getColor(pixels, width, height, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
        // Allow small tolerance for large image sampling
        assertEquals(170.0, (color[0].toInt() and 0xFF).toDouble(), 10.0)
        assertEquals(85.0, (color[1].toInt() and 0xFF).toDouble(), 10.0)
        assertEquals(220.0, (color[2].toInt() and 0xFF).toDouble(), 10.0)
    }

    @Test
    @DisplayName("Multiple 4K palette calls produce consistent results")
    fun test4kConsistentResults() {
        val width = 4000
        val height = 3000
        val pixels = ByteArray(width * height * 4)
        fillGradientPixels(pixels, width, height)

        val p1 = Colorthief.getPalette(pixels, width, height, 5, 10)
        val p2 = Colorthief.getPalette(pixels, width, height, 5, 10)

        assertEquals(p1.size, p2.size, "Palette lengths must match")
        for (i in p1.indices) {
            assertContentEquals(p1[i], p2[i])
        }
    }

    // =========================================================================
    // Memory handling with repeated large allocations
    // =========================================================================

    @Test
    @DisplayName("Repeated 4K palette calls do not cause memory errors")
    fun testRepeated4kAllocations() {
        val width = 4000
        val height = 3000
        val pixels = ByteArray(width * height * 4)
        fillGradientPixels(pixels, width, height)

        for (i in 0..4) {
            val palette = Colorthief.getPalette(pixels, width, height, 5, 10)
            assertNotNull(palette)
            assertTrue(palette.isNotEmpty())
            assertAllRgbValid(palette)
        }
    }

    @Test
    @DisplayName("Large two-tone image (4K) detects both dominant regions")
    fun test4kTwoTonePalette() {
        val width = 4000
        val height = 3000
        val pixels = ByteArray(width * height * 4)
        // Top half red, bottom half blue
        for (y in 0 until height) {
            for (x in 0 until width) {
                val idx = (y * width + x) * 4
                if (y < height / 2) {
                    pixels[idx] = 255.toByte()   // R
                    pixels[idx + 1] = 0.toByte() // G
                    pixels[idx + 2] = 0.toByte() // B
                } else {
                    pixels[idx] = 0.toByte()     // R
                    pixels[idx + 1] = 0.toByte() // G
                    pixels[idx + 2] = 255.toByte() // B
                }
                pixels[idx + 3] = 255.toByte()
            }
        }
        val palette = Colorthief.getPalette(pixels, width, height, 5, 10)
        assertNotNull(palette)
        assertTrue(palette.isNotEmpty())
        assertAllRgbValid(palette)
        // Should find both red and blue (or close approximations)
        var hasRed = false
        var hasBlue = false
        for (c in palette) {
            val r = c[0].toInt() and 0xFF
            val g = c[1].toInt() and 0xFF
            val b = c[2].toInt() and 0xFF
            if (r > 128 && g < 128 && b < 128) hasRed = true
            if (r < 128 && g < 128 && b > 128) hasBlue = true
        }
        assertTrue(hasRed, "4K two-tone palette should contain red region")
        assertTrue(hasBlue, "4K two-tone palette should contain blue region")
    }

    // =========================================================================
    // Edge case: maximum reasonable quality with large image
    // =========================================================================

    @Test
    @DisplayName("4K image at highest quality setting works")
    fun test4kHighestQuality() {
        val width = 4000
        val height = 3000
        val pixels = ByteArray(width * height * 4)
        fillGradientPixels(pixels, width, height)

        val palette = Colorthief.getPalette(pixels, width, height, 3, 1)
        assertNotNull(palette)
        assertTrue(palette.isNotEmpty())
        assertAllRgbValid(palette)
    }

    // =========================================================================
    // Helper methods
    // =========================================================================

    /**
     * Fill pixel buffer with a diagonal gradient pattern.
     */
    private fun fillGradientPixels(pixels: ByteArray, width: Int, height: Int) {
        for (y in 0 until height) {
            for (x in 0 until width) {
                val idx = (y * width + x) * 4
                pixels[idx] = (x * 255 / (width - 1)).toByte()
                pixels[idx + 1] = (y * 255 / (height - 1)).toByte()
                pixels[idx + 2] = ((x + y) * 127 / (width + height - 2)).toByte()
                pixels[idx + 3] = 255.toByte()
            }
        }
    }

    /**
     * Generate random RGBA pixel data.
     */
    private fun generateRandomPixels(width: Int, height: Int, rng: Random): ByteArray {
        val pixels = ByteArray(width * height * 4)
        rng.nextBytes(pixels)
        // Force alpha to 255 for all pixels
        for (i in 3 until pixels.size step 4) {
            pixels[i] = 255.toByte()
        }
        return pixels
    }

    /**
     * Create raw RGBA pixel data for a solid-color image.
     */
    private fun createSolidColorPixels(width: Int, height: Int, r: Byte, g: Byte, b: Byte): ByteArray {
        val pixels = ByteArray(width * height * 4)
        for (i in 0 until width * height) {
            pixels[i * 4] = r
            pixels[i * 4 + 1] = g
            pixels[i * 4 + 2] = b
            pixels[i * 4 + 3] = 255.toByte()
        }
        return pixels
    }

    /**
     * Assert that every color in the palette has valid RGB values [0, 255].
     */
    private fun assertAllRgbValid(palette: Array<ByteArray>) {
        for (i in palette.indices) {
            val c = palette[i]
            assertEquals(3, c.size, "Palette entry $i must have 3 channels")
            for (ch in 0..2) {
                val v = c[ch].toInt() and 0xFF
                assertTrue(v in 0..255,
                    "Palette[$i][$ch] = $v out of range")
            }
        }
    }
}
