# NAViz - A Visualizer for Neutral Atom Quantum Computers

<div align="center">
	<img src="./gui/rsc/icon.png" width="25%" alt="The NAViz logo">
</div>

## About

NAViz is a tool used to visualize atom movements of a neutral atom quantum computer.
It allows defining different machine architectures and supports importing external formats.
The visualization style can also be customized either by selecting one of the predefined styles or creating a new style.

## Building

To build NAViz,
an installation of [Rust](https://www.rust-lang.org/learn/get-started) is needed.

### Native

To build the native version of NAViz,
`cargo build --release` can be executed in the project root,
which will build a release version.
After the build is finished,
the NAViz binary can be found in `target/release/naviz-gui`.

### Python

Documentation on how to build the python-module
can be found in that crate's [`README`](./python/README.md).

### Web

To build the web-version of NAViz,
the rust compiler for the `wasm32-unknown-unknown`-target needs to be installed.
If Rust was installed using [rustup](https://rustup.rs/),
this can be achieved by running `rustup target add wasm32-unknown-unknown`.
Afterwards,
[`trunk`](https://trunkrs.dev/) needs to be installed using `cargo install trunk`.

After all build-tools and compilers are installed,
the web-version of NAViz can be built by running `trunk build --release`
in [`gui`](./gui).
After the build is finished,
the NAViz web-version can be found in `gui/dist`
and can be deployed to a web-server of choice.

### Web (Docker)

Alternatively,
a container can be built for the web-version of NAViz
using the provided [`Dockerfile`](./Dockerfile).
To build the container,
simply run `docker build -t naviz .`
(assuming [`docker`](https://www.docker.com/) is installed).

The docker container can then be run using `docker run -d -p 8080:80 naviz`,
which will start the web-server on port `8080`.

## Usage

NAViz allows opening `.naviz` instruction files
or importing instructions from external formats such as `mqt na`
under the `File`-menu.
Alternatively,
files can simply be dropped onto or pasted into the application.

The machine and style can be selected from the `Machine` and `Style` menus respectively.
These menus allow selecting a config from the loaded configs
as well as opening or importing a new config.

When the animation plays,
the progress-bar at the bottom of the window can be used to seek through the visualization.

### Python

NAViz can also be built as a python-package.
The usage is documented in python-[`README`](./python/README.md).

## Documentation

The documentation of all NAViz-internal formats
(i.e., all formats that are not imported)
can be found in [`doc/FileFormat.md`](doc/FileFormat.md).

## License

NAViz is released under the GNU AGPL.
Some libraries are licensed under the GNU LGPL instead
to allow more broad use.
If this is the case,
a `LICENSE`-file is included in the crate.
Additionally,
the `license`-field of the crate's `Cargo.toml`
specifies the license for the respective crate.
