package io.baseplate_admin.modern_colorthief

import org.junit.jupiter.api.Test
import java.io.File
import java.net.URI
import kotlin.test.assertEquals
import kotlin.test.assertNotNull

private val classLoader = ClassLoader.getSystemClassLoader()
private fun loadResource(name: String) = classLoader.getResource(name)

@Test
fun testRealImageDominantColor() {
    val resource = loadResource("test_images/test.jpg")
    assertNotNull(resource, "test.jpg should be available on classpath")
    val imageFile = File(URI.create(resource.toString()).toURL().toURI())
    val bytes = imageFile.readBytes()
    val width = 400
    val height = 300
    val palette = Colorthief.getPalette(bytes, width, height, 5, 1)
    assertNotNull(palette)
}

@Test
fun testRealImagePalette() {
    val resource = loadResource("test_images/test.jpg")
    assertNotNull(resource, "test.jpg should be available on classpath")
    val imageFile = File(URI.create(resource.toString()).toURL().toURI())
    val bytes = imageFile.readBytes()
    val width = 400
    val height = 300
    val color = Colorthief.getColor(bytes, width, height, 1)
    assertNotNull(color)
    assertEquals(3, color.size)
}

@Test
fun testRealImageSmallDominantColor() {
    val resource = loadResource("test_images/test_small.jpg")
    assertNotNull(resource, "test_small.jpg should be available on classpath")
    val imageFile = File(URI.create(resource.toString()).toURL().toURI())
    val bytes = imageFile.readBytes()
    val width = 64
    val height = 48
    val color = Colorthief.getColor(bytes, width, height, 1)
    assertNotNull(color)
    assertEquals(3, color.size)
}

@Test
fun testRealImageSmallPalette() {
    val resource = loadResource("test_images/test_small.jpg")
    assertNotNull(resource, "test_small.jpg should be available on classpath")
    val imageFile = File(URI.create(resource.toString()).toURL().toURI())
    val bytes = imageFile.readBytes()
    val width = 64
    val height = 48
    val palette = Colorthief.getPalette(bytes, width, height, 5, 1)
    assertNotNull(palette)
}

@Test
fun testResourceLoading() {
    val resource = loadResource("test_images/test.jpg")
    assertNotNull(resource, "test.jpg should be available on classpath")
}

@Test
fun testResourceLoadingSmall() {
    val resource = loadResource("test_images/test_small.jpg")
    assertNotNull(resource, "test_small.jpg should be available on classpath")
}
