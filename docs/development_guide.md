# Development Guide

Ready to contribute to the project? This guide will get you started.

## Initial Setup

1. Get the code

   ::::{tab-set}
   :::{tab-item} External Contribution
   If you do not have write access to the [cda-tum/mqt-naviz](https://github.com/cda-tum/mqt-naviz) repository,
   fork the repository on GitHub (see <https://docs.github.com/en/get-started/quickstart/fork-a-repo>)
   and clone your fork locally.

   ```console
   $ git clone git@github.com:your_name_here/mqt-naviz.git mqt-naviz
   ```

   :::
   :::{tab-item} Internal Contribution
   If you do have write access to the [cda-tum/mqt-naviz](https://github.com/cda-tum/mqt-naviz) repository,
   clone the repository locally.

   ```console
   $ git clone git@github.com/cda-tum/mqt-naviz.git mqt-naviz
   ```

   :::
   ::::

2. Change into the project directory

   ```console
   $ cd mqt-naviz
   ```

3. Create a branch for local development

   ```console
   $ git checkout -b name-of-your-bugfix-or-feature
   ```

   Now you can make your changes locally.

4. If you plan to [work on the Python package](#working-on-the-python-package), we highly recommend using [`uv`](https://docs.astral.sh/uv/).
   It is an extremely fast Python package and project manager, written in Rust and developed by [Astral](https://astral.sh/) (the same team behind [`ruff`](https://docs.astral.sh/ruff/)).
   It can act as a drop-in replacement for `pip` and `virtualenv`, and provides a more modern and faster alternative to the traditional Python package management tools.
   It automatically handles the creation of virtual environments and the installation of packages, and is much faster than `pip`.
   Additionally, it can even set up Python for you if it is not installed yet.

   If you do not have `uv` installed yet, you can install it via:

   ::::{tab-set}
   :::{tab-item} macOS and Linux

   ```console
   $ curl -LsSf https://astral.sh/uv/install.sh | sh
   ```

   :::
   :::{tab-item} Windows

   ```console
   $ powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"
   ```

   :::
   ::::
   Check out their excellent [documentation](https://docs.astral.sh/uv/) for more information.

5. We also highly recommend to install and set up [pre-commit](https://pre-commit.com/) to automatically
   run a set of checks before each commit.

   ::::{tab-set}
   :::{tab-item} via `uv`
   :sync: uv
   The easiest way to install pre-commit is via [uv](https://docs.astral.sh/uv/).

   ```console
   $ uv tool install pre-commit
   ```

   :::
   :::{tab-item} via `brew`
   :sync: brew
   If you use macOS, then pre-commit is in Homebrew, and you can use

   ```console
   $ brew install pre-commit
   ```

   :::
   :::{tab-item} via `pipx`
   :sync: pipx
   If you prefer to use [pipx](https://pypa.github.io/pipx/), you can install pre-commit with

   ```console
   $ pipx install pre-commit
   ```

   :::
   :::{tab-item} via `pip`
   :sync: pip
   If you prefer to use regular `pip` (preferably in a virtual environment), you can install pre-commit with

   ```console
   $ pip install pre-commit
   ```

   :::
   ::::

   Afterwards, you can install the pre-commit hooks with

   ```console
   $ pre-commit install
   ```

## Working on the Rust library

Building the project requires a Rust compiler.
As of 2025, our CI pipeline on GitHub continuously tests the library under the following matrix of systems and compilers:

- Ubuntu 24.04 on x86_64 and arm64
- Ubuntu 22.04 on x86_64 and arm64
- macOS 13 on x86_64
- macOS 14 on arm64
- macOS 15 on arm64
- Windows 2022 on x86_64
- Windows 2025 on x86_64

To access the latest build logs, visit the [GitHub Actions page](https://github.com/cda-tum/mqt-naviz/actions/workflows/ci.yml).

We are not aware of any issues with other compilers or operating systems.
If you encounter any problems, please [open an issue](https://github.com/cda-tum/mqt-naviz/issues) and let us know.

### Configure and Build

To build NAViz, an installation of [Rust](https://www.rust-lang.org/learn/get-started) is needed.

#### Native

To build the native version of NAViz, `cargo build --release` can be executed in the project root, which will build a release version.
After the build is finished, the NAViz binary can be found in `target/release/naviz-gui`.

#### Web

To build the web version of NAViz, the rust compiler for the `wasm32-unknown-unknown`-target needs to be installed.
If Rust was installed using [rustup](https://rustup.rs/), this can be achieved by running `rustup target add wasm32-unknown-unknown`.
Afterward, [`trunk`](https://trunkrs.dev/) needs to be installed using `cargo install trunk.`

After all build tools and compilers are installed, the web version of NAViz can be built by running `trunk build --release`
in [`gui`](./gui).
After the build is finished, the NAViz web version can be found in `gui/dist` and can be deployed to a web server of choice.

#### Web (Docker)

Alternatively, a container can be built for the web version of NAViz using the provided [`Dockerfile`](./Dockerfile).
To build the container, simply run `docker build -t naviz .` (assuming [`docker`](https://www.docker.com/) is installed).

The docker container can then be run using `docker run -d -p 8080:80 naviz`, which will start the web server on port `8080`.

### Rust Testing and Code Coverage

<!-- todo -->

### C++ Code Formatting and Linting

<!-- todo -->

Our pre-commit configuration also includes clang-format.
If you have installed pre-commit, it will automatically run clang-format on your code before each commit.
If you do not have pre-commit setup, the [pre-commit.ci](https://pre-commit.ci) bot will run clang-format on your code
and automatically format it according to the style guide.

:::{tip}
Remember to pull the changes back into your local repository after the bot has formatted your code to avoid merge conflicts.
:::

Our CI pipeline will also run clang-tidy over the changes in your pull request and report any issues it finds.
Due to technical limitations, the workflow can only post pull request comments if the changes are not coming from a fork.
If you are working on a fork, you can still see the clang-tidy results either in the GitHub Actions logs,
on the workflow summary page, or in the "Files changed" tab of the pull request.

### Rust Documentation

<!-- todo -->

## Working on the Python package

### Building the Python package

This package uses [`maturin`](https://github.com/PyO3/maturin) to export this crate as a python wheel.
The wheel can be built using `maturin build` or alternatively `maturin develop` for faster development-build.
For more information on [`maturin`](https://github.com/PyO3/maturin) and the difference between the build commands, see [`maturin`'s README](https://github.com/PyO3/maturin?tab=readme-ov-file#maturin).

### Running Python Tests

The Python part of the code base is tested by unit tests using the [pytest](https://docs.pytest.org/en/latest/) framework.
The corresponding test files can be found in the {code}`test/python` directory.

<!-- todo -->

### Python Code Formatting and Linting

The Python code is formatted and linted using a collection of [pre-commit hooks](https://pre-commit.com/).
This collection includes:

- [ruff](https://docs.astral.sh/ruff/) -- an extremely fast Python linter and formatter, written in Rust.
- [mypy](https://mypy-lang.org/) -- a static type checker for Python code

There are two ways of using these hooks:

- You can install the hooks manually by running

  ```console
  $ pre-commit install
  ```

  in the project root directory.
  This will install the hooks in the {code}`.git/hooks` directory of the repository.
  The hooks will then be executed automatically when committing changes.

- You can use the {code}`nox` session {code}`lint` to run the hooks manually.

  ```console
  $ nox -s lint
  ```

  :::{note}
  If you don't want to use {code}`nox`, you can also run the hooks directly using {code}`pre-commit`.

  ```console
  $ pre-commit run --all-files
  ```

  :::

### Python Documentation

The Python part of the code base is documented using [Google-style docstrings](https://google.github.io/styleguide/pyguide.html#s3.8-comments-and-docstrings).
Every public function, class, and module should have a docstring that explains what it does and how to use it.
Ruff will check for missing docstrings and will explicitly warn you if you forget to add one.

We heavily rely on [type hints](https://docs.python.org/3/library/typing.html) to document the expected types of function arguments and return values.
For the compiled parts of the code base, we provide type hints in the form of stub files in the {code}`src/mqt/core` directory.

The Python API documentation is integrated into the overall documentation that we host on ReadTheDocs using the
[sphinx-autoapi](https://sphinx-autoapi.readthedocs.io/en/latest/) extension for Sphinx.

## Working on the Documentation

The documentation is written in [MyST](https://myst-parser.readthedocs.io/en/latest/index.html) (a flavour of Markdown) and built using [Sphinx](https://www.sphinx-doc.org/en/master/).
The documentation source files can be found in the {code}`docs/` directory.

On top of the API documentation, we provide a set of tutorials and examples that demonstrate how to use the library.
These are written in Markdown using [myst-nb](https://myst-nb.readthedocs.io/en/latest/), which allows to execute Python code blocks in the documentation.
The code blocks are executed during the documentation build process, and the output is included in the documentation.
This allows us to provide up-to-date examples and tutorials that are guaranteed to work with the latest version of the library.

You can build the documentation using the {code}`nox` session {code}`docs`.

```console
$ nox -s docs
```

This will install all dependencies for building the documentation in an isolated environment, build the Python package, and then build the documentation.
Finally, it will host the documentation on a local web server for you to view.

:::{note}
If you don't want to use {code}`nox`, you can also build the documentation directly using {code}`sphinx-build`.

```console
(.venv) $ sphinx-build -b html docs/ docs/_build
```

The docs can then be found in the {code}`docs/_build` directory.
:::
