"""Pytest bootstrap: make ``scripts/`` importable so test modules can import
``deploy_cdn``, ``_cdn_cache``, and other sibling modules the same way the
CLI entry points do (plain ``from deploy_cdn import ...``).

Without this, pytest's rootdir insertion would only expose ``tests/`` itself,
not the sibling script modules under ``scripts/``.

Also makes ``.github/scripts/`` importable for CI-helper tests (e.g. the ASC
release-type script of issue #730), mirroring the ``python3
.github/scripts/<script>.py`` invocation where sys.path[0] is the script's
directory.
"""

from __future__ import annotations

import sys
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parents[2]
_SCRIPTS_DIR = _REPO_ROOT / "scripts"
_GH_SCRIPTS_DIR = _REPO_ROOT / ".github" / "scripts"
for _dir in (_SCRIPTS_DIR, _GH_SCRIPTS_DIR):
    if str(_dir) not in sys.path:
        sys.path.insert(0, str(_dir))
