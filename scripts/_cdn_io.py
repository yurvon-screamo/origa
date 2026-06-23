"""Shared I/O helpers for cdn/ mutation scripts.

The `cdn/` directory is gitignored (see `.gitignore:83`), so a half-written
file cannot be recovered from version control. Every script that mutates CDN
artifacts should route its writes through :func:`atomic_write_json` to get
tempfile + `os.replace` semantics: a crash mid-write leaves the destination
file untouched.

Usage::

    from _cdn_io import atomic_write_json
    atomic_write_json(path, payload)
"""

from __future__ import annotations

import json
import os
import tempfile
from pathlib import Path
from typing import Any


def atomic_write_json(
    path: Path,
    payload: Any,
    *,
    ensure_ascii: bool = False,
    compact: bool = True,
) -> None:
    """Write `payload` as JSON to `path` atomically.

    Writes to a NamedTemporaryFile in the same directory, then `os.replace`s
    into place. `os.replace` is atomic on POSIX and on Windows for files on
    the same volume. A crash before the replace leaves the original file
    untouched; a crash after leaves the new file in place.

    `compact=True` produces byte-stable output (`separators=(",", ":")`,
    no indent) suitable for hashing. `compact=False` produces indented JSON
    for human-readable artifacts.
    """
    separators = (",", ":") if compact else None
    indent = None if compact else 2
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp_name = tempfile.mkstemp(
        dir=path.parent,
        prefix=f".{path.name}.",
        suffix=".tmp",
    )
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            json.dump(
                payload,
                f,
                ensure_ascii=ensure_ascii,
                separators=separators,
                indent=indent,
            )
        os.replace(tmp_name, path)
    except BaseException:
        # Best-effort cleanup of the temp file on any failure; ignore errors
        # because we're already in an error path.
        try:
            os.unlink(tmp_name)
        except OSError:
            pass
        raise
