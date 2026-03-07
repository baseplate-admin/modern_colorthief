# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information
import modern_colorthief
import datetime

project = "Modern Colorthief"
author = "baseplate-admin"
copyright = f"2024-{datetime.date.today().year}, {author}"
release = modern_colorthief.__version__

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    "myst_parser",
    "sphinx.ext.autodoc",
    "sphinx.ext.intersphinx",
    "sphinx.ext.viewcode",
    "sphinx_js",
]
templates_path = ["_templates"]
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]
source_suffix = {
    ".rst": "restructuredtext",
    ".md": "markdown",
}
autodoc_typehints = "description"

# -- sphinx-js (WASM / TypeScript API) --------------------------------------
js_language = "typescript"
# Path to the TypeScript source that documents the WASM API,
# relative to this conf.py file.
js_source_path = "../src/wasm"
# Point sphinx-js at the typedoc.json at the project root.
jsdoc_config_path = "../typedoc.json"
# TypeScript compiler settings for TypeDoc.
jsdoc_tsconfig_path = "../tsconfig.json"
# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = "shibuya"
html_static_path = ["_static"]
