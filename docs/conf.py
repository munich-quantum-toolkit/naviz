"""Sphinx configuration file."""

from __future__ import annotations

from importlib import metadata

# Filter git details from version
version = metadata.version("mqt.naviz")
release = version.split("+")[0]

project = "MQT NAViz"
author = "Chair for Design Automation, TUM & Munich Quantum Software Company"
language = "en"
project_copyright = "2023 - 2025 Chair for Design Automation, TUM & 2025 Munich Quantum Software Company"

master_doc = "index"

templates_path = ["_templates"]

extensions = [
    "myst_nb",
    "autoapi.extension",
    "sphinx.ext.autodoc",
    "sphinx.ext.intersphinx",
    "sphinx.ext.napoleon",
    "sphinx_copybutton",
    "sphinx_design",
    "sphinxext.opengraph",
    "sphinx.ext.viewcode",
    "sphinxcontrib.inkscapeconverter",
    "sphinxcontrib_rust",
]

source_suffix = [".rst", ".md"]

exclude_patterns = [
    "_build",
    "**.ipynb_checkpoints",
    "**.jupyter_cache",
    "**jupyter_execute",
    "Thumbs.db",
    ".DS_Store",
    ".env",
    ".venv",
]

pygments_style = "colorful"

intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
    "numpy": ("https://numpy.org/doc/stable/", None),
    "qiskit": ("https://docs.quantum.ibm.com/api/qiskit", None),
    "mqt": ("https://mqt.readthedocs.io/en/latest", None),
    "core": ("https://mqt.readthedocs.io/projects/core/en/latest", None),
    "ddsim": ("https://mqt.readthedocs.io/projects/ddsim/en/latest", None),
    "qcec": ("https://mqt.readthedocs.io/projects/qcec/en/latest", None),
    "qecc": ("https://mqt.readthedocs.io/projects/qecc/en/latest", None),
    "syrec": ("https://mqt.readthedocs.io/projects/syrec/en/latest", None),
}

myst_enable_extensions = [
    "amsmath",
    "colon_fence",
    "substitution",
    "deflist",
    "dollarmath",
]
myst_substitutions = {
    "version": version,
}
myst_heading_anchors = 3

# -- Options for {MyST}NB ----------------------------------------------------

nb_execution_mode = "cache"
nb_mime_priority_overrides = [
    # builder name, mime type, priority
    ("latex", "image/svg+xml", 15),
]


copybutton_prompt_text = r"(?:\(\.?venv\) )?(?:\[.*\] )?\$ "
copybutton_prompt_is_regexp = True
copybutton_line_continuation_character = "\\"

modindex_common_prefix = ["mqt.naviz."]

autoapi_dirs = ["../python/mqt"]
autoapi_python_use_implicit_namespaces = True
autoapi_root = "api"
autoapi_add_toctree_entry = False
autoapi_options = [
    "members",
    "show-inheritance",
    "show-module-summary",
]
autoapi_keep_files = True
add_module_names = False
toc_object_entries_show_parents = "hide"
python_use_unqualified_type_names = True
napoleon_google_docstring = True
napoleon_numpy_docstring = False

# -- Options for HTML output -------------------------------------------------

html_theme = "furo"
html_static_path = ["_static"]
html_css_files = [
    "custom.css",
    "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0/css/fontawesome.min.css",
    "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0/css/solid.min.css",
    "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0/css/brands.min.css",
]
html_theme_options = {
    "light_logo": "mqt_dark.png",
    "dark_logo": "mqt_light.png",
    "source_repository": "https://github.com/munich-quantum-toolkit/naviz/",
    "source_branch": "main",
    "source_directory": "docs/",
    "navigation_with_keys": True,
    "footer_icons": [
        {
            "name": "GitHub",
            "url": "https://github.com/munich-quantum-toolkit/naviz/",
            "html": "",
            "class": "fa-brands fa-solid fa-github fa-2x",
        },
        {
            "name": "PyPI",
            "url": "https://pypi.org/project/mqt-naviz/",
            "html": "",
            "class": "fa-brands fa-solid fa-python fa-2x",
        },
    ],
}

# -- Options for LaTeX output ------------------------------------------------

numfig = True
numfig_secnum_depth = 0

sd_fontawesome_latex = True
image_converter_args = ["-density", "300"]
latex_engine = "pdflatex"
latex_documents = [
    (
        master_doc,
        "mqt_naviz.tex",
        r"MQT NAViz\\{\Large A tool for Mapping Quantum Circuits to various Hardware Technologies}",
        r"""Chair for Design Automation\\ Technical University of Munich, Germany\\
        \href{mailto:quantum.cda@xcit.tum.de}{quantum.cda@xcit.tum.de}\\
        Munich Quantum Software Company GmbH\\Garching near Munich, Germany""",
        "howto",
        False,
    ),
]
latex_logo = "_static/mqt_dark.png"
latex_elements = {
    "papersize": "a4paper",
    "releasename": "Version",
    "printindex": r"\footnotesize\raggedright\printindex",
    "tableofcontents": "",
    "sphinxsetup": "iconpackage=fontawesome",
    "extrapackages": r"\usepackage{qrcode,graphicx,calc,amsthm,etoolbox,flushend,mathtools}",
    "preamble": r"""
\patchcmd{\thebibliography}{\addcontentsline{toc}{section}{\refname}}{}{}{}
\DeclarePairedDelimiter\abs{\lvert}{\rvert}
\DeclarePairedDelimiter\mket{\lvert}{\rangle}
\DeclarePairedDelimiter\mbra{\langle}{\rvert}
\DeclareUnicodeCharacter{03C0}{$\pi$}
\DeclareUnicodeCharacter{2728}{\faicon{star}}
\DeclareUnicodeCharacter{1F6B8}{\faicon{user-plus}}
\DeclareUnicodeCharacter{1F4DD}{\faicon{book}}
\DeclareUnicodeCharacter{1F69A}{\faicon{truck}}
\DeclareUnicodeCharacter{267B}{\faicon{recycle}}
\DeclareUnicodeCharacter{2B06}{\faicon{arrow-up}}
\DeclareUnicodeCharacter{1F4C4}{\faicon{file-alt}}
\DeclareUnicodeCharacter{1F525}{\faicon{fire}}
\DeclareUnicodeCharacter{1F41B}{\faicon{bug}}
\DeclareUnicodeCharacter{1F4DA}{\faicon{book-open}}
\DeclareUnicodeCharacter{1F4E6}{\faicon{archive}}
\DeclareUnicodeCharacter{23EA}{\faicon{angle-double-left}}
\DeclareUnicodeCharacter{FE0F}{}

\newcommand*{\ket}[1]{\ensuremath{\mket{\mkern1mu#1}}}
\newcommand*{\bra}[1]{\ensuremath{\mbra{\mkern1mu#1}}}
\newtheorem{example}{Example}
\clubpenalty=10000
\widowpenalty=10000
\interlinepenalty 10000
\def\subparagraph{} % because IEEE classes don't define this, but titlesec assumes it's present
""",
    "extraclassoptions": r"journal, onecolumn",
    "fvset": r"\fvset{fontsize=\small}",
    "figure_align": "htb",
}
latex_domain_indices = False
latex_docclass = {
    "howto": "IEEEtran",
}

# -- Options for Rust output -------------------------------------------------

rust_crates = {
    "animator": "animator",
    "bindings": "bindings",
    "renderer": "renderer",
    "repository": "repository",
    "state": "state",
    "video": "video",
}
rust_doc_dir = "docs/crates"
rust_rustdoc_fmt = "md"
