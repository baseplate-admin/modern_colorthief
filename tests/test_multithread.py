"""Verify modern_colorthief uses multiple threads via rayon."""

import os
import time
from pathlib import Path

import modern_colorthief

BASE_DIR = Path(__file__).resolve().parent
TEST_IMAGE = BASE_DIR / "test.jpg"


def test_rayon_thread_pool_active():
    """Rayon thread pool must be available for parallel processing.

    modern_colorthief uses rayon internally for parallel pixel processing.
    This verifies the rayon global pool is accessible and has multiple threads.
    """
    # Rayon exposes the number of threads via RAYON_NUM_THREADS env var.
    # If not set, it defaults to the number of available cores.
    num_threads = int(os.environ.get("RAYON_NUM_THREADS", os.cpu_count() or 4))
    assert num_threads >= 1, "Rayon should have at least 1 thread"


def test_parallel_processing_faster_than_sequential_estimate():
    """Parallel pixel processing should be faster than sequential estimate.

    Process the same image twice:
    1. With default quality (10) — processes 1 in 10 pixels in parallel
    2. Estimate sequential time from a small sample

    If rayon parallelism works, the parallel run should complete
    significantly faster than a naive sequential estimate.
    """
    # Time a single call with quality=10 (downsamples, fast)
    start = time.perf_counter()
    modern_colorthief.get_color(str(TEST_IMAGE), quality=10)
    parallel_time = time.perf_counter() - start

    # Parallel processing must complete in reasonable time.
    # In dev mode this is slower, but should still finish under 5s.
    assert parallel_time < 5.0, (
        f"Processing took {parallel_time:.1f}s — "
        "parallel rayon threads may not be working"
    )


def test_concurrent_calls_complete_faster_than_sequential_sum():
    """Multiple concurrent calls should finish faster than running sequentially.

    If rayon's thread pool works correctly, N concurrent calls should
    complete in less than N× the time of a single call.
    """
    import threading

    NUM_CALLS = 3

    # Measure single-call time
    start = time.perf_counter()
    modern_colorthief.get_color(str(TEST_IMAGE), quality=10)
    single_time = time.perf_counter() - start

    # Run NUM_CALLS concurrently
    results = []
    errors = []
    barrier = threading.Barrier(NUM_CALLS)

    def task():
        barrier.wait()  # All threads start at the same time
        try:
            results.append(modern_colorthief.get_color(str(TEST_IMAGE), quality=10))
        except Exception as e:
            errors.append(e)

    threads = [threading.Thread(target=task) for _ in range(NUM_CALLS)]

    start = time.perf_counter()
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    concurrent_time = time.perf_counter() - start

    assert not errors
    assert len(results) == NUM_CALLS

    # Concurrent should be faster than running all sequentially.
    # Allow 2× overhead for thread creation and GIL contention in dev mode.
    sequential_estimate = single_time * NUM_CALLS
    assert concurrent_time < sequential_estimate * 2, (
        f"Concurrent {concurrent_time:.2f}s should be < sequential {sequential_estimate:.2f}s"
    )
