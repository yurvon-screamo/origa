"""Regression pins for the cache-budget layer of cleanup-cache.yml (task
20261110221612, PR #926).

The task made the 10 GB GH Actions cache budget observable and bounded:
a per-tick budget report, FIFO eviction of the oldest non-seed non-master
entries from 95% down to 85%, a red cron (::error + exit 1) when eviction
cannot save the budget, TTL 1d for non-master refs and a second schedule
tick. The logic lives in python embedded in the workflow YAML, so nothing
below is exercised by cargo/bun tests — CI itself is the only runtime.

These tests extract the live embedded script from cleanup-cache.yml and
execute it (minus the trailing ``main()`` call) inside a namespace, then
unit-test the extracted functions and drive ``main()`` itself with mocked
``gh`` boundaries (cache listing and delete subprocess). Every test here
was either the main scenario of the task checklist or a review finding:

1. Two schedule ticks + the dry_run dispatch input — burst protection and
   the report-only mode.
2. Non-master TTL is 1 day (was 2) — the only lever against cross-ref
   isolated duplicates; the boundary must stay ``> TTL``.
3. ``family_of`` groups the new families (qlty, playwright, apt, uv,
   Windows-gradle) used by Rule 3 and the budget report; existing
   families must not regress.
4. MASTER_SEEDS matches the un-suffixed trailbase keys — the pin drift of
   #916 made ``trailbase-v0.35.10`` evictable until this slice added
   ``(-verified)?``.
5. The budget report prints totals, the master split and family ranking.
6. Eviction: fires at 95%, stops at 85%, FIFO by created_at, protects
   seeds on any ref and master unconditionally, never double-plans an
   entry already scheduled by Rules 1-3.
7. Already-gone deletes (parallel PR-close job race) are skips, not
   failures — the red cron must stay noise-free (review round 2 High).
8. Unsalvageable budgets (::error + exit 1) and the 80% ::warning.
9. DRY_RUN deletes nothing; non-dry deletes exactly the plan.
"""

from __future__ import annotations

import re
from datetime import datetime, timedelta, timezone
from pathlib import Path

import pytest
import yaml

_REPO_ROOT = Path(__file__).resolve().parents[2]
CLEANUP_FILE = _REPO_ROOT / ".github" / "workflows" / "cleanup-cache.yml"

MASTER = "refs/heads/master"
NOW = datetime.now(timezone.utc)  # fixture ages are relative to real "now"


def _workflow() -> dict:
    return yaml.safe_load(CLEANUP_FILE.read_text(encoding="utf-8"))


def _script() -> str:
    """The embedded python of the scheduled-cleanup step, as committed."""
    run = _workflow()["jobs"]["scheduled-cleanup"]["steps"][0]["run"]
    marker = re.search(r"python3 - <<'PY'\n(.*?)\nPY\s*$", run, re.DOTALL)
    assert marker, "heredoc python block not found in cleanup-cache.yml"
    return marker.group(1)


def _namespace() -> dict:
    """Exec the embedded script without its trailing main() call."""
    script = _script()
    head, tail = script.rsplit("\nmain()", 1)
    assert tail.strip() == "", "unexpected trailing code after main() call"
    namespace: dict = {}
    exec(compile(head, str(CLEANUP_FILE), "exec"), namespace)  # noqa: S102 - test rig
    return namespace


def _cache(
    cache_id: int,
    key: str,
    ref: str,
    size: int,
    hours_old: float,
) -> dict:
    created = (NOW - timedelta(hours=hours_old)).isoformat().replace("+00:00", "Z")
    return {
        "id": cache_id,
        "key": key,
        "ref": ref,
        "size_in_bytes": size,
        "created_at": created,
    }


GB = 1_000_000_000


class _Completed:
    def __init__(self, returncode: int, stderr: str):
        self.returncode = returncode
        self.stderr = stderr


class _FakeGhDelete:
    """Stands in for the ``subprocess`` module inside main()'s delete loop."""

    def __init__(self, outcomes: dict[int, tuple[int, str]] | None = None):
        self.outcomes = outcomes or {}
        self.delete_calls: list[str] = []

    def run(self, cmd, **_kwargs):
        assert cmd[:3] == ["gh", "cache", "delete"], f"unexpected command: {cmd}"
        cache_id = cmd[3]
        self.delete_calls.append(cache_id)
        returncode, stderr = self.outcomes.get(int(cache_id), (0, ""))
        return _Completed(returncode, stderr)


# --- Contract 1: two ticks + dry_run dispatch input ---------------------------


