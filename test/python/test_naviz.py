"""Tests for the MQT NAViz Python bindings."""

from __future__ import annotations

import pytest

from mqt.naviz import Repository, default_import_settings


def test_machines_repository_provides_example() -> None:
    """The bundled machines repository contains the ``example`` machine."""
    machine = Repository.machines().get("example")
    assert machine is not None


def test_styles_repository_provides_tum() -> None:
    """The bundled styles repository contains the ``tum`` style."""
    style = Repository.styles().get("tum")
    assert style is not None


def test_unknown_repository_entry_is_none() -> None:
    """An unknown ID yields ``None`` rather than raising."""
    assert Repository.machines().get("definitely-not-a-machine") is None


def test_default_import_settings_for_mqt_na() -> None:
    """``default_import_settings`` returns the defaults for the MQT NA format."""
    settings = default_import_settings("MqtNa")
    assert isinstance(settings, dict)
    assert settings["MqtNa"]["atom_prefix"] == "atom"


def test_default_import_settings_rejects_unknown_format() -> None:
    """An unknown import format is rejected."""
    with pytest.raises(RuntimeError, match="unknown variant"):
        default_import_settings("definitely-not-a-format")
