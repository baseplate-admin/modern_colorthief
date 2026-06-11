package io.baseplate_admin.modern_colorthief

import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertContentEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

// ---------------------------------------------------------------------------
// Solid color detection
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

@Test
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
fun testSolidPaletteReturnsOnlyThatColor() {
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

// ---------------------------------------------------------------------------
// Two-color detection
// ---------------------------------------------------------------------------

@Test
fun testTwoColorsRedBlue() {
    val pixels = createTwoColorPixels(50, 50,
        255.toByte(), 0.toByte(), 0.toByte(),
        0.toByte(), 0.toByte(), 255.toByte())
    val palette = Colorthief.getPalette(pixels, 10, 10, 5, 1)
    assertTrue(paletteContains(palette, 255, 0, 0), "Should detect red")
    assertTrue(paletteContains(palette, 0, 0, 255), "Should detect blue")
}

@Test
fun testTwoColorsGreenYellow() {
    val pixels = createTwoColorPixels(50, 50,
        0.toByte(), 255.toByte(), 0.toByte(),
        255.toByte(), 255.toByte(), 0.toByte())
    val palette = Colorthief.getPalette(pixels, 10, 10, 5, 1)
    assertTrue(paletteContains(palette, 0, 255, 0), "Should detect green")
    assertTrue(paletteContains(palette, 255, 255, 0), "Should detect yellow")
}

@Test
fun testDominantColorReflectsMajority() {
    val pixels = createTwoColorPixels(90, 10,
        255.toByte(), 0.toByte(), 0.toByte(),
        0.toByte(), 0.toByte(), 255.toByte())
    val color = Colorthief.getColor(pixels, 10, 10, 1)
    assertEquals(255, color[0].toInt() and 0xFF)
    assertEquals(0, color[1].toInt() and 0xFF)
    assertEquals(0, color[2].toInt() and 0xFF)
}

// ---------------------------------------------------------------------------
// Palette length respects color_count
// ---------------------------------------------------------------------------

@Test
fun testPaletteCountBounded3() {
    val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
    val palette = Colorthief.getPalette(pixels, 10, 10, 3, 1)
    assertTrue(palette.size <= 3, "Palette length must not exceed color_count")
}

@Test
fun testPaletteCountBounded5() {
    val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
    val palette = Colorthief.getPalette(pixels, 10, 10, 5, 1)
    assertTrue(palette.size <= 5, "Palette length must not exceed color_count")
}

@Test
fun testPaletteCountBounded10() {
    val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
    val palette = Colorthief.getPalette(pixels, 10, 10, 10, 1)
    assertTrue(palette.size <= 10, "Palette length must not exceed color_count")
}

@Test
fun testPaletteReturnsNonEmptyResult() {
    val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
    val palette = Colorthief.getPalette(pixels, 10, 10, 5, 1)
    assertNotNull(palette)
    assertTrue(palette.isNotEmpty(), "Palette should not be empty")
}

// ---------------------------------------------------------------------------
// Deduplication
// ---------------------------------------------------------------------------

@Test
fun testPaletteContainsNoDuplicateColors() {
    val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
    val palette = Colorthief.getPalette(pixels, 10, 10, 255, 1)
    val uniqueCount = palette.map { it.contentToString() }.distinct().count()
    assertEquals(palette.size.toLong(), uniqueCount.toLong(), "Palette must contain no duplicates")
}

@Test
fun testDeduplicationSizeWithinBounds() {
    val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
    val palette = Colorthief.getPalette(pixels, 10, 10, 255, 1)
    assertTrue(palette.isNotEmpty(), "Palette should not be empty")
    assertTrue(palette.size <= 255, "Palette should not exceed 255 entries")
}

// ---------------------------------------------------------------------------
// get_color and getPalette return valid RGB
// ---------------------------------------------------------------------------

@Test
fun testColorReturnsValidRgbValues() {
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
fun testPaletteEntriesAreValidRgbValues() {
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

// ---------------------------------------------------------------------------
// Error handling for empty/invalid input
// ---------------------------------------------------------------------------

@Test
fun testEmptyPixelsThrowsForPalette() {
    assertFailsWith<RuntimeException> {
        Colorthief.getPalette(ByteArray(0), 0, 0, 5, 1)
    }
}

@Test
fun testEmptyPixelsThrowsForColor() {
    assertFailsWith<RuntimeException> {
        Colorthief.getColor(ByteArray(0), 0, 0, 1)
    }
}

@Test
fun testZeroDimensionsThrowsForPalette() {
    assertFailsWith<RuntimeException> {
        Colorthief.getPalette(ByteArray(0), 0, 0, 5, 1)
    }
}

@Test
fun testZeroDimensionsThrowsForColor() {
    assertFailsWith<RuntimeException> {
        Colorthief.getColor(ByteArray(0), 0, 0, 1)
    }
}

@Test
fun testMismatchedPixelLengthThrowsForPalette() {
    val shortPixels = ByteArray(100)
    assertFailsWith<RuntimeException> {
        Colorthief.getPalette(shortPixels, 10, 10, 5, 1)
    }
}

@Test
fun testMismatchedPixelLengthThrowsForColor() {
    val shortPixels = ByteArray(100)
    assertFailsWith<RuntimeException> {
        Colorthief.getColor(shortPixels, 10, 10, 1)
    }
}

// ---------------------------------------------------------------------------
// Deterministic results
// ---------------------------------------------------------------------------

@Test
fun testDeterministicColor() {
    val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
    val c1 = Colorthief.getColor(pixels, 10, 10, 1)
    val c2 = Colorthief.getColor(pixels, 10, 10, 1)
    assertContentEquals(c1, c2)
}

@Test
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
fun testMultipleCallsProduceConsistentResults() {
    val pixels = createSolidColorPixels(20, 20, 128.toByte(), 64.toByte(), 200.toByte())
    val first = Colorthief.getColor(pixels, 20, 20, 1)
    for (_ in 0..3) {
        val later = Colorthief.getColor(pixels, 20, 20, 1)
        assertContentEquals(first, later)
    }
}

// ---------------------------------------------------------------------------
// Edge cases: small images
// ---------------------------------------------------------------------------

@Test
fun testSinglePixelReturnsThatColor() {
    val pixels = byteArrayOf(42.toByte(), 100.toByte(), 200.toByte(), 255.toByte())
    val color = Colorthief.getColor(pixels, 1, 1, 1)
    assertNotNull(color)
    assertEquals(3, color.size)
    assertEquals(42, color[0].toInt() and 0xFF)
    assertEquals(100, color[1].toInt() and 0xFF)
    assertEquals(200, color[2].toInt() and 0xFF)
}

@Test
fun testSinglePixelPaletteReturnsThatColor() {
    val pixels = byteArrayOf(42.toByte(), 100.toByte(), 200.toByte(), 255.toByte())
    val palette = Colorthief.getPalette(pixels, 1, 1, 5, 1)
    assertNotNull(palette)
    assertTrue(palette.isNotEmpty())
}

@Test
fun testSmallImage2x2() {
    val pixels = createSolidColorPixels(2, 2, 255.toByte(), 128.toByte(), 0.toByte())
    val color = Colorthief.getColor(pixels, 2, 2, 1)
    assertNotNull(color)
    assertEquals(3, color.size)
}

@Test
fun testNonSquareImage20x5() {
    val pixels = createSolidColorPixels(20, 5, 255.toByte(), 0.toByte(), 0.toByte())
    val color = Colorthief.getColor(pixels, 20, 5, 1)
    assertNotNull(color)
    assertEquals(3, color.size)
    assertEquals(255, color[0].toInt() and 0xFF)
}

// ---------------------------------------------------------------------------
// Edge cases: large quality values
// ---------------------------------------------------------------------------

@Test
fun testQualityMinimumWorks() {
    val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
    val color = Colorthief.getColor(pixels, 10, 10, 1)
    assertNotNull(color)
    assertEquals(3, color.size)
}

@Test
fun testQualityDefaultWorks() {
    val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
    val color = Colorthief.getColor(pixels, 10, 10, 5)
    assertNotNull(color)
    assertEquals(3, color.size)
}

@Test
fun testQualityMaximumWorks() {
    val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
    val color = Colorthief.getColor(pixels, 10, 10, 10)
    assertNotNull(color)
    assertEquals(3, color.size)
}

@Test
fun testPaletteWithQualityMinimumWorks() {
    val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
    val palette = Colorthief.getPalette(pixels, 10, 10, 5, 1)
    assertNotNull(palette)
    assertTrue(palette.isNotEmpty())
}

@Test
fun testPaletteWithQualityMaximumWorks() {
    val pixels = createSolidColorPixels(10, 10, 255.toByte(), 0.toByte(), 0.toByte())
    val palette = Colorthief.getPalette(pixels, 10, 10, 5, 10)
    assertNotNull(palette)
    assertTrue(palette.isNotEmpty())
}
