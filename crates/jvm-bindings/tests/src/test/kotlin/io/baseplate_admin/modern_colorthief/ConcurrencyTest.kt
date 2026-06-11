package io.baseplate_admin.modern_colorthief

import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull

@Test
fun testConcurrentPaletteCalls() {
    val pixels = createSolidColorPixels(50, 50, 100.toByte(), 150.toByte(), 200.toByte())
    val jobs = (1..5).map {
        kotlin.concurrent.thread {
            val palette = Colorthief.getPalette(pixels, 50, 50, 5, 1)
            assertNotNull(palette)
            palette.size
        }
    }
    jobs.forEach { it.join() }
}

@Test
fun testConcurrentColorCalls() {
    val pixels = createSolidColorPixels(50, 50, 100.toByte(), 150.toByte(), 200.toByte())
    val jobs = (1..5).map {
        kotlin.concurrent.thread {
            val color = Colorthief.getColor(pixels, 50, 50, 1)
            assertNotNull(color)
            assertEquals(3, color.size)
        }
    }
    jobs.forEach { it.join() }
}