def _triggers(workflow: dict) -> dict:
    # pyyaml parses the bare "on:" key as boolean True (YAML 1.1).
    return workflow.get(True) or workflow.get("on") or {}


def test_schedule_has_two_half_day_ticks():
    crons = _triggers(_workflow())["schedule"]
    assert sorted(c["cron"] for c in crons) == ["0 15 * * *", "0 3 * * *"], (
        f"expected the 03:00 and 15:00 UTC ticks, got: {crons}"
    )


def test_dispatch_input_dry_run_gates_the_env_flag():
    workflow = _workflow()
    triggers = _triggers(workflow)
    inputs = triggers["workflow_dispatch"]["inputs"]
    assert inputs["dry_run"]["type"] == "boolean"
    env = workflow["jobs"]["scheduled-cleanup"]["steps"][0]["env"]
    assert "inputs.dry_run" in env["DRY_RUN"], (
        "DRY_RUN env must be wired to the dispatch input"
    )


# --- Contract 2: non-master TTL ------------------------------------------------


def test_non_master_ttl_is_one_day():
    ns = _namespace()
    assert ns["NON_MASTER_TTL_DAYS"] == 1, "TTL window was narrowed to 1d by #926"


def test_ttl_reaps_only_entries_older_than_one_day():
    ns = _namespace()
    caches = [
        _cache(1, "cache-apt-pkgs_older", "refs/pull/900/merge", 1 * GB, hours_old=25),
        _cache(
            2, "cache-apt-pkgs_boundary", "refs/pull/900/merge", 1 * GB, hours_old=24
        ),
        _cache(3, "cache-apt-pkgs_fresh", "refs/pull/900/merge", 1 * GB, hours_old=2),
    ]
    deletions = ns["plan_rule_deletions"](caches, NOW)
    assert set(deletions) == {1}, f"TTL boundary broken: planned {set(deletions)}"
    assert "non-master" in deletions[1][1]


def test_ttl_never_touches_master():
    ns = _namespace()
    caches = [_cache(1, "trunk-tools-v2", MASTER, 1 * GB, hours_old=1000)]
    assert ns["plan_rule_deletions"](caches, NOW) == {}


# --- Contract 3: family_of groups new and legacy families ---------------------


@pytest.mark.parametrize(
    ("key", "family"),
    [
        # New families of this slice (report grouping + Rule 3 orphan reaping).
        ("qlty-plugins-Linux-v1-abc123", "qlty-plugins-Linux-v1"),
        ("playwright-chromium-v1-abc123", "playwright-chromium-v1"),
        ("setup-uv-2-x86_64-ubuntu-24.04-3.12.3", "setup-uv-2-x86_64-ubuntu-24.04"),
        ("Windows-gradle-abc123", "Windows-gradle"),
        ("cache-apt-pkgs_abc123", "cache-apt-pkgs"),  # underscore separator
        # Legacy families must keep working.
        ("Linux-gradle-abc123", "Linux-gradle"),
        ("cdn-data-v15-abc123", "cdn-data-v15"),
        ("v0-rust-a-b-c", "v0-rust-a"),
        # Live codeql key shape (…-<dbhash>-<lang>-<version>): rsplit("-", 3).
        (
            "codeql-overlay-base-database-1-d953d79b74456ce0-python-2.27.4",
            "codeql-overlay-base-database-1",
        ),
        # No family: unversioned one-off keys stay orphans.
        ("trunk-tools-v2", None),
    ],
)
def test_family_of(key, family):
    ns = _namespace()
    assert ns["family_of"](key) == family, f"family_of({key!r}) drifted"


# --- Contract 4: seed regex protects the un-suffixed trailbase keys -----------


@pytest.mark.parametrize(
    "key",
    [
        "trailbase-v0.35.10",  # #916 pin drift: no -verified suffix anymore
        "trailbase-v0.35.10-verified",  # legacy suffix shape
        "trailbase-v0.28.3-verified",
        "xwin-Linux-x86_64-pc-windows-msvc-v17",
        "trunk-tools-v2",
        "frontend-tools-Linux-X64-binstall-v1",
        "wasm-pack-Linux-X64-v0.15.0",
    ],
)
def test_master_seeds_match_live_key_shapes(key):
    ns = _namespace()
    assert ns["MASTER_SEEDS"].fullmatch(key), f"live seed key not protected: {key}"


