package colorthief

import java.nio.file.Files
import java.nio.file.Path

fun main() {
    val pixels = Files.readAllBytes(Path.of("examples/test.jpg"))
    // Note: This expects raw RGBA pixel data. Use Java AWT or TwelveMonkeys to decode.
    // val palette = Colorthief.getPalette(pixels, width, height, 10, 10)
    // println("Palette: ${palette.contentDeepToString()}")
    //
    // val color = Colorthief.getColor(pixels, width, height, 10)
    // println("Dominant color: ${color.contentToString()}")
}
