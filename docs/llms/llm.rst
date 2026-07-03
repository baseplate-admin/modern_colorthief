.. _llm-files:

====================
LLM Context Files
====================

Modern Colorthief ships with structured context files optimized for feeding to
LLM-based code assistants.  Each file describes the API surface, types, and
usage patterns for one language binding.

Why per-language files?
   Full API context can exceed 200 KB.  Per-language files keep the prompt
   small and focused — only the binding the model actually needs.

Files
-----

Full API
~~~~~~~~

.. rst-class:: llm-file

   ``llms-full.txt`` — Complete API documentation for all bindings
   (architecture, common API, GPU, performance notes).

Per-Language API
~~~~~~~~~~~~~~~~

.. rst-class:: llm-file

   ``llms-python.txt`` — Python (PyO3) binding API
   ``llms-nodejs.txt`` — Node.js (NAPI) binding API
   ``llms-wasm.txt``   — WASM (wasm-bindgen) binding API
   ``llms-java.txt``   — JVM (JNI) binding API
   ``llms-ruby.txt``   — Ruby (rb-sys/magnus) binding API
   ``llms-php.txt``    — PHP (ext-php-rs) binding API

All files live under ``docs/llms/`` in the repository root.

How they're generated
~~~~~~~~~~~~~~~~~~~~

Every file is produced by a static-analysis script that reads the Rust source
and extracts function signatures, parameters, return types, and doc comments.

.. code-block:: bash

   # Regenerate all per-language files
   python docs/scripts/generate_llms_per_lang.py

   # Verify files are up to date (exits 1 if stale)
   python docs/scripts/generate_llms_per_lang.py --check

The generator lives at ``docs/scripts/generate_llms_per_lang.py``.

Using with Claude Code
~~~~~~~~~~~~~~~~~~~~~~

Point Claude at the relevant file for focused, accurate answers:

.. code-block:: text

   Read docs/llms/llms-python.txt and answer how to call get_palette from Python.

For a full multi-language overview, use ``docs/llms/llms-full.txt``.

Using with other LLMs
~~~~~~~~~~~~~~~~~~~~~

Include the appropriate ``llms-<lang>.txt`` in your system prompt or
retrieval context.  Files are plain Markdown, ~1–2 KB each, and designed to
be consumed as-is without preprocessing.