@pytest.mark.parametrize(
    "key",
    [
        "trailbase-v0.35.10-x",  # suffixes must not widen the match
        "xwin-Linux-x86_64-pc-windows-msvc-v16-beta",
        "cache-apt-pkgs_abc123",
    ],
)
def test_master_seeds_do_not_overmatch(key):
    ns = _namespace()
    assert not ns["MASTER_SEEDS"].fullmatch(key), f"seed regex overmatched: {key}"


# --- Contract 5: the budget report ---------------------------------------------


def test_budget_report_prints_totals_master_split_and_families(capsys):
    ns = _namespace()
    caches = [
        _cache(1, "cdn-data-v15-abc", MASTER, 1 * GB, hours_old=2),
        _cache(
            2, "qlty-plugins-Linux-v1-a1", "refs/pull/900/merge", 2 * GB, hours_old=3
        ),
        _cache(
            3, "qlty-plugins-Linux-v1-b2", "refs/pull/901/merge", 3 * GB, hours_old=4
        ),
        _cache(4, "trunk-tools-v2", MASTER, int(0.5 * GB), hours_old=5),
    ]
    ns["print_budget_report"](caches)
    out = capsys.readouterr().out
    assert (
        "=== Cache budget: 6.50 GB / 10.00 GB (65%) — master 1.50 GB, non-master 5.00 GB, 4 entries ==="
        in out
    )
    # Families are ranked by size; the qlty duplicates collapse into one line.
    families = re.findall(r"^  (\S+) — (\d+) MB × (\d+)$", out, re.MULTILINE)
    assert families[0] == ("qlty-plugins-Linux-v1", "5000", "2"), families


# --- Contract 6: budget eviction -----------------------------------------------


def test_eviction_fires_at_95_and_stops_at_85_fifo():
    ns = _namespace()
    # 10.2 GB total; the only fresh deletable entries are four 1 GB PR caches
    # created in increasing age order (FIFO target: oldest first).
    caches = [
        _cache(1, "trunk-tools-v2", MASTER, int(0.2 * GB), hours_old=2),
        _cache(2, "cache-apt-pkgs_oldest", "refs/pull/900/merge", 1 * GB, hours_old=8),
        _cache(3, "cache-apt-pkgs_mid", "refs/pull/901/merge", 1 * GB, hours_old=6),
        _cache(4, "cache-apt-pkgs_young", "refs/pull/902/merge", 1 * GB, hours_old=4),
        _cache(5, "cache-apt-pkgs_newest", "refs/pull/903/merge", 1 * GB, hours_old=2),
        _cache(
            6, "qlty-plugins-Linux-v1-x1", "refs/pull/904/merge", 6 * GB, hours_old=2
        ),
    ]
    projected, evicted = ns["plan_budget_eviction"](caches, {})
    assert [c["id"] for c in evicted] == [2, 3], (
        "eviction must take the oldest entries first and stop at the 85% target"
    )
    assert projected == pytest.approx(8.2 * GB)
    assert projected <= ns["EVICT_TO_BYTES"]


def test_eviction_protects_seeds_on_any_ref_and_master():
    ns = _namespace()
    caches = [
        _cache(1, "trunk-tools-v2", MASTER, 5 * GB, hours_old=2),
        # Seed key on a PR merge ref: protected even though it is huge.
        _cache(2, "trailbase-v0.35.10", "refs/pull/920/merge", 4 * GB, hours_old=3),
        # The only evictable entry: a fresh non-seed PR duplicate.
        _cache(3, "cache-apt-pkgs_x", "refs/pull/900/merge", 1 * GB, hours_old=2),
        _cache(4, "cache-apt-pkgs_y", "refs/pull/901/merge", 1 * GB, hours_old=1),
    ]
    deletions: dict = {}
    projected, evicted = ns["plan_budget_eviction"](caches, deletions)
    # 11 GB total, only the two 1 GB plain duplicates are evictable.
    assert projected == pytest.approx(9 * GB), "evictable entry survived"
    assert [c["id"] for c in evicted] == [3, 4], "FIFO order violated"
    assert 1 not in deletions and 2 not in deletions, "seed/master protection broken"


def test_eviction_is_noop_below_the_threshold():
    ns = _namespace()
    caches = [
        _cache(1, "trunk-tools-v2", MASTER, 2 * GB, hours_old=2),
        _cache(2, "cache-apt-pkgs_x", "refs/pull/900/merge", 6 * GB, hours_old=2),
    ]
    projected, evicted = ns["plan_budget_eviction"](caches, {})
    assert projected == pytest.approx(8 * GB)
    assert evicted == []


