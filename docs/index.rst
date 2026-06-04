.. Modern Colorthief documentation master file

===========================================
  Modern Colorthief
===========================================

.. raw:: html

   <div class="hero-section">

   <h1 class="hero-title">Modern Colorthief</h1>

   <p class="hero-subtitle">
      Extract dominant colors and color palettes from images at blazing speed.
   </p>

   <p>
      <span class="speed-badge">~100x faster than pure Python colorthief</span>
   </p>

   <div class="badge-row">
      <img src="https://img.shields.io/pypi/v/modern_colorthief.svg?color=6366f1&style=flat-square" alt="PyPI Version"/>
      <img src="https://img.shields.io/pypi/pyversions/modern_colorthief.svg?color=6366f1&style=flat-square" alt="Python Versions"/>
      <img src="https://img.shields.io/pypi/l/modern_colorthief.svg?color=6366f1&style=flat-square" alt="License"/>
      <img src="https://img.shields.io/crates/v/modern_colorthief.svg?color=4f46e5&label=cargo&style=flat-square" alt="Crates.io"/>
      <img src="https://img.shields.io/badge/implemented%20in-Rust-dca280?style=flat-square" alt="Rust"/>
   </div>

   </div>
   <div style="clear:both;"></div>

.. rubric:: Overview

Modern Colorthief is a high-performance Python library for extracting dominant
colors and color palettes from images. Built with `Rust`_ and `PyO3`_, it
delivers near-C++ performance with the simplicity of a pure Python API.

.. note::

   Powered by the **Median Cut Color Quantization** algorithm, compiled to a
   native extension via **Maturin**. Zero runtime dependencies beyond Python
   itself.

.. rubric:: Quick Start

.. code-block:: python

   >>> from modern_colorthief import get_color, get_palette
   >>>
   >>> dominant = get_color("photo.jpg")
   >>> print(dominant)
   (139, 69, 19)
   >>>
   >>> palette = get_palette("photo.jpg", color_count=5)
   >>> print(palette)
   [(139, 69, 19), (220, 20, 60), (255, 250, 240), (34, 139, 34), (70, 130, 180)]

.. rubric:: Explore the Documentation

   .. raw:: html

      <div class="card-grid">

      <a class="card-item" href="installation.html">
         <span class="card-icon">&#x1F4E6;</span>
         <div class="card-title">Installation</div>
         <div class="card-desc">Get up and running in seconds. Install via pip, poetry, or uv.</div>
      </a>

      <a class="card-item" href="usage.html">
         <span class="card-icon">&#x26A1;</span>
         <div class="card-title">Usage</div>
         <div class="card-desc">Work with file paths, BytesIO, Pillow images, and NumPy arrays.</div>
      </a>

      <a class="card-item" href="api.html">
         <span class="card-icon">&#x1F4CB;</span>
         <div class="card-title">API Reference</div>
         <div class="card-desc">Complete documentation for get_color, get_palette, and all parameters.</div>
      </a>

      <a class="card-item" href="cli.html">
         <span class="card-icon">&#x1F5A5;</span>
         <div class="card-title">Command Line</div>
         <div class="card-desc">Use the modern-colorthief CLI for quick color extraction from the terminal.</div>
      </a>

      <a class="card-item" href="benchmarks.html">
         <span class="card-icon">&#x1F4CA;</span>
         <div class="card-title">Benchmarks</div>
         <div class="card-desc">See how modern_colorthief stacks up against Python and C++ alternatives.</div>
      </a>

      <a class="card-item" href="mmcq.html">
         <span class="card-icon">&#x1F3A8;</span>
         <div class="card-title">Algorithm</div>
         <div class="card-desc">Deep dive into the Median Cut Color Quantization algorithm.</div>
      </a>

      </div>
      <div style="clear:both;"></div>

.. rubric:: Why Modern Colorthief?

- **Blazing Fast** -- Rust-powered core delivers ~100x speedup over pure Python
- **Zero Dependencies** -- No Pillow, no NumPy required at runtime
- **Cross-Platform** -- Prebuilt wheels for Linux, macOS, Windows, and more
- **Simple API** -- Two functions, one CLI, that just work
- **Battle-Tested** -- Used in production by developers worldwide

.. toctree::
   :maxdepth: 2
   :caption: Table of Contents
   :hidden:

   installation
   usage
   api
   cli
   benchmarks
   differences
   api_differences
   mmcq
   roadmaps
   license

.. rubric:: Indices and Tables

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`

.. _Rust: https://www.rust-lang.org
.. _PyO3: https://github.com/PyO3/PyO3
