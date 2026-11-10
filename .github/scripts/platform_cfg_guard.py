#!/usr/bin/env python3
"""Warning-only guard for new platform cfg branches (task 20261104060604, GH #854).

Scans the PR diff over `tauri/src` for ADDED lines that introduce cfg-based
platform branches (`cfg(target_os`, `cfg(mobile)`, `cfg(desktop)`) and checks
whether the touched file carries any sign of per-platform test coverage
(a `#[cfg(test)]` module, an `#[rstest]` table, `#[test]`s or a `mod tests`).
Files without it get a `::warning::` GitHub annotation — informational,
never blocking (the #701 class: a platform branch that no Linux CI runner
can see, while the platform it targets ships dead).

Known blind spots, accepted in the task body (NOTICED BUT NOT TOUCHING):
- `cfg!()` macro decisions (`cfg!(target_os = ...)`) are not detected — the
  pure-routing-table pattern replaces them instead of policing them;
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

CFG_LINE_RE = re.compile(r"cfg\((?:target_os|mobile|desktop)\b")
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
    """Files with ADDED cfg-branch lines, plus the line count per file."""
    current_file = None
    counts: dict[str, int] = {}
    for line in lines:
        if line.startswith("+++ b/"):
            current_file = line[len("+++ b/") :]
        elif line.startswith("+") and not line.startswith("+++"):
            if current_file and CFG_LINE_RE.search(line):
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