def test_eviction_never_double_plans_rule_deletions():
    ns = _namespace()
    caches = [
        _cache(1, "cache-apt-pkgs_ttl", "refs/pull/900/merge", 1 * GB, hours_old=30),
        _cache(2, "cache-apt-pkgs_x", "refs/pull/901/merge", 9 * GB, hours_old=2),
        _cache(3, "trunk-tools-v2", MASTER, 1 * GB, hours_old=2),
    ]
    deletions = ns["plan_rule_deletions"](caches, NOW)
    assert set(deletions) == {1}, "fixture error: TTL entry not planned"
    _, evicted = ns["plan_budget_eviction"](caches, deletions)
    assert [c["id"] for c in evicted] == [2], "a Rule 1 entry got evicted twice"


# --- Contract 7: already-gone deletes are skips, not failures ------------------


_ALREADY_GONE = "X Could not find a cache matching {id} in yurvon-screamo/origa"


def _two_stale_caches() -> list[dict]:
    return [
        _cache(101, "cache-apt-pkgs_a", "refs/pull/900/merge", 1 * GB, hours_old=30),
        _cache(202, "cache-apt-pkgs_b", "refs/pull/901/merge", 1 * GB, hours_old=30),
    ]


def _two_stale_plus_seed() -> list[dict]:
    """2 GB scheduled for deletion, 1 GB survives: projected 1.00 GB."""
    return [
        _cache(101, "cache-apt-pkgs_a", "refs/pull/900/merge", 1 * GB, hours_old=30),
        _cache(202, "cache-apt-pkgs_b", "refs/pull/901/merge", 1 * GB, hours_old=30),
        _cache(303, "trunk-tools-v2", MASTER, 1 * GB, hours_old=2),
    ]


def test_already_gone_race_is_a_skip_and_exits_zero(capsys):
    ns = _namespace()
    ns["DRY_RUN"] = False
    ns["fetch_all_caches"] = _two_stale_caches
    fake = _FakeGhDelete(
        {101: (1, _ALREADY_GONE.format(id=101)), 202: (1, _ALREADY_GONE.format(id=202))}
    )
    ns["subprocess"] = fake
    ns["main"]()  # must not raise SystemExit
    out = capsys.readouterr().out
    assert out.count("SKIP already-gone") == 2, out
    assert "=== Cleanup complete: 0 deleted, 2 already-gone, 0 failed ===" in out
    assert "::error" not in out


@pytest.mark.parametrize(
    "stderr",
    [
        "DELETE https://api.github.com/repos/x/y/actions/caches/1: 404 Not Found",
        "cache delete failed: cache not found",
    ],
)
def test_compatible_already_gone_wordings_are_skips(stderr, capsys):
    ns = _namespace()
    ns["DRY_RUN"] = False
    ns["fetch_all_caches"] = _two_stale_caches
    ns["subprocess"] = _FakeGhDelete({101: (1, stderr)})
    ns["main"]()
    out = capsys.readouterr().out
    assert "SKIP already-gone" in out, out
    assert "1 failed" not in out


def test_real_api_failure_counts_as_failed_and_exits_one(capsys):
    ns = _namespace()
    ns["DRY_RUN"] = False
    ns["fetch_all_caches"] = _two_stale_caches
    ns["subprocess"] = _FakeGhDelete(
        {
            101: (1, "HTTP 403: Resource not accessible by integration"),
            202: (0, ""),
        }
    )
    with pytest.raises(SystemExit) as excinfo:
        ns["main"]()
    assert excinfo.value.code == 1
    out = capsys.readouterr().out
    assert "WARNING: failed to delete cache 101" in out, out
    assert "=== Cleanup complete: 1 deleted, 0 already-gone, 1 failed ===" in out


# --- Contract 8: unsalvageable budget fails the cron; 80% warns ----------------


def test_unsalvageable_budget_prints_error_and_exits_one(capsys):
    ns = _namespace()
    # Only protected entries: master + seed keys on merge refs, all fresh —
    # nothing is deletable, projected stays above the eviction threshold.
    ns["DRY_RUN"] = False
    ns["fetch_all_caches"] = lambda: [
        _cache(1, "trunk-tools-v2", MASTER, 6 * GB, hours_old=2),
        _cache(2, "trailbase-v0.35.10", "refs/pull/920/merge", 5 * GB, hours_old=2),
    ]
    ns["subprocess"] = _FakeGhDelete()
    with pytest.raises(SystemExit) as excinfo:
        ns["main"]()
    assert excinfo.value.code == 1
    out = capsys.readouterr().out
    assert "::error::cache budget unsalvageable" in out, out
    assert "EVICT [budget]" not in out


