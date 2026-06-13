package colorthief;

import java.nio.file.Files;
import java.nio.file.Path;

public class ColorthiefTest {
    public static void main(String[] args) throws Exception {
        byte[] pixels = Files.readAllBytes(Path.of("examples/test.jpg"));
        // Note: This expects raw RGBA pixel data. Use Java AWT or TwelveMonkeys to decode.
        // byte[][] palette = Colorthief.getPalette(pixels, width, height, 10, 10);
        // System.out.println("Palette: " + java.util.Arrays.toString(palette));
        //
        // byte[] color = Colorthief.getColor(pixels, width, height, 10);
        // System.out.println("Dominant color: " + java.util.Arrays.toString(color));
    }
}
