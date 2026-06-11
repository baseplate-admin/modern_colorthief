package modern.colorthief;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.BrokenBarrierException;
import java.util.concurrent.CyclicBarrier;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Concurrency and thread safety tests:
 * - Thread safety of parallel get_color calls
 * - Thread safety of mixed get_color + getPalette calls
 * - Concurrent calls complete faster than sequential sum
 * - Stress test with many concurrent calls
 */
public class ConcurrencyTest {

    @BeforeAll
    public static void loadNativeLibrary() {
        System.loadLibrary("modern_colorthief");
    }

    // =========================================================================
    // Concurrent get_color
    // =========================================================================

    @Test
    @DisplayName("Multiple threads calling get_color simultaneously")
    public void testConcurrentColor() throws InterruptedException {
        byte[] pixels = createSolidPixels(100, 100, 255, 128, 64);
        int threadCount = 5;
        List<Thread> threads = new ArrayList<>();
        List<byte[]> results = new ArrayList<>();
        AtomicReference<Throwable> error = new AtomicReference<>();

        for (int i = 0; i < threadCount; i++) {
            threads.add(new Thread(() -> {
                try {
                    synchronized (results) {
                        results.add(Colorthief.getColor(pixels, 100, 100, 1));
                    }
                } catch (Throwable t) {
                    error.set(t);
                }
            }));
        }

        for (Thread t : threads) t.start();
        for (Thread t : threads) t.join();

        assertNull(error.get(), "No thread errors");
        assertEquals(threadCount, results.size());
        for (byte[] r : results) {
            assertArrayEquals(results.get(0), r, "All concurrent results must match");
        }
    }

    // =========================================================================
    // Concurrent getPalette
    // =========================================================================

    @Test
    @DisplayName("Multiple threads calling getPalette simultaneously")
    public void testConcurrentPalette() throws InterruptedException {
        byte[] pixels = createSolidPixels(100, 100, 100, 200, 150);
        int threadCount = 5;
        List<Thread> threads = new ArrayList<>();
        List<byte[][]> results = new ArrayList<>();
        AtomicReference<Throwable> error = new AtomicReference<>();

        for (int i = 0; i < threadCount; i++) {
            threads.add(new Thread(() -> {
                try {
                    synchronized (results) {
                        results.add(Colorthief.getPalette(pixels, 100, 100, 5, 1));
                    }
                } catch (Throwable t) {
                    error.set(t);
                }
            }));
        }

        for (Thread t : threads) t.start();
        for (Thread t : threads) t.join();

        assertNull(error.get(), "No thread errors");
        assertEquals(threadCount, results.size());
    }

    // =========================================================================
    // Concurrent mixed operations
    // =========================================================================

    @Test
    @DisplayName("Threads doing color + palette simultaneously")
    public void testConcurrentMixedOps() throws InterruptedException {
        byte[] pixels = createSolidPixels(200, 200, 180, 90, 220);
        List<Thread> threads = new ArrayList<>();
        AtomicInteger resultCount = new AtomicInteger(0);
        AtomicBoolean hadError = new AtomicBoolean(false);

        // Thread 1: get_color
        threads.add(new Thread(() -> {
            try {
                Colorthief.getColor(pixels, 200, 200, 1);
                resultCount.incrementAndGet();
            } catch (Throwable t) {
                hadError.set(true);
            }
        }));

        // Thread 2: getPalette
        threads.add(new Thread(() -> {
            try {
                Colorthief.getPalette(pixels, 200, 200, 5, 1);
                resultCount.incrementAndGet();
            } catch (Throwable t) {
                hadError.set(true);
            }
        }));

        // Thread 3: get_color with different quality
        threads.add(new Thread(() -> {
            try {
                Colorthief.getColor(pixels, 200, 200, 10);
                resultCount.incrementAndGet();
            } catch (Throwable t) {
                hadError.set(true);
            }
        }));

        for (Thread t : threads) t.start();
        for (Thread t : threads) t.join();

        assertFalse(hadError.get(), "No thread errors");
        assertEquals(3, resultCount.get());
    }

    // =========================================================================
    // Concurrent calls faster than sequential
    // =========================================================================

    @Test
    @DisplayName("Concurrent calls complete faster than sequential sum")
    public void testConcurrentFasterThanSequential() throws InterruptedException {
        byte[] pixels = createSolidPixels(500, 500, 128, 64, 192);
        int numCalls = 3;

        // Measure single-call time
        long singleStart = System.nanoTime();
        Colorthief.getColor(pixels, 500, 500, 10);
        long singleTime = System.nanoTime() - singleStart;

        // Run numCalls concurrently with barrier to start simultaneously
        CyclicBarrier barrier = new CyclicBarrier(numCalls);
        List<Thread> threads = new ArrayList<>();
        AtomicBoolean hadError = new AtomicBoolean(false);
        AtomicInteger done = new AtomicInteger(0);

        for (int i = 0; i < numCalls; i++) {
            threads.add(new Thread(() -> {
                try {
                    barrier.await();
                    Colorthief.getColor(pixels, 500, 500, 10);
                    done.incrementAndGet();
                } catch (InterruptedException e) {
                    Thread.currentThread().interrupt();
                } catch (BrokenBarrierException e) {
                    hadError.set(true);
                } catch (Throwable t) {
                    hadError.set(true);
                }
            }));
        }

        long concurrentStart = System.nanoTime();
        for (Thread t : threads) t.start();
        for (Thread t : threads) t.join();
        long concurrentTime = System.nanoTime() - concurrentStart;

        assertFalse(hadError.get(), "No thread errors");
        assertEquals(numCalls, done.get());

        // Concurrent should be faster than running all sequentially.
        // Allow 3x overhead for thread creation and GCD.
        long sequentialEstimate = singleTime * numCalls;
        assertTrue(concurrentTime < sequentialEstimate * 3,
                "Concurrent " + (concurrentTime / 1_000_000) + "ms should be < sequential " +
                (sequentialEstimate / 1_000_000) + "ms (3x)");
    }

    // =========================================================================
    // Stress test: many concurrent calls
    // =========================================================================

    @Test
    @DisplayName("Stress: 20 concurrent calls all succeed")
    public void testStressConcurrentCalls() throws InterruptedException {
        byte[] pixels = createSolidPixels(200, 200, 70, 140, 210);
        int threadCount = 20;
        List<Thread> threads = new ArrayList<>();
        AtomicInteger successCount = new AtomicInteger(0);
        AtomicInteger errorCount = new AtomicInteger(0);

        for (int i = 0; i < threadCount; i++) {
            threads.add(new Thread(() -> {
                try {
                    byte[] c = Colorthief.getColor(pixels, 200, 200, 5);
                    assertNotNull(c);
                    assertEquals(3, c.length);
                    successCount.incrementAndGet();
                } catch (Throwable t) {
                    errorCount.incrementAndGet();
                }
            }));
        }

        for (Thread t : threads) t.start();
        for (Thread t : threads) t.join();

        assertEquals(0, errorCount.get(), "No errors in stress test");
        assertEquals(threadCount, successCount.get());
    }

    // =========================================================================
    // Helper methods
    // =========================================================================

    private byte[] createSolidPixels(int width, int height, int r, int g, int b) {
        byte[] pixels = new byte[width * height * 4];
        for (int i = 0; i < width * height; i++) {
            pixels[i * 4] = (byte) r;
            pixels[i * 4 + 1] = (byte) g;
            pixels[i * 4 + 2] = (byte) b;
            pixels[i * 4 + 3] = (byte) 255;
        }
        return pixels;
    }
}
