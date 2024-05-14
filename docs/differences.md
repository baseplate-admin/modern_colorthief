# Differences

## With `fast-colorthief`

-   Supports more architectures. ( `pybind11` vs `pyo3` )
-   Doesn't have a hard dependency on `numpy`
-   Code is simple compared to `fast-colorthief`'s CPP codebase
-   Automated tooling powered by `maturin` and `github-actions`
-   The size of `fast-colorthief` is 52kb-60kb,compared to 500kb-700kb for `modern_colorthief`

## With `color-thief-py`

-   Superior execution time (nearly 100x)
-   Doesn't have a hard dependency on `pillow`
-   `color-thief`'s codebase is not in par with modern python versions
