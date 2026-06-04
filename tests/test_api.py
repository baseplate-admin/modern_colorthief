"""Module API surface: exports, docstrings, version."""

import modern_colorthief


def test_get_palette_exists():
    assert hasattr(modern_colorthief, "get_palette")


def test_get_color_exists():
    assert hasattr(modern_colorthief, "get_color")


def test_version_exists():
    assert hasattr(modern_colorthief, "__version__")


def test_get_palette_is_callable():
    assert callable(modern_colorthief.get_palette)


def test_get_color_is_callable():
    assert callable(modern_colorthief.get_color)


def test_get_palette_docstring():
    doc = modern_colorthief.get_palette.__doc__
    assert doc is not None
    assert len(doc) > 20


def test_get_color_docstring():
    doc = modern_colorthief.get_color.__doc__
    assert doc is not None
    assert len(doc) > 20


def test_get_palette_mentions_args():
    doc = modern_colorthief.get_palette.__doc__
    assert "color_count" in doc.lower() or "color" in doc.lower()


def test_get_color_mentions_quality():
    doc = modern_colorthief.get_color.__doc__
    assert "quality" in doc.lower() or "image" in doc.lower()


def test_version_is_string():
    assert isinstance(modern_colorthief.__version__, str)


def test_version_not_empty():
    assert len(modern_colorthief.__version__) > 0


def test_version_format_semver_like():
    parts = modern_colorthief.__version__.split(".")
    assert len(parts) >= 2
    assert parts[0].isdigit()
    assert parts[1].isdigit()


def test_version_no_whitespace():
    assert modern_colorthief.__version__.strip() == modern_colorthief.__version__


def test_internal_palette_location():
    assert hasattr(modern_colorthief, "_get_palette_given_location")


def test_internal_palette_bytes():
    assert hasattr(modern_colorthief, "_get_palette_given_bytes")


def test_internal_color_location():
    assert hasattr(modern_colorthief, "_get_color_given_location")


def test_internal_color_bytes():
    assert hasattr(modern_colorthief, "_get_color_given_bytes")


def test_internal_have_docstrings():
    """Rust docstrings propagate to Python help()."""
    for name in [
        "_get_palette_given_location",
        "_get_palette_given_bytes",
        "_get_color_given_location",
        "_get_color_given_bytes",
    ]:
        fn = getattr(modern_colorthief, name)
        doc = fn.__doc__
        assert doc is not None, f"{name} has no docstring"
