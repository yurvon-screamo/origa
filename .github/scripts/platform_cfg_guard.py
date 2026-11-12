#!/usr/bin/env python3
"""Warning-only guard for new platform cfg branches (task 20261104060604, GH #854).

Scans the PR diff over `tauri/src` for ADDED lines that introduce cfg-based
platform branches: a `cfg(` attribute/expression carrying a platform token
(`target_os =`, `target_family =`, `mobile`, `desktop`, `windows`, `unix`).
All wrapper shapes are covered — bare (`#[cfg(target_os = "linux")]` and
short `#[cfg(windows)]`), `cfg(any(...))`, `cfg(all(...))`, `cfg(not(...))`
and their nesting; `cfg!(...)` macro decisions are NOT matched (the `!`
breaks the `cfg(` prefix) — an accepted blind spot, see the task body's
not-goals. It checks whether the touched file carries any sign of
per-platform test coverage (a `#[cfg(test)]` module, an `#[rstest]` table,
`#[test]`s or a `mod tests`). Files without it get a `::warning::` GitHub
annotation — informational, never blocking (the #701 class: a platform
branch that no Linux CI runner can see, while the platform it targets
ships dead).

Other accepted blind spots (task body, NOTICED BUT NOT TOUCHING):
- a plain `invoke` of a platform-scoped plugin without any cfg is not
  detected (too many false positives).

Override the compared base with PLATFORM_GUARD_BASE (local demo runs).
"""

import os
import re
import subprocess
import sys

BASE = os.environ.get("PLATFORM_GUARD_BASE", "origin/master")
DIFF_PATHS = ["tauri/src"]

# `cfg(` but not `cfg!(` — the macro form is a declared blind spot.
CFG_LINE_RE = re.compile(r"\bcfg\(")
PLATFORM_TOKEN_RE = re.compile(
    r"target_os\s*=|target_family\s*=|\b(?:mobile|desktop|windows|unix)\b"
)
TEST_SIGN_RE = re.compile(
    r"#\[cfg\(test\)\]|#\[rstest\]|#\[test\]|mod tests|wasm_bindgen_test"
)


def diff_lines() -> list[str]:
    """Added/removed-line diff against the merge-base, unified to 0."""
    result = subprocess.run(
        ["git", "diff", "--unified=0", f"{BASE}...HEAD", "--", *DIFF_PATHS],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        # A broken diff probe is not a finding — fail loudly so the broken
        # run is visible instead of silently producing zero warnings.
        print(f"::error::platform_cfg_guard: git diff failed: {result.stderr.strip()}")
        sys.exit(1)
    return result.stdout.splitlines()


def added_cfg_paths(lines: list[str]) -> dict[str, int]:
    """Files with ADDED cfg-branch lines, plus the line count per file.

    A line counts when it both opens a `cfg(` (attribute or expression, any
    any/all/not wrapper shape, short `windows`/`unix` forms included) and
    carries a platform token. `cfg!(...)` macros never match — the `!`
    breaks the `cfg(` prefix, keeping the declared macro blind spot honest.
    """
    current_file = None
    counts: dict[str, int] = {}
    for line in lines:
        if line.startswith("+++ b/"):
            current_file = line[len("+++ b/") :]
        elif line.startswith("+") and not line.startswith("+++"):
            if (
                current_file
                and CFG_LINE_RE.search(line)
                and PLATFORM_TOKEN_RE.search(line)
            ):
                counts[current_file] = counts.get(current_file, 0) + 1
    return counts


def has_test_coverage(path: str) -> bool:
    try:
        with open(path, encoding="utf-8") as handle:
            return bool(TEST_SIGN_RE.search(handle.read()))
    except OSError:
        return False


def main() -> None:
    counts = added_cfg_paths(diff_lines())
    for path, added in sorted(counts.items()):
        if has_test_coverage(path):
            print(
                f"platform_cfg_guard: {path}: {added} added cfg line(s), test coverage present"
            )
            continue
        print(
            f"::warning file={path}::"
            f"{added} added platform cfg line(s) without an in-file test module — "
            f"consider a pure per-platform routing table with an rstest matrix "
            f"(see platform_routing.rs, task 20261104060604 / GH #854)"
        )
    print(f"platform_cfg_guard: done over {BASE} (warning-only, never blocking)")


if __name__ == "__main__":
    main()
