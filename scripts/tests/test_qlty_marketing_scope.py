"""Regression pins for the qlty scan scope of the nested marketing repo (uwuwu#412).

`qlty check --all` used to exit 3 in a local master checkout: prettier could not
parse generator-owned bytes under `marketing/` (an HTML skeleton with `<TOTAL>`
placeholders and a JSON dump with a captured stderr line). The fix adds one
root-anchored `marketing/**` rule to `.qlty/qlty.toml`; it excludes the whole
nested git repository, not just those two files.

Two layers below: a pure config-contract guard that runs in the pytest CI job,
and a real qlty subprocess A/B that runs whenever the qlty binary is available
(the CI pytest job does not install it). The A/B builds its own throwaway git
contour — it never pins the local `~/origa` checkout or its untracked artifacts.
"""

from __future__ import annotations

import re
import shutil
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
QLTY_TOML = REPO_ROOT / ".qlty" / "qlty.toml"
QLTY = shutil.which("qlty")
NESTED_RULE = "marketing/**"
BAD_HTML = "<div><span>unclosed</div></span>\n"

# Files the nested repo actually ships, plus a hypothetical future generator
# output: the rule must cover the class, not just today's two paths.
ARTIFACTS = (
    "marketing/.opencode/skills/changelog-video/examples/master-skeleton.html",
    "marketing/vi/tools/slice_bing_abs_2026-10-12.json",
    "marketing/templates/ph-s3.html",
    "marketing/vi/tools/generated-next-dump.json",
)


def active_exclude_patterns() -> list[str]:
    """String literals of the `exclude_patterns` array in `.qlty/qlty.toml`.

    Deliberately narrow: comments are stripped before scanning, so names and
    prose in the rationale cannot be mistaken for rules. A format change that
    breaks this reader fails loudly instead of silently passing with no rules.
    """
    lines = QLTY_TOML.read_text(encoding="utf-8").splitlines()
    start = next(
        (
            i
            for i, line in enumerate(lines)
            if re.match(r"^exclude_patterns\s*=\s*\[", line)
        ),
        None,
    )
    assert start is not None, "exclude_patterns block not found in .qlty/qlty.toml"

    patterns: list[str] = []
    for line in lines[start:]:
        code = line.split("#", 1)[0]
        patterns.extend(re.findall(r'"([^"]*)"', code))
        if "]" in code:
            break
    assert patterns, "exclude_patterns parsed empty — the config reader drifted"
    return patterns


def matches(pattern: str, path: str) -> bool:
    """Minimal glob semantics for the forms this config uses.

    `**` spans directories, `*` and `?` do not cross `/`. Deliberately not a
    general gitignore implementation; unsupported forms simply do not match
    and are caught by the explicit assertions below.
    """
    out: list[str] = []
    i = 0
    while i < len(pattern):
        char = pattern[i]
        if char == "*" and i + 1 < len(pattern) and pattern[i + 1] == "*":
            out.append(".*")
            i += 2
        elif char == "*":
            out.append("[^/]*")
            i += 1
        elif char == "?":
            out.append("[^/]")
            i += 1
        else:
            out.append(re.escape(char))
            i += 1
    return re.fullmatch("".join(out), path) is not None


def covering_patterns(path: str, patterns: list[str]) -> list[str]:
    return [pattern for pattern in patterns if matches(pattern, path)]


def test_nested_marketing_artifacts_are_excluded() -> None:
    patterns = active_exclude_patterns()
    assert NESTED_RULE in patterns, (
        "the root-anchored nested-repo rule is missing from exclude_patterns; "
        f"active rules: {patterns}"
    )
    for path in ARTIFACTS:
        covering = covering_patterns(path, patterns)
        assert covering, f"{path} is scanned by qlty — no exclude rule covers it"
        assert NESTED_RULE in covering, (
            f"{path} is excluded only by {covering}, not by the nested-repo rule"
        )


def test_nested_rule_does_not_swallow_tracked_or_docs_paths() -> None:
    patterns = active_exclude_patterns()
    for path in (
        "origa/src/lib.rs",
        "tauri/tauri.conf.json",
        "docs/marketing/notes/keep.json",
    ):
        assert not covering_patterns(path, patterns), (
            f"{path} must stay in qlty scope but is excluded by {covering_patterns(path, patterns)}"
        )
    assert not matches(NESTED_RULE, "docs/marketing/notes/keep.json"), (
        "marketing/** must be root-anchored, not a recursive '**/marketing/**'"
    )


def test_existing_docs_marketing_exceptions_are_intact() -> None:
    patterns = active_exclude_patterns()
    for prefix in (
        "docs/marketing/playbooks/",
        "docs/marketing/research/",
        "docs/marketing/strategies/",
    ):
        assert prefix + "**" in patterns, (
            f"the tracked marketing exception {prefix}** was lost"
        )


def _run_contour(config: str, tmp_path: Path, name: str) -> str:
    contour = tmp_path / name
    (contour / ".qlty").mkdir(parents=True)
    (contour / ".qlty" / "qlty.toml").write_text(config, encoding="utf-8")
    (contour / "marketing" / "gen").mkdir(parents=True)
    (contour / "marketing" / "gen" / "bad.html").write_text(BAD_HTML, encoding="utf-8")
    # A control outside the excluded tree: qlty must still report it, otherwise
    # a green result would only prove that this contour was scanned with nothing.
    (contour / "root_bad.html").write_text(BAD_HTML, encoding="utf-8")
    subprocess.run(["git", "init", "-q"], cwd=contour, check=True)
    subprocess.run(["git", "add", "-A"], cwd=contour, check=True)
    subprocess.run(
        ["git", "-c", "user.email=t@t", "-c", "user.name=t", "commit", "-qm", "init"],
        cwd=contour,
        check=True,
    )
    result = subprocess.run(
        [
            str(QLTY),
            "check",
            "--all",
            "--no-fix",
            "--filter",
            "prettier",
            "--no-progress",
        ],
        cwd=contour,
        capture_output=True,
        text=True,
        timeout=180,
    )
    return result.stdout + result.stderr


def test_qlty_red_green_on_a_self_contained_contour(tmp_path: Path) -> None:
    if QLTY is None:
        # The pytest CI job does not install qlty; the config-contract guards
        # above run there. Run this test locally/on a qlty-equipped runner.
        return
    config = QLTY_TOML.read_text(encoding="utf-8")
    assert NESTED_RULE in config, "test fixture: nested rule missing from the config"

    fixed = _run_contour(config, tmp_path, "fixed")
    assert "marketing/gen/bad.html" not in fixed, (
        f"qlty still scans the nested marketing tree:\n{fixed}"
    )
    assert "root_bad.html" in fixed, (
        f"qlty did not scan the out-of-scope control:\n{fixed}"
    )

    broken = _run_contour(
        config.replace(f'    "{NESTED_RULE}",\n', ""), tmp_path, "broken"
    )
    assert "marketing/gen/bad.html" in broken, (
        "red control failed: removing the rule did not bring the artifact back "
        f"into prettier scope — the green result above is not caused by the rule:\n{broken}"
    )
