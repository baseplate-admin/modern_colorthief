============
 Installation
============

.. rubric:: Requirements

.. list-table::
   :widths: 30 70
   :header-rows: 0

   * - **Python**
     - 3.10 or newer
   * - **Operating System**
     - Linux, macOS, Windows (prebuilt wheels available)

.. rubric:: Install via pip

.. code-block:: bash

   pip install modern_colorthief

.. rubric:: Install via Poetry

.. code-block:: bash

   poetry add modern_colorthief

.. rubric:: Install via uv

.. code-block:: bash

   uv pip install modern_colorthief

.. tip::

   Use ``uv`` for the fastest installation experience. It resolves and
   installs dependencies up to 10x faster than pip.

.. rubric:: Verify Installation

.. code-block:: python

   >>> import modern_colorthief
   >>> print(modern_colorthief.__version__)
   0.2.1

.. note::

   If you encounter platform-specific build errors, ensure your system has a
   C compiler and the latest Rust toolchain installed. Prebuilt wheels cover
   most common platforms.

.. rubric:: Multi-Language Installation

Modern Colorthief provides native bindings for multiple languages.  Use
``sphinx-tabs`` to switch between them:

.. tabs::

   .. code-tab:: py Python

      .. code-block:: bash

         pip install modern_colorthief

   .. code-tab:: rb Ruby

      .. code-block:: bash

         gem install modern_colorthief

   .. code-tab:: js Node.js

      .. code-block:: bash

         npm install modern-colorthief

   .. code-tab:: bash Java

      .. code-block:: bash

         # Download the JAR from GitHub Releases
         # Add to your classpath or Maven/Gradle dependencies

   .. code-tab:: php PHP

      .. code-block:: bash

         # Build and install as a PHP extension
         # Add to php.ini: extension=modern_colorthief.so

   .. code-tab:: js WebAssembly

      .. code-block:: bash

         npm install colorthief-wasm

.. seealso::

   :doc:`usage` -- Once installed, see how to use the library.

   :doc:`api_multilang` -- Multi-language API reference and examples.
