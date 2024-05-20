import modern_colorthief


def test_version():
    version = modern_colorthief.__version__

    assert isinstance(version, str)
