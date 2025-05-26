# Installation

MQT NAViz is mainly developed as a Rust library with Python bindings.
The resulting Python package is available on [PyPI](https://pypi.org/project/mqt.naviz/) and can be installed on all major operating systems using all modern Python versions.

:::::{tip}
We highly recommend using [`uv`](https://docs.astral.sh/uv/) for working with Python projects.
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

::::

Check out their excellent [documentation](https://docs.astral.sh/uv/) for more information.

:::::

::::{tab-set}
:sync-group: installer

:::{tab-item} uv _(recommended)_
:sync: uv

```console
$ uv pip install mqt.naviz
```

:::

:::{tab-item} pip
:sync: pip

```console
(.venv) $ python -m pip install mqt.naviz
```

:::
::::

In most practical cases (under 64-bit Linux, MacOS incl. Apple Silicon, and Windows), this requires no compilation and merely downloads and installs a platform-specific pre-built wheel.

Once installed, you can check if the installation was successful by running:

<!-- todo: adapt the following code -->
```console
(.venv) $ python -c "import mqt.naviz; print(mqt.core.__version__)"
```

which should print the installed version of the library.

## Building from source for performance

To build NAViz, an installation of [Rust](https://www.rust-lang.org/learn/get-started) is needed.
The library is continuously tested under Linux, MacOS, and Windows using the [latest available system versions for GitHub Actions](https://github.com/actions/virtual-environments).
In order to access the latest build logs, visit the [GitHub Actions page](https://github.com/cda-tum/mqt-naviz/actions/workflows/ci.yml).

### Native

To build the native version of NAViz, `cargo build --release` can be executed in the project root, which will build a release version.
After the build is finished, the NAViz binary can be found in `target/release/naviz-gui`.

### Python

This package uses [`maturin`](https://github.com/PyO3/maturin) to export this crate as a python wheel.
The wheel can be built using `maturin build` or alternatively `maturin develop` for faster development-build.
For more information on [`maturin`](https://github.com/PyO3/maturin) and the difference between the build commands, see [`maturin`'s README](https://github.com/PyO3/maturin?tab=readme-ov-file#maturin).

### Web

To build the web version of NAViz, the rust compiler for the `wasm32-unknown-unknown`-target needs to be installed.
If Rust was installed using [rustup](https://rustup.rs/), this can be achieved by running `rustup target add wasm32-unknown-unknown`.
Afterward, [`trunk`](https://trunkrs.dev/) needs to be installed using `cargo install trunk.`

After all build tools and compilers are installed, the web version of NAViz can be built by running `trunk build --release`
in [`gui`](./gui).
After the build is finished, the NAViz web version can be found in `gui/dist` and can be deployed to a web server of choice.

### Web (Docker)

Alternatively, a container can be built for the web version of NAViz using the provided [`Dockerfile`](./Dockerfile).
To build the container, simply run `docker build -t naviz .` (assuming [`docker`](https://www.docker.com/) is installed).

The docker container can then be run using `docker run -d -p 8080:80 naviz`, which will start the web server on port `8080`.

## Integrating MQT NAViz into your project

If you want to use the MQT Core Python package in your own project, you can simply add it as a dependency in your `pyproject.toml` or `setup.py` file.
This will automatically install the MQT Core package when your project is installed.

::::{tab-set}

:::{tab-item} uv _(recommended)_

```console
$ uv add mqt.core
```

:::

:::{tab-item} pyproject.toml

```toml
[project]
# ...
dependencies = ["mqt.core>=3.0.0"]
# ...
```

:::

:::{tab-item} setup.py

```python
from setuptools import setup

setup(
    # ...
    install_requires=["mqt.core>=3.0.0"],
    # ...
)
```

:::
::::
