"""Coinswap Python bindings package."""

from __future__ import annotations

from importlib import import_module


def _load_bindings():
    try:
        return import_module(".coinswap", __name__)
    except Exception as exc:  # pragma: no cover - simple import shim
        raise ImportError(
            "Coinswap bindings are not generated. Run ffi-commons/create_bindings.sh "
            "to generate the Python bindings (coinswap.py and native libraries) and "
            "reinstall the package."
        ) from exc


_bindings = _load_bindings()

_export_names = getattr(_bindings, "__all__", None)
if _export_names is None:
    _export_names = [
        _name
        for _name in dir(_bindings)
        if not _name.startswith("_")
    ]

for _name in _export_names:
    globals()[_name] = getattr(_bindings, _name)

__all__ = list(_export_names)
