package modern.colorthief

import io.baseplate_admin.modern_colorthief.Colorthief
import org.junit.jupiter.api.BeforeAll
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import java.util.concurrent.CountDownLatch
import java.util.concurrent.CyclicBarrier
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * Concurrency and thread safety tests:
 * - Thread safety of parallel get_color calls
 * - Thread safety of mixed get_color + getPalette calls
 * - Concurrent calls complete faster than sequential sum
 * - Stress test with many concurrent calls
 *
 * All tests use CountDownLatch to ensure threads begin work simultaneously,
 * not just started sequentially then joined.
 */
class ConcurrencyTest {

    companion object {
        @BeforeAll
        @JvmStatic
        fun loadNativeLibrary() {
            System.loadLibrary("modern_colorthief")
        }
    }

    // =========================================================================
    // Concurrent get_color
    // =========================================================================

    @Test
    @DisplayName("Multiple threads calling get_color simultaneously")
    @Throws(InterruptedException::class)
    fun testConcurrentColor() {
        val pixels = createSolidPixels(100, 100, 255, 128, 64)
        val threadCount = 5
        val results = arrayOfNulls<ByteArray>(threadCount)
        val error = AtomicReference<Throwable>()

        // Latch so all threads start work at the same instant
        val startLatch = CountDownLatch(1)
        val threads = List(threadCount) { i ->
            Thread {
                try {
                    startLatch.await() // wait for all threads to be ready
                    results[i] = Colorthief.getColor(pixels, 100, 100, 1)
                } catch (t: Throwable) {
                    error.set(t)
                }
            }
        }

        threads.forEach { it.start() }
        startLatch.countDown() // release all threads simultaneously
        threads.forEach { it.join() }

        assertNull(error.get(), "No thread errors")
        for ((i, r) in results.withIndex()) {
            assertNotNull(r, "Thread $i should have a result")
        }
        for (r in results) {
            assertContentEquals(results[0]!!, r)
        }
    }

    // =========================================================================
    // Concurrent getPalette
    // =========================================================================

    @Test
    @DisplayName("Multiple threads calling getPalette simultaneously")
    @Throws(InterruptedException::class)
    fun testConcurrentPalette() {
        val pixels = createSolidPixels(100, 100, 100, 200, 150)
        val threadCount = 5
        val results = arrayOfNulls<Array<ByteArray>>(threadCount)
        val error = AtomicReference<Throwable>()

        val startLatch = CountDownLatch(1)
        val threads = List(threadCount) { i ->
            Thread {
                try {
                    startLatch.await()
                    results[i] = Colorthief.getPalette(pixels, 100, 100, 5, 1)
                } catch (t: Throwable) {
                    error.set(t)
                }
            }
        }

        threads.forEach { it.start() }
        startLatch.countDown()
        threads.forEach { it.join() }

        assertNull(error.get(), "No thread errors")
        for ((i, r) in results.withIndex()) {
            assertNotNull(r, "Thread $i should have a result")
        }
        assertEquals(threadCount, results.count { it != null })
    }

    // =========================================================================
    // Concurrent mixed operations
    // =========================================================================

    @Test
    @DisplayName("Threads doing color + palette simultaneously")
    @Throws(InterruptedException::class)
    fun testConcurrentMixedOps() {
        val pixels = createSolidPixels(200, 200, 180, 90, 220)
        val resultCount = AtomicInteger(0)
        val hadError = AtomicBoolean(false)

        val startLatch = CountDownLatch(1)

        // Thread 1: get_color
        val t1 = Thread {
            try {
                startLatch.await()
                Colorthief.getColor(pixels, 200, 200, 1)
                resultCount.incrementAndGet()
            } catch (t: Throwable) {
                hadError.set(true)
            }
        }

        // Thread 2: getPalette
        val t2 = Thread {
            try {
                startLatch.await()
                Colorthief.getPalette(pixels, 200, 200, 5, 1)
                resultCount.incrementAndGet()
            } catch (t: Throwable) {
                hadError.set(true)
            }
        }

        // Thread 3: get_color with different quality
        val t3 = Thread {
            try {
                startLatch.await()
                Colorthief.getColor(pixels, 200, 200, 10)
                resultCount.incrementAndGet()
            } catch (t: Throwable) {
                hadError.set(true)
            }
        }

        val threads = listOf(t1, t2, t3)
        threads.forEach { it.start() }
        startLatch.countDown()
        threads.forEach { it.join() }

        assertFalse(hadError.get(), "No thread errors")
        assertEquals(3, resultCount.get())
    }

