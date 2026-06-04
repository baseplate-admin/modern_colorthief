=========
 Roadmaps
=========

.. rubric:: Completed

- Reading from
  :py:class:`BytesIO <python:io.BytesIO>` objects
  (`#47 <https://github.com/baseplate-admin/modern_colorthief/pull/47>`_)

.. rubric:: Planned

- Making the entire module faster with **PGO** (Profile-Guided
  Optimization), when PyO3 supports it.

  .. todo::

     Track PGO support in `maturin#1840 <https://github.com/PyO3/maturin/issues/1840>`_.

.. rubric:: Future Ideas

- Native support for more image formats without requiring Pillow
- GPU-accelerated color quantization via WebGPU/WGPU bindings
- Batch processing API for analyzing multiple images at once
- Integration with popular Python data science libraries

.. note::

   Priorities may shift based on community feedback. Open an issue on
   `GitHub <https://github.com/baseplate-admin/modern_colorthief/issues>`_
   to suggest features or vote on existing ones.
