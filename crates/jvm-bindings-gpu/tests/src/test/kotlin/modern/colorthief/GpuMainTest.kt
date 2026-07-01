package modern.colorthief

import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class GpuMainTest {

    companion object {
        init {
            try {
                System.loadLibrary("modern_colorthief_gpu")
            } catch (_: Throwable) {
            }
        }
    }

    // ---------------------------------------------------------------------------
    // Solid color detection
    // ---------------------------------------------------------------------------

    @Test
    fun gpuSolidRedDominantColor() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val color = ColorthiefGpu.getColor(pixels, 10, 10, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
        assertEquals(255, color[0].toInt() and 0xFF)
        assertEquals(0, color[1].toInt() and 0xFF)
        assertEquals(0, color[2].toInt() and 0xFF)
    }

    @Test
    fun gpuSolidGreenDominantColor() {
        val pixels = createSolidColorPixels(10, 10, 0.toByte(), 255.toByte(), 0.toByte())
        val color = ColorthiefGpu.getColor(pixels, 10, 10, 1)
        assertNotNull(color)
        assertEquals(0, color[0].toInt() and 0xFF)
        assertEquals(255, color[1].toInt() and 0xFF)
        assertEquals(0, color[2].toInt() and 0xFF)
    }

    @Test
    fun gpuSolidBlueDominantColor() {
        val pixels = createSolidColorPixels(10, 10, 0.toByte(), 0.toByte(), 255.toByte())
        val color = ColorthiefGpu.getColor(pixels, 10, 10, 1)
        assertNotNull(color)
        assertEquals(0, color[0].toInt() and 0xFF)
        assertEquals(0, color[1].toInt() and 0xFF)
        assertEquals(255, color[2].toInt() and 0xFF)
    }

    @Test
    fun gpuSolidPaletteReturnsOnlyThatColor() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = ColorthiefGpu.getPalette(pixels, 10, 10, 5, 1)
        assertNotNull(palette)
        assertTrue(palette.isNotEmpty())
        for (c in palette) {
            assertEquals(255, c[0].toInt() and 0xFF)
            assertEquals(0, c[1].toInt() and 0xFF)
            assertEquals(0, c[2].toInt() and 0xFF)
        }
    }

    // ---------------------------------------------------------------------------
    // Two-color detection
    // ---------------------------------------------------------------------------

    @Test
    fun gpuTwoColorsRedBlue() {
        val pixels = createTwoColorPixels(50, 50,
            255.toByte(), 0.toByte(), 0.toByte(),
            0.toByte(), 0.toByte(), 255.toByte())
        val palette = ColorthiefGpu.getPalette(pixels, 10, 10, 5, 1)
        assertTrue(paletteContains(palette, 255, 0, 0), "Should detect red")
        assertTrue(paletteContains(palette, 0, 0, 255), "Should detect blue")
    }

    // ---------------------------------------------------------------------------
    // Palette length respects color_count
    // ---------------------------------------------------------------------------

    @Test
    fun gpuPaletteCountBounded3() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = ColorthiefGpu.getPalette(pixels, 10, 10, 3, 1)
        assertTrue(palette.size <= 3, "Palette length must not exceed color_count")
    }

    @Test
    fun gpuPaletteReturnsNonEmptyResult() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = ColorthiefGpu.getPalette(pixels, 10, 10, 5, 1)
        assertNotNull(palette)
        assertTrue(palette.isNotEmpty(), "Palette should not be empty")
    }

    // ---------------------------------------------------------------------------
    // Deduplication
    // ---------------------------------------------------------------------------

    @Test
    fun gpuPaletteContainsNoDuplicateColors() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = ColorthiefGpu.getPalette(pixels, 10, 10, 255, 1)
        val uniqueCount = palette.map { it.contentToString() }.distinct().count()
        assertEquals(palette.size.toLong(), uniqueCount.toLong(), "Palette must contain no duplicates")
    }

    // ---------------------------------------------------------------------------
    // get_color and getPalette return valid RGB
    // ---------------------------------------------------------------------------

    @Test
    fun gpuColorReturnsValidRgbValues() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val color = ColorthiefGpu.getColor(pixels, 10, 10, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
        for (v in color) {
            val unsigned = v.toInt() and 0xFF
            assertTrue(unsigned in 0..255, "RGB value must be in [0, 255]")
        }
    }

    @Test
    fun gpuPaletteEntriesAreValidRgbValues() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val palette = ColorthiefGpu.getPalette(pixels, 10, 10, 5, 1)
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

    // ---------------------------------------------------------------------------
    // Deterministic results
    // ---------------------------------------------------------------------------

    @Test
    fun gpuDeterministicColor() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val c1 = ColorthiefGpu.getColor(pixels, 10, 10, 1)
        val c2 = ColorthiefGpu.getColor(pixels, 10, 10, 1)
        assertEquals(c1.contentHashCode(), c2.contentHashCode())
    }

    @Test
    fun gpuDeterministicPalette() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val p1 = ColorthiefGpu.getPalette(pixels, 10, 10, 5, 1)
        val p2 = ColorthiefGpu.getPalette(pixels, 10, 10, 5, 1)
        assertEquals(p1.size, p2.size)
        for (i in p1.indices) {
            assertEquals(p1[i].contentHashCode(), p2[i].contentHashCode())
        }
    }

    // ---------------------------------------------------------------------------
    // Edge cases: small images
    // ---------------------------------------------------------------------------

    @Test
    fun gpuSinglePixelReturnsThatColor() {
        val pixels = byteArrayOf(42.toByte(), 100.toByte(), 200.toByte(), 255.toByte())
        val color = ColorthiefGpu.getColor(pixels, 1, 1, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
        assertEquals(42, color[0].toInt() and 0xFF)
        assertEquals(100, color[1].toInt() and 0xFF)
        assertEquals(200, color[2].toInt() and 0xFF)
    }

    @Test
    fun gpuSmallImage2x2() {
        val pixels = createSolidColorPixels(2, 2, 255.toByte(), 128.toByte(), 0.toByte())
        val color = ColorthiefGpu.getColor(pixels, 2, 2, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
    }

    // ---------------------------------------------------------------------------
    // Non-square images
    // ---------------------------------------------------------------------------

    @Test
    fun gpuWideImage() {
        val pixels = createSolidColorPixels(20, 5, 255.toByte(), 0.toByte(), 0.toByte())
        val color = ColorthiefGpu.getColor(pixels, 20, 5, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
    }

    @Test
    fun gpuTallImage() {
        val pixels = createSolidColorPixels(5, 20, 255.toByte(), 0.toByte(), 0.toByte())
        val color = ColorthiefGpu.getColor(pixels, 5, 20, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
    }

    // ---------------------------------------------------------------------------
    // Gradient image
    // ---------------------------------------------------------------------------

    @Test
    fun gpuGradientReturnsMultipleColors() {
        val pixels = createGradientPixels(20, 10)
        val palette = ColorthiefGpu.getPalette(pixels, 20, 10, 10, 1)
        assertTrue(palette.size > 1, "Gradient should produce >1 color")
    }

    // ---------------------------------------------------------------------------
    // Checkerboard
    // ---------------------------------------------------------------------------

    @Test
    fun gpuCheckerboard() {
        val pixels = createCheckerboardPixels(10, 10)
        val palette = ColorthiefGpu.getPalette(pixels, 10, 10, 5, 1)
        assertTrue(palette.isNotEmpty())
    }

    // ---------------------------------------------------------------------------
    // Quality values
    // ---------------------------------------------------------------------------

    @Test
    fun gpuQualityMinimumWorks() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val color = ColorthiefGpu.getColor(pixels, 10, 10, 1)
        assertNotNull(color)
        assertEquals(3, color.size)
    }

    @Test
    fun gpuQualityMiddleWorks() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val color = ColorthiefGpu.getColor(pixels, 10, 10, 5)
        assertNotNull(color)
        assertEquals(3, color.size)
    }

    @Test
    fun gpuQualityMaximumWorks() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val color = ColorthiefGpu.getColor(pixels, 10, 10, 10)
        assertNotNull(color)
        assertEquals(3, color.size)
    }

    @Test
    fun gpuQualityZeroClamped() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val color = ColorthiefGpu.getColor(pixels, 10, 10, 0)
        assertNotNull(color)
        assertEquals(3, color.size)
    }

    @Test
    fun gpuQuality100Works() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val color = ColorthiefGpu.getColor(pixels, 10, 10, 100)
        assertNotNull(color)
        assertEquals(3, color.size)
    }

    // ---------------------------------------------------------------------------
    // Different images produce different results
    // ---------------------------------------------------------------------------

    @Test
    fun gpuDifferentImagesDifferentPalette() {
        val red = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val blue = createSolidColorPixels(10, 10, 0.toByte(), 0.toByte(), 255.toByte())
        val p1 = ColorthiefGpu.getPalette(red, 10, 10, 5, 1)
        val p2 = ColorthiefGpu.getPalette(blue, 10, 10, 5, 1)
        assertTrue(p1.size != p2.size || p1[0].contentHashCode() != p2[0].contentHashCode(), "Red and blue palettes should differ")
    }

    @Test
    fun gpuDifferentImagesDifferentColor() {
        val red = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        val green = createSolidColorPixels(10, 10, 0.toByte(), 255.toByte(), 0.toByte())
        val c1 = ColorthiefGpu.getColor(red, 10, 10, 1)
        val c2 = ColorthiefGpu.getColor(green, 10, 10, 1)
        assertTrue(c1.contentHashCode() != c2.contentHashCode(), "Red and green colors should differ")
    }

    // ---------------------------------------------------------------------------
    // Dominant color in palette
    // ---------------------------------------------------------------------------

    @Test
    fun gpuDominantInPalette() {
        val pixels = createTwoColorPixels(50, 50,
            255.toByte(), 0.toByte(), 0.toByte(),
            0.toByte(), 0.toByte(), 255.toByte())
        val color = ColorthiefGpu.getColor(pixels, 10, 10, 1)
        val palette = ColorthiefGpu.getPalette(pixels, 10, 10, 5, 1)
        assertTrue(palette.any { it.contentEquals(color) }, "Dominant color should be in palette")
    }

    // ---------------------------------------------------------------------------
    // Error handling
    // ---------------------------------------------------------------------------

    @Test
    fun gpuEmptyPixelsThrowsForPalette() {
        assertFailsWith<RuntimeException> {
            ColorthiefGpu.getPalette(ByteArray(0), 0, 0, 5, 1)
        }
    }

    @Test
    fun gpuEmptyPixelsThrowsForColor() {
        assertFailsWith<RuntimeException> {
            ColorthiefGpu.getColor(ByteArray(0), 0, 0, 1)
        }
    }

    @Test
    fun gpuZeroDimensionsThrowsForPalette() {
        assertFailsWith<RuntimeException> {
            ColorthiefGpu.getPalette(ByteArray(0), 0, 0, 5, 1)
        }
    }

    @Test
    fun gpuMismatchedPixelLengthThrows() {
        val shortPixels = ByteArray(100)
        assertFailsWith<RuntimeException> {
            ColorthiefGpu.getPalette(shortPixels, 10, 10, 5, 1)
        }
    }

    // ---------------------------------------------------------------------------
    // GC stress
    // ---------------------------------------------------------------------------

    @Test
    fun gpuGcStressPalette() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        repeat(50) {
            val palette = ColorthiefGpu.getPalette(pixels, 10, 10, 5, 1)
            assertTrue(palette.isNotEmpty())
        }
    }

    @Test
    fun gpuGcStressColor() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        repeat(50) {
            val color = ColorthiefGpu.getColor(pixels, 10, 10, 1)
            assertEquals(3, color.size)
        }
    }

    @Test
    fun gpuGcStressMixed() {
        val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
        repeat(25) {
            val palette = ColorthiefGpu.getPalette(pixels, 10, 10, 5, 1)
            val color = ColorthiefGpu.getColor(pixels, 10, 10, 1)
            assertTrue(palette.isNotEmpty())
            assertEquals(3, color.size)
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fun createSolidColorPixels(width: Int, height: Int, r: Byte, g: Byte, b: Byte): ByteArray {
    val pixelCount = width * height
    val pixels = ByteArray(pixelCount * 4)
    for (i in 0 until pixelCount) {
        pixels[i * 4] = r
        pixels[i * 4 + 1] = g
        pixels[i * 4 + 2] = b
        pixels[i * 4 + 3] = 255.toByte()
    }
    return pixels
}

fun createTwoColorPixels(
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

fun paletteContains(palette: Array<ByteArray>, r: Int, g: Int, b: Int): Boolean {
    for (c in palette) {
        if ((c[0].toInt() and 0xFF) == r && (c[1].toInt() and 0xFF) == g && (c[2].toInt() and 0xFF) == b) {
            return true
        }
    }
    return false
}

fun createGradientPixels(width: Int, height: Int): ByteArray {
    val pixels = ByteArray(width * height * 4)
    for (y in 0 until height) {
        for (x in 0 until width) {
            val idx = (y * width + x) * 4
            pixels[idx] = (x * 13).toByte()
            pixels[idx + 1] = (x * 7).toByte()
            pixels[idx + 2] = (x * 5).toByte()
            pixels[idx + 3] = 255.toByte()
        }
    }
    return pixels
}

fun createCheckerboardPixels(width: Int, height: Int): ByteArray {
    val pixels = ByteArray(width * height * 4)
    for (y in 0 until height) {
        for (x in 0 until width) {
            val idx = (y * width + x) * 4
            if ((x + y) % 2 == 0) {
                pixels[idx] = 200.toByte()
                pixels[idx + 1] = 50.toByte()
                pixels[idx + 2] = 50.toByte()
            } else {
                pixels[idx] = 50.toByte()
                pixels[idx + 1] = 50.toByte()
                pixels[idx + 2] = 200.toByte()
            }
            pixels[idx + 3] = 255.toByte()
        }
    }
    return pixels
}
