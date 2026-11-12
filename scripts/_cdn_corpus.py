"""Local CDN corpus root resolver (shared by content-maintenance scripts).

The corpus is machine-local state outside the repository: canonical path is
``$ORIGA_CDN_DIR``, falling back to ``~/origa-cdn``. One seed per machine —
every checkout and worktree on it sees the same corpus. Mirrors
``origa/src/corpus.rs`` (the contract's source of truth); keep in sync.
"""

from __future__ import annotations

import os
from pathlib import Path


def corpus_dir() -> Path:
    """Local CDN corpus root: ``$ORIGA_CDN_DIR`` or ``~/origa-cdn``."""
    env_dir = os.environ.get("ORIGA_CDN_DIR")
    if env_dir:
        return Path(env_dir)
    return Path.home() / "origa-cdn"
