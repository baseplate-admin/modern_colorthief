package modern.colorthief

import io.baseplate_admin.modern_colorthief.Colorthief
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertNotEquals
import kotlin.test.assertTrue

/**
 * Cross-image comparison and input consistency tests:
 * - Different images produce different dominant colors
 * - Input consistency: separate pixel arrays with same data yield same results
 * - Color distance calculations between results
 */
class CrossImageTest {

    companion object {
        @BeforeAll
        @JvmStatic
        fun loadNativeLibrary() {
            System.loadLibrary("modern_colorthief")
        }
    }

    // =========================================================================
    // Different images produce different colors
    // =========================================================================

    @Test
    @DisplayName("Different pixel data produces different dominant colors")
    fun testDifferentImagesDifferentColors() {
        val redPixels = createSolidPixels(50, 50, 255, 0, 0)
        val bluePixels = createSolidPixels(50, 50, 0, 0, 255)

        val c1 = Colorthief.getColor(redPixels, 50, 50, 1)
        val c2 = Colorthief.getColor(bluePixels, 50, 50, 1)

        assertNotEquals(c1[0].toInt() and 0xFF, c2[0].toInt() and 0xFF, "Red image R channel must differ from blue image R channel")
        assertNotEquals(c1[2].toInt() and 0xFF, c2[2].toInt() and 0xFF, "Red image B channel must differ from blue image B channel")
    }

    @Test
    @DisplayName("Different gradient patterns produce different palettes")
    fun testDifferentGradientsDifferentPalettes() {
        val hGrad = createHorizontalGradient(200, 100)
        val vGrad = createVerticalGradient(100, 200)

        val p1 = Colorthief.getPalette(hGrad, 200, 100, 10, 5)
        val p2 = Colorthief.getPalette(vGrad, 100, 200, 10, 5)

        // Palettes should differ in at least one entry
        var differs = p1.size != p2.size
        if (!differs) {
            for (i in p1.indices) {
                if (!p1[i].contentEquals(p2[i])) {
                    differs = true
                    break
                }
            }
        }
        assertTrue(differs, "Different gradient patterns should produce different palettes")
    }

    // =========================================================================
    // Input consistency: separate arrays with same data yield same results
    // =========================================================================

    @Test
    @DisplayName("Separate pixel arrays with same data yield same color")
    fun testConsistentAcrossSeparateArrays() {
        val a = createSolidPixels(100, 100, 180, 100, 220)
        val b = createSolidPixels(100, 100, 180, 100, 220)

        val c1 = Colorthief.getColor(a, 100, 100, 1)
        val c2 = Colorthief.getColor(b, 100, 100, 1)

        assertContentEquals(c1, c2)
    }

    @Test
    @DisplayName("Separate pixel arrays with same data yield same palette")
    fun testConsistentPaletteAcrossSeparateArrays() {
        val a = createSolidPixels(100, 100, 180, 100, 220)
        val b = createSolidPixels(100, 100, 180, 100, 220)

        val p1 = Colorthief.getPalette(a, 100, 100, 5, 1)
        val p2 = Colorthief.getPalette(b, 100, 100, 5, 1)

        assertEquals(p1.size, p2.size)
        for (i in p1.indices) {
            assertContentEquals(p1[i], p2[i])
        }
    }

    @Test
    @DisplayName("Color distance between identical inputs is zero")
    fun testColorDistanceZero() {
        val pixels = createSolidPixels(200, 200, 128, 64, 192)
        val c1 = Colorthief.getColor(pixels, 200, 200, 5)
        val c2 = Colorthief.getColor(pixels, 200, 200, 5)

        val dist = colorDistance(c1, c2)
        assertEquals(0.0, dist, 0.001, "Identical inputs must have zero color distance")
    }

    @Test
    @DisplayName("Color distance between different inputs is non-zero")
    fun testColorDistanceNonZero() {
        val red = createSolidPixels(100, 100, 255, 0, 0)
        val cyan = createSolidPixels(100, 100, 0, 255, 255)

        val c1 = Colorthief.getColor(red, 100, 100, 1)
        val c2 = Colorthief.getColor(cyan, 100, 100, 1)

        val dist = colorDistance(c1, c2)
        assertTrue(dist > 100, "Red vs cyan should have large color distance, got $dist")
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

    private fun createHorizontalGradient(width: Int, height: Int): ByteArray {
        val pixels = ByteArray(width * height * 4)
        for (y in 0 until height) {
            for (x in 0 until width) {
                val idx = (y * width + x) * 4
                pixels[idx] = (x * 255 / (width - 1)).toByte()
                pixels[idx + 1] = (y * 255 / (height - 1)).toByte()
                pixels[idx + 2] = 128.toByte()
                pixels[idx + 3] = 255.toByte()
            }
        }
        return pixels
    }

    private fun createVerticalGradient(width: Int, height: Int): ByteArray {
        val pixels = ByteArray(width * height * 4)
        for (y in 0 until height) {
            for (x in 0 until width) {
                val idx = (y * width + x) * 4
                pixels[idx] = 255.toByte()
                pixels[idx + 1] = (y * 255 / (height - 1)).toByte()
                pixels[idx + 2] = 0.toByte()
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
}
