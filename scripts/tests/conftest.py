"""Pytest bootstrap: make ``scripts/`` importable so test modules can import
``validate_grammar``, ``merge_grammar_chunks``, and ``fix_grammar_content``
the same way the CLI entry points do (plain ``from validate_grammar import ...``).

Without this, pytest's rootdir insertion would only expose ``tests/`` itself,
not the sibling script modules under ``scripts/``.
"""

from __future__ import annotations

import sys
from pathlib import Path

_SCRIPTS_DIR = Path(__file__).resolve().parent.parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))
