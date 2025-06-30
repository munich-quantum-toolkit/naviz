"""Nox sessions."""

from __future__ import annotations

import argparse

import nox

nox.needs_version = ">=2024.3.2"
nox.options.default_venv_backend = "uv"


@nox.session(reuse_venv=True)
def docs(session: nox.Session) -> None:
    """Build the docs. Use "--non-interactive" to avoid serving. Pass "-b linkcheck" to check links."""
    parser = argparse.ArgumentParser()
    parser.add_argument("-b", dest="builder", default="html", help="Build target (default: html)")
    args, posargs = parser.parse_known_args(session.posargs)

    serve = args.builder == "html" and session.interactive
    if serve:
        session.install("sphinx-autobuild")

    env = {"UV_PROJECT_ENVIRONMENT": session.virtualenv.location}
    # install build and docs dependencies on top of the existing environment
    session.run(
        "uv",
        "sync",
        "--inexact",
        "--only-group",
        "build",
        "--only-group",
        "docs",
        env=env,
    )

    shared_args = [
        "-n",  # nitpicky mode
        "-T",  # full tracebacks
        f"-b={args.builder}",
        "docs",
        f"docs/_build/{args.builder}",
        *posargs,
    ]

    session.run(
        "uv",
        "run",
        "--no-dev",  # do not auto-install dev dependencies
        "--no-build-isolation-package",
        "mqt-naviz",  # build the project without isolation
        "sphinx-autobuild" if serve else "sphinx-build",
        *shared_args,
        env=env,
    )
