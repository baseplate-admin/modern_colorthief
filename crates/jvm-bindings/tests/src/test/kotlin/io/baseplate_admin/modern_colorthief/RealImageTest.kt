package io.baseplate_admin.modern_colorthief

import org.junit.jupiter.api.Test
import java.io.File
import kotlin.test.assertEquals
import kotlin.test.assertNotNull

@Test
fun testRealImageDominantColor() {
    val resource = javaClass.classLoader.getResource("test_images/test.jpg")
    assertNotNull(resource, "test.jpg should be available on classpath")
    val imageFile = File(resource.toURI())
    val bytes = imageFile.readBytes()
    val width = 400
    val height = 300
    val palette = Colorthief.getPalette(bytes, width, height, 5, 1)
    assertNotNull(palette)
}

@Test
fun testRealImagePalette() {
    val resource = javaClass.classLoader.getResource("test_images/test.jpg")
    assertNotNull(resource, "test.jpg should be available on classpath")
    val imageFile = File(resource.toURI())
    val bytes = imageFile.readBytes()
    val width = 400
    val height = 300
    val color = Colorthief.getColor(bytes, width, height, 1)
    assertNotNull(color)
    assertEquals(3, color.size)
}

@Test
fun testRealImageSmallDominantColor() {
    val resource = javaClass.classLoader.getResource("test_images/test_small.jpg")
    assertNotNull(resource, "test_small.jpg should be available on classpath")
    val imageFile = File(resource.toURI())
    val bytes = imageFile.readBytes()
    val width = 64
    val height = 48
    val color = Colorthief.getColor(bytes, width, height, 1)
    assertNotNull(color)
    assertEquals(3, color.size)
}

@Test
fun testRealImageSmallPalette() {
    val resource = javaClass.classLoader.getResource("test_images/test_small.jpg")
    assertNotNull(resource, "test_small.jpg should be available on classpath")
    val imageFile = File(resource.toURI())
    val bytes = imageFile.readBytes()
    val width = 64
    val height = 48
    val palette = Colorthief.getPalette(bytes, width, height, 5, 1)
    assertNotNull(palette)
}

@Test
fun testResourceLoading() {
    val resource = javaClass.classLoader.getResource("test_images/test.jpg")
    assertNotNull(resource, "test.jpg should be available on classpath")
}

@Test
fun testResourceLoadingSmall() {
    val resource = javaClass.classLoader.getResource("test_images/test_small.jpg")
    assertNotNull(resource, "test_small.jpg should be available on classpath")
}