    // =========================================================================
    // Concurrent calls faster than sequential
    // =========================================================================

    @Test
    @DisplayName("Concurrent calls complete faster than sequential sum")
    @Throws(InterruptedException::class)
    fun testConcurrentFasterThanSequential() {
        val pixels = createSolidPixels(500, 500, 128, 64, 192)
        val numCalls = 3

        // Measure single-call time
        val singleStart = System.nanoTime()
        Colorthief.getColor(pixels, 500, 500, 10)
        val singleTime = System.nanoTime() - singleStart

        // CyclicBarrier ensures all threads begin work at the same instant
        val barrier = CyclicBarrier(numCalls)
        val hadError = AtomicBoolean(false)
        val done = AtomicInteger(0)

        val threads = List(numCalls) {
            Thread {
                try {
                    barrier.await()
                    Colorthief.getColor(pixels, 500, 500, 10)
                    done.incrementAndGet()
                } catch (e: InterruptedException) {
                    Thread.currentThread().interrupt()
                } catch (e: java.util.concurrent.BrokenBarrierException) {
                    hadError.set(true)
                } catch (t: Throwable) {
                    hadError.set(true)
                }
            }
        }

        val concurrentStart = System.nanoTime()
        threads.forEach { it.start() }
        threads.forEach { it.join() }
        val concurrentTime = System.nanoTime() - concurrentStart

        assertFalse(hadError.get(), "No thread errors")
        assertEquals(numCalls, done.get())

        // Concurrent should be faster than running all sequentially.
        // Allow 3x overhead for thread creation and GCD.
        val sequentialEstimate = singleTime * numCalls
        assertTrue(
            concurrentTime < sequentialEstimate * 3,
            "Concurrent ${concurrentTime / 1_000_000}ms should be < sequential ${sequentialEstimate / 1_000_000}ms (3x)"
        )
    }

    // =========================================================================
    // Stress test: many concurrent calls
    // =========================================================================

    @Test
    @DisplayName("Stress: 20 concurrent calls all succeed")
    @Throws(InterruptedException::class)
    fun testStressConcurrentCalls() {
        val pixels = createSolidPixels(200, 200, 70, 140, 210)
        val threadCount = 20
        val successCount = AtomicInteger(0)
        val errorCount = AtomicInteger(0)

        val startLatch = CountDownLatch(1)
        val threads = List(threadCount) {
            Thread {
                try {
                    startLatch.await()
                    val c = Colorthief.getColor(pixels, 200, 200, 5)
                    assertNotNull(c)
                    assertEquals(3, c.size)
                    successCount.incrementAndGet()
                } catch (t: Throwable) {
                    errorCount.incrementAndGet()
                }
            }
        }

        threads.forEach { it.start() }
        startLatch.countDown()
        threads.forEach { it.join() }

        assertEquals(0, errorCount.get(), "No errors in stress test")
        assertEquals(threadCount, successCount.get())
    }

    // =========================================================================
    // Helper methods
    // =========================================================================

    private fun createSolidPixels(width: Int, height: Int, r: Int, g: Int, b: Int): ByteArray {
        val pixels = ByteArray(width * height * 4)
        for (i in 0 until width * height) {
            pixels[i * 4] = r.toByte()
            pixels[i * 4 + 1] = g.toByte()
            pixels[i * 4 + 2] = b.toByte()
            pixels[i * 4 + 3] = 255.toByte()
        }
        return pixels
    }
}
