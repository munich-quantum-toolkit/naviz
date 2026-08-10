"""The MQT NAViz Python bindings."""

from typing import Any, final

@final
class Repository:
    def get(self, /, identifier: str) -> str | None:
        """Get a config entry by ``identifier`` from this repository."""
    @staticmethod
    def machines() -> Repository:
        """Get the machines-repository."""
    @staticmethod
    def styles() -> Repository:
        """Get the styles-repository."""

def default_import_settings(input_format: str) -> dict[str, Any]:
    """Get the default import settings for the specified ``import_format``."""

def export_video(
    input_data: str,
    output: str,
    resolution: tuple[int, int],
    fps: int,
    machine: str,
    style: str,
    import_options: dict | None = None,
) -> None:
    """Export a video from the ``input_data`` to the ``output`` location.

    The video is exported at the specified ``resolution`` with the specified framerate (``fps``).
    When ``import_options`` are specified, the ``input`` is imported from the specified format.
    """
