import os
import sys

# Point Sphinx at the generated bindings so autodoc can import the module
sys.path.insert(0, os.path.abspath("../src/coinswap/native/linux-x86_64"))

project = "Coinswap Python"
copyright = "2026, Citadel-Tech"
author = "Citadel-Tech"

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",
]

# Napoleon settings (for Google-style docstrings in the generated bindings)
napoleon_google_docstrings = True
napoleon_numpy_docstrings = False

# Autodoc settings
autodoc_member_order = "bysource"
autodoc_default_options = {
    "members": True,
    "undoc-members": True,
    "show-inheritance": True,
    "exclude-members": "_uniffi_clone_handle, _uniffi_make_instance",
}

# Theme
html_theme = "furo"
html_title = "Coinswap Python API"
html_theme_options = {
    "source_repository": "https://github.com/citadel-tech/coinswap-ffi/",
    "source_branch": "main",
    "source_directory": "coinswap-python/docs/",
}

# Suppress warnings about missing native library during doc generation
autodoc_mock_imports = []
