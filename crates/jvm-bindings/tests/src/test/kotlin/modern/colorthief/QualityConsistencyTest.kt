package modern.colorthief

import io.baseplate_admin.modern_colorthief.Colorthief
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

/**
 * Quality consistency tests:
 * - Same pixel data at different quality settings yields similar results
 * - Palette deduplication on complex multi-color images
 */
class QualityConsistencyTest {

    companion object {
        init {
            val libPath = System.getProperty("native.lib.path")
            if (libPath != null) {
                System.load(libPath + "/libmodern_colorthief.so")
            } else {
                System.loadLibrary("modern_colorthief")
            }
        }
    }

    // =========================================================================
    // Same pixels, different quality, similar results
    // =========================================================================

    @Test
    @DisplayName("get_color at quality 1 and quality 10 yield similar dominant color")
    fun testColorConsistentAcrossQuality() {
        val pixels = createGradientPixels(500, 500)

        val c1 = Colorthief.getColor(pixels, 500, 500, 1)
        val c10 = Colorthief.getColor(pixels, 500, 500, 10)

        assertNotNull(c1)
        assertNotNull(c10)
        assertEquals(3, c1.size)
        assertEquals(3, c10.size)

        val dist = colorDistance(c1, c10)
        assertTrue(dist < 200,
            "Quality 1 vs 10 color distance $dist should be < 200")
    }

    @Test
    @DisplayName("getPalette at quality 1 and quality 10 both return valid results")
    fun testPaletteConsistentAcrossQuality() {
        val pixels = createGradientPixels(500, 500)

        val p1 = Colorthief.getPalette(pixels, 500, 500, 10, 1)
        val p10 = Colorthief.getPalette(pixels, 500, 500, 10, 10)

        assertNotNull(p1)
        assertNotNull(p10)
        assertTrue(p1.isNotEmpty())
        assertTrue(p10.isNotEmpty())
        assertTrue(p1.size <= 10)
        assertTrue(p10.size <= 10)

        // All entries valid RGB
        for (c in p1) assertAllValid(c)
        for (c in p10) assertAllValid(c)
    }

    @Test
    @DisplayName("All quality values 1-10 produce non-null valid results")
    fun testAllQualityValuesProduceValidResults() {
        val pixels = createSolidPixels(100, 100, 200, 100, 50)

        for (q in 1..10) {
            val c = Colorthief.getColor(pixels, 100, 100, q)
            assertNotNull(c)
            assertEquals(3, c.size)
        }
    }

    // =========================================================================
    // Palette deduplication on complex multi-color images
    // =========================================================================

    @Test
    @DisplayName("Multi-color gradient palette has no duplicates")
    fun testDeduplicationOnGradient() {
        val pixels = createGradientPixels(500, 500)
        val palette = Colorthief.getPalette(pixels, 500, 500, 255, 1)

        assertTrue(palette.size > 1, "Gradient should produce multiple colors")
        assertTrue(palette.size <= 255, "Palette must not exceed requested count")

        val uniqueCount = palette.map { it.contentToString() }.distinct().count()
        assertEquals(palette.size.toLong(), uniqueCount.toLong(), "No duplicates in gradient palette")
    }

    @Test
    @DisplayName("Checkerboard palette has no duplicates")
    fun testDeduplicationOnCheckerboard() {
        val pixels = createCheckerboardPixels(200, 200, 16)
        val palette = Colorthief.getPalette(pixels, 200, 200, 255, 1)

        assertTrue(palette.isNotEmpty())
        assertTrue(palette.size <= 255)

        val uniqueCount = palette.map { it.contentToString() }.distinct().count()
        assertEquals(palette.size.toLong(), uniqueCount.toLong(), "No duplicates in checkerboard palette")
    }

    // =========================================================================
    // Helper methods
    // =========================================================================

    private fun createSolidPixels(width: Int, height: Int, r: Int, g: Int, b: Int): ByteArray {
        val pixels = ByteArray(width * height * 4)
        for (i in 0 until width * height) {
            pixels[i * 4] = r.toByte()
            pixels[i * 4 + 1] = g.toByte()
            pixels[i * 4 + 2] = b.toByte()
            pixels[i * 4 + 3] = 255.toByte()
        }
        return pixels
    }

    private fun createGradientPixels(width: Int, height: Int): ByteArray {
        val pixels = ByteArray(width * height * 4)
        for (y in 0 until height) {
            for (x in 0 until width) {
                val idx = (y * width + x) * 4
                pixels[idx] = (x * 255 / (width - 1)).toByte()
                pixels[idx + 1] = (y * 255 / (height - 1)).toByte()
                pixels[idx + 2] = ((x + y) * 127 / (width + height - 2)).toByte()
                pixels[idx + 3] = 255.toByte()
            }
        }
        return pixels
    }

    private fun createCheckerboardPixels(width: Int, height: Int, tileSize: Int): ByteArray {
        val pixels = ByteArray(width * height * 4)
        for (y in 0 until height) {
            for (x in 0 until width) {
                val idx = (y * width + x) * 4
                val dark = ((x / tileSize) + (y / tileSize)) % 2 != 0
                pixels[idx] = if (dark) 50.toByte() else 200.toByte()
                pixels[idx + 1] = if (dark) 100.toByte() else 150.toByte()
                pixels[idx + 2] = if (dark) 150.toByte() else 50.toByte()
                pixels[idx + 3] = 255.toByte()
            }
        }
        return pixels
    }

    private fun colorDistance(a: ByteArray, b: ByteArray): Double {
        var sum = 0.0
        for (i in 0..2) {
            val diff = (a[i].toInt() and 0xFF) - (b[i].toInt() and 0xFF)
            sum += diff * diff
        }
        return Math.sqrt(sum)
    }

    private fun assertAllValid(c: ByteArray) {
        assertEquals(3, c.size)
        for (i in 0..2) {
            assertTrue((c[i].toInt() and 0xFF) in 0..255)
        }
    }
}
