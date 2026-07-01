package io.baseplate_admin.modern_colorthief

import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

@Test
fun testCrossImagePaletteDiffer() {
    val solidRed = createSolidColorPixels(20, 20, 255.toByte(), 0.toByte(), 0.toByte())
    val solidBlue = createSolidColorPixels(20, 20, 0.toByte(), 0.toByte(), 255.toByte())
    val paletteRed = Colorthief.getPalette(solidRed, 20, 20, 5, 1)
    val paletteBlue = Colorthief.getPalette(solidBlue, 20, 20, 5, 1)
    assertNotNull(paletteRed)
    assertNotNull(paletteBlue)
    assertTrue(paletteRed.isNotEmpty())
    assertTrue(paletteBlue.isNotEmpty())
    val firstRed = paletteRed[0]
    val firstBlue = paletteBlue[0]
    if ((firstRed[0].toInt() and 0xFF) == (firstBlue[0].toInt() and 0xFF) &&
        (firstRed[1].toInt() and 0xFF) == (firstBlue[1].toInt() and 0xFF) &&
        (firstRed[2].toInt() and 0xFF) == (firstBlue[2].toInt() and 0xFF)) {
        throw AssertionError("Different images should produce different palettes")
    }
}

@Test
fun testGradientImageProducesMultipleColors() {
    val width = 50
    val height = 50
    val pixels = ByteArray(width * height * 4)
    for (y in 0 until height) {
        for (x in 0 until width) {
            val idx = (y * width + x) * 4
            val r = ((x.toFloat() / width) * 255).toInt().toByte()
            val g = ((y.toFloat() / height) * 255).toInt().toByte()
            pixels[idx] = r
            pixels[idx + 1] = g
            pixels[idx + 2] = 0.toByte()
            pixels[idx + 3] = 255.toByte()
        }
    }
    val palette = Colorthief.getPalette(pixels, width, height, 10, 1)
    assertNotNull(palette)
    assertTrue(palette.size > 1, "Gradient should produce more than 1 color")
}
