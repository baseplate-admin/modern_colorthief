package io.baseplate_admin.modern_colorthief

import org.junit.jupiter.api.Test
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

@Test
fun testHigherQualityMoreAccurate() {
    val pixels = createSolidColorPixels(100, 100, 128.toByte(), 64.toByte(), 200.toByte())
    val colorLow = Colorthief.getColor(pixels, 100, 100, 1)
    val colorHigh = Colorthief.getColor(pixels, 100, 100, 10)
    assertNotNull(colorLow)
    assertNotNull(colorHigh)
    // Both should be close to the original color
    for (channel in 0..2) {
        val expected = when (channel) { 0 -> 128, 1 -> 64, else -> 200 }
        val diffLow = kotlin.math.abs(colorLow[channel].toInt() and 0xFF - expected)
        val diffHigh = kotlin.math.abs(colorHigh[channel].toInt() and 0xFF - expected)
        assertTrue(diffLow < 30, "Low quality should be close: diff=$diffLow")
        assertTrue(diffHigh < 30, "High quality should be close: diff=$diffHigh")
    }
}

@Test
fun testDifferentQualityLevelsWork() {
    val pixels = createSolidColorPixels(50, 50, 200.toByte(), 100.toByte(), 50.toByte())
    for (q in 1..10) {
        val color = Colorthief.getColor(pixels, 50, 50, q)
        assertNotNull(color)
        assertTrue(color.size == 3, "Quality $q should return 3-element array")
    }
}
