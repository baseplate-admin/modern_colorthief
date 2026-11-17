package io.baseplate_admin.modern_colorthief;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * Asynchronous wrapper around the native {@link Colorthief} API.
 *
 * <p>Uses a virtual-thread executor on Java 21+ (where
 * {@link Executors#newVirtualThreadPerTaskExecutor()} is available) and falls
 * back to a cached thread pool on earlier versions. The native JNI call
 * itself is unchanged — this wrapper only moves it off the calling thread so
 * that the host (e.g. a servlet container or a Kotlin coroutine dispatcher) is
 * not blocked.
 *
 * <pre>{@code
 * CompletableFuture<byte[][]> future =
 *     ColorthiefAsync.getPalette(pixels, width, height, 5, 1);
 * byte[][] palette = future.join();
 * }</pre>
 */
public final class ColorthiefAsync {

    private static final ExecutorService EXECUTOR;

    static {
        if (java.lang.Runtime.version().feature() >= 21) {
            EXECUTOR = Executors.newVirtualThreadPerTaskExecutor();
        } else {
            EXECUTOR = Executors.newCachedThreadPool();
        }
    }

    private ColorthiefAsync() {}

    /**
     * Extract a palette asynchronously.
     *
     * @param pixels     raw RGBA pixel data (4 bytes per pixel)
     * @param width      image width in pixels
     * @param height     image height in pixels
     * @param colorCount number of colors to extract
     * @param quality    sampling quality
     * @return a {@link CompletableFuture} that resolves to the palette
     */
    public static CompletableFuture<byte[][]> getPalette(
            byte[] pixels, int width, int height, int colorCount, int quality) {
        return CompletableFuture.supplyAsync(
                () -> Colorthief.getPalette(pixels, width, height, colorCount, quality),
                EXECUTOR);
    }

    /**
     * Extract the dominant color asynchronously.
     *
     * @param pixels  raw RGBA pixel data (4 bytes per pixel)
     * @param width   image width in pixels
     * @param height  image height in pixels
     * @param quality sampling quality
     * @return a {@link CompletableFuture} that resolves to the dominant color as {@code byte[3]}
     */
    public static CompletableFuture<byte[]> getColor(
            byte[] pixels, int width, int height, int quality) {
        return CompletableFuture.supplyAsync(
                () -> Colorthief.getColor(pixels, width, height, quality),
                EXECUTOR);
    }
}
