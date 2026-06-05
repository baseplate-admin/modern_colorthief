"""Concurrency: verify thread-safety of parallel processing."""

import threading
from pathlib import Path

import modern_colorthief

BASE_DIR = Path(__file__).resolve().parent
TEST_IMAGE = BASE_DIR / "test.jpg"


def test_concurrent_color():
    """Multiple threads calling get_color simultaneously."""
    results, errors = [], []

    def task():
        try:
            results.append(modern_colorthief.get_color(str(TEST_IMAGE)))
        except Exception as e:
            errors.append(e)

    threads = [threading.Thread(target=task) for _ in range(3)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    assert not errors
    assert len(results) == 3
    assert all(r == results[0] for r in results)


def test_concurrent_mixed_ops():
    """Threads doing color + palette + bytes simultaneously."""
    results, errors = [], []

    def color_task():
        try:
            results.append(modern_colorthief.get_color(str(TEST_IMAGE)))
        except Exception as e:
            errors.append(e)

    def palette_task():
        try:
            results.append(modern_colorthief.get_palette(str(TEST_IMAGE), color_count=3))
        except Exception as e:
            errors.append(e)

    with open(TEST_IMAGE, "rb") as f:
        data = f.read()

    def bytes_task():
        try:
            results.append(modern_colorthief.get_color(data))
        except Exception as e:
            errors.append(e)

    threads = [
        threading.Thread(target=color_task),
        threading.Thread(target=palette_task),
        threading.Thread(target=bytes_task),
    ]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    assert not errors
    assert len(results) == 3