def test_projected_above_80_percent_warns_but_exits_zero(capsys):
    ns = _namespace()
    ns["DRY_RUN"] = False
    ns["fetch_all_caches"] = lambda: [
        _cache(1, "trunk-tools-v2", MASTER, 2 * GB, hours_old=2),
        _cache(2, "cache-apt-pkgs_x", "refs/pull/900/merge", 7 * GB, hours_old=2),
    ]
    ns["subprocess"] = _FakeGhDelete()
    ns["main"]()
    out = capsys.readouterr().out
    assert "::warning::cache budget projected at 9.00 GB" in out, out
    assert "::error" not in out
    assert "0 deleted, 0 already-gone, 0 failed" in out


# --- Contract 9: dry run deletes nothing ---------------------------------------


def test_dry_run_plans_but_never_deletes(capsys):
    ns = _namespace()
    ns["DRY_RUN"] = True
    ns["fetch_all_caches"] = _two_stale_plus_seed
    fake = _FakeGhDelete()
    ns["subprocess"] = fake
    ns["main"]()
    out = capsys.readouterr().out
    assert "=== DRY RUN: nothing deleted ===" in out
    assert "=== 3 caches, deleting 2 (2.00 GB), projected after: 1.00 GB ===" in out
    assert fake.delete_calls == [], "dry run must not invoke gh cache delete"


def test_non_dry_deletes_exactly_the_planned_entries(capsys):
    ns = _namespace()
    ns["DRY_RUN"] = False
    ns["fetch_all_caches"] = _two_stale_plus_seed
    fake = _FakeGhDelete()
    ns["subprocess"] = fake
    ns["main"]()
    assert sorted(fake.delete_calls) == ["101", "202"], fake.delete_calls
    out = capsys.readouterr().out
    assert "=== Cleanup complete: 2 deleted, 0 already-gone, 0 failed ===" in out


# --- Regressions from review round 2 -------------------------------------------


def test_evicted_lines_are_marked_separately_from_rule_deletions(capsys):
    """EVICT [budget] vs DELETE [reason]: the evicted list must drive the
    label, not a reason-string match (round 2 Low, already-gone fix family)."""
    ns = _namespace()
    ns["DRY_RUN"] = True
    ns["fetch_all_caches"] = lambda: [
        _cache(1, "cache-apt-pkgs_ttl", "refs/pull/900/merge", 1 * GB, hours_old=30),
        _cache(2, "cache-apt-pkgs_x", "refs/pull/901/merge", 9 * GB, hours_old=2),
        _cache(3, "trunk-tools-v2", MASTER, 1 * GB, hours_old=2),
    ]
    ns["main"]()
    out = capsys.readouterr().out
    assert "EVICT [budget]: cache-apt-pkgs_x" in out, out
    assert "DELETE [non-master" in out and "cache-apt-pkgs_ttl" in out, out


def test_annotation_percentages_follow_the_byte_constants(capsys):
    """80%/95% in annotation texts must follow the byte thresholds (round 2
    Low fix): the printed percentage is derived, not a hardcoded literal."""
    ns = _namespace()
    ns["DRY_RUN"] = False
    ns["subprocess"] = _FakeGhDelete()
    # 9 GB projected lands between the warn (8 GB) and evict (9.5 GB) marks.
    ns["fetch_all_caches"] = lambda: [
        _cache(1, "trunk-tools-v2", MASTER, 2 * GB, hours_old=2),
        _cache(2, "cache-apt-pkgs_x", "refs/pull/900/merge", 7 * GB, hours_old=2),
    ]
    ns["main"]()
    out = capsys.readouterr().out
    assert "(>= 80% of 10.00 GB)" in out, out
    # The unsalvageable branch prints the eviction threshold percentage.
    ns["fetch_all_caches"] = lambda: [
        _cache(1, "trunk-tools-v2", MASTER, 6 * GB, hours_old=2),
        _cache(2, "trailbase-v0.35.10", "refs/pull/920/merge", 5 * GB, hours_old=2),
    ]
    with pytest.raises(SystemExit):
        ns["main"]()
    out = capsys.readouterr().out
    assert "stays >= 95% after eviction" in out, out


def test_projected_total_is_reported_in_the_summary(capsys):
    """The cleanup summary must carry the projected total (budget observability)."""
    ns = _namespace()
    ns["DRY_RUN"] = True
    ns["fetch_all_caches"] = _two_stale_plus_seed
    ns["main"]()
    out = capsys.readouterr().out
    assert "projected after: 1.00 GB" in out, out
