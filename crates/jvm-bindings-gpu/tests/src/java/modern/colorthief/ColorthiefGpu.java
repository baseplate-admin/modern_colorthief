package modern.colorthief;

public class ColorthiefGpu {
    public static native byte[][] getPalette(byte[] pixels, int width, int height, int colorCount, int quality);

    public static native byte[] getColor(byte[] pixels, int width, int height, int quality);
}
