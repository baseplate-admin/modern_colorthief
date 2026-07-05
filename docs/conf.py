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

# -- sphinx-tabs compatible builders -----------------------------------------
# https://sphinx-tabs.readthedocs.io/
sphinx_tabs_valid_builders = ["linkcheck"]

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.intersphinx",
    "sphinx.ext.viewcode",
     "sphinx.ext.todo",
    "sphinx_copybutton",
    "sphinx_tabs.tabs",
]

# Render todo items as visible checklists
todo_include_todos = True

templates_path = ["_templates"]
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

# Only RST source files -- no markdown
source_suffix = {
    ".rst": "restructuredtext",
}

autodoc_typehints = "description"

# -- Options for copybutton --------------------------------------------------
# https://sphinx-copybutton.readthedocs.io/

copybutton_prompt_text = r">>> |\.\.\. |\$ |In \\\d+: | {2,5}\\.\.\.: "
copybutton_prompt_is_regexp = True
copybutton_line_continuation_marker = "..."
copybutton_here_doc_delimiter = "EOF"
copybutton_include_children = False
copybutton_selector = "div.highlight"
copybutton_copy_empty_lines = True
copybutton_remove_prompts = True

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = "shibuya"
html_static_path = ["_static"]
html_css_files = [
    "custom.css",
]

# Shibuya theme options -- custom color scheme, layout, and features
html_theme_options = {
    # Branding
    "sidebar_primary_name": "Modern Colorthief",
    "sidebar_hide_name": False,

    # Accent color -- violet matches our indigo-based custom palette
    "accent_color": "violet",

    # Dark code blocks for contrast
    "dark_code": True,

    # Light/dark mode default
    "color_mode": "auto",

    # Layout
    "page_layout": "default",
    "page_width_exceeded": "wrap",
    "code_header_reflow": True,

    # Sidebar icon links
    "icon_links": [
        {
            "name": "GitHub",
            "url": "https://github.com/baseplate-admin/modern_colorthief",
            "icon": "fa-brands fa-github",
            "type": "fontawesome",
        },
        {
            "name": "PyPI",
            "url": "https://pypi.org/project/modern_colorthief",
            "icon": "fa-solid fa-box",
            "type": "fontawesome",
        },
        {
            "name": "Documentation",
            "url": "https://modern-colorthief.readthedocs.io",
            "icon": "fa-solid fa-book",
            "type": "fontawesome",
        },
    ],
    "icon_links_label": "Links",

    # Sidebar spacing
    "sidebar_header_spacing": True,
}

# -- Intersphinx mapping -----------------------------------------------------
intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
}
