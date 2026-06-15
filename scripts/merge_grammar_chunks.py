"""Merge grammar rule/chunk files into the final ``cdn/grammar/grammar.json``.

Two input formats are supported (the per-rule format is preferred; the legacy
chunk format remains for backward compatibility and emits a deprecation warning):

  - **Per-rule (preferred):** ``cdn/grammar/rules/<chunk_id>/rule_*.json`` — one
    rule object per file. No ``metadata``/``rules`` wrapper.
  - **Legacy chunk (deprecated):** ``cdn/grammar/chunks/<chunk_id>.json`` — a
    single file with ``{"metadata": {...}, "rules": [...]}``.

The merge pipeline:
  1. Load the existing ``grammar.json`` (N5/N4 base, order preserved).
  2. Collect new rules from per-rule files and (optionally) legacy chunks.
  3. Validate every new rule (shared checks from ``validate_grammar``).
  4. Patch-or-append: a new rule whose ``rule_id`` already exists in the base
     REPLACES that base rule in-place (patch mode — lets ``rules/`` act as
     source-of-truth for legacy N5/N4 rules). New rule_ids are appended.
  5. WARN on duplicate ``(level, title)`` among the new rules.
  6. Sort new (appended) rules by ``level`` (N3 → N2 → N1) → ``lesson`` → ``title``.
  7. Write ``grammar.json`` atomically (temp-file + rename).

The ``_example`` subdirectory of ``rules/`` is always excluded from the merge.

Usage:
    python scripts/merge_grammar_chunks.py
    python scripts/merge_grammar_chunks.py --dry-run
    python scripts/merge_grammar_chunks.py --rules-dir cdn/grammar/rules \\
        --chunks-dir cdn/grammar/chunks --output cdn/grammar/grammar.json

Requirements: Python 3.10+ (stdlib only).
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from validate_grammar import (
    DEFAULT_WHITELIST_SOURCE,
    ValidationReport,
    load_rules,
    parse_format_action_whitelist,
    validate_rules,
)

DEFAULT_RULES_DIR = Path("cdn/grammar/rules")
DEFAULT_CHUNKS_DIR = Path("cdn/grammar/chunks")
DEFAULT_OUTPUT = Path("cdn/grammar/grammar.json")
LEVEL_ORDER = {"N5": 0, "N4": 1, "N3": 2, "N2": 3, "N1": 4}

# Tooling-only metadata that lives in per-rule files for the merge report and
# sort key, but is NOT part of the Rust ``GrammarRule`` struct. Stripped before
# writing the canonical store so the schema stays homogeneous with the existing
# N5/N4 rules (which never carried these fields).
TOOLING_ONLY_FIELDS = ("chunk_id", "lesson")


@dataclass
class MergeReportContext:
    """Aggregated inputs to ``build_report`` — keeps the helper's arity small."""

    existing_count: int
    total_count: int
    rules_per_chunk: dict[str, list[Path]] = field(default_factory=dict)
    chunk_id_per_subdir: dict[str, str | None] = field(default_factory=dict)
    legacy_chunks: list[Path] = field(default_factory=list)
    new_count: int = 0
    patched_count: int = 0


def collect_rule_files(rules_dir: Path) -> dict[str, list[Path]]:
    """Return ``{chunk_id: [rule_paths]}`` for non-excluded subdirs of ``rules_dir``.

    Subdirectories whose name starts with ``_`` (samples, scratch) are skipped.
    Only files matching ``rule_*.json`` inside each subdirectory are considered,
    so non-rule JSON (e.g. ``manifest.json``) is ignored.
    """
    if not rules_dir.is_dir():
        return {}

    result: dict[str, list[Path]] = {}
    for subdir in sorted(rules_dir.iterdir()):
        if not subdir.is_dir() or subdir.name.startswith("_"):
            continue
        rule_paths = sorted(subdir.glob("rule_*.json"))
        if rule_paths:
            result[subdir.name] = rule_paths
    return result


def collect_legacy_chunks(chunks_dir: Path) -> list[Path]:
    """Return sorted legacy chunk files (deprecated format).

    All ``*.json`` files are returned except those whose name starts with ``_``
    (samples). Callers should treat this format as deprecated and migrate to
    per-rule files under ``--rules-dir``.
    """
    if not chunks_dir.is_dir():
        return []

    return sorted(p for p in chunks_dir.glob("*.json") if not p.name.startswith("_"))


def load_existing_grammar(
    output_path: Path,
    report: ValidationReport,
) -> list[dict[str, Any]]:
    """Load the existing grammar.json store. Returns empty list if absent.

    The canonical store lives in gitignored ``cdn/`` and cannot be restored
    from version control, so a corrupt base is reported as ERROR (aborting the
    merge) rather than left to surface as a raw traceback mid-write.
    """
    if not output_path.exists():
        return []
    try:
        return load_rules(output_path)
    except (json.JSONDecodeError, ValueError, OSError) as exc:
        report.error(f"Cannot load existing grammar store {output_path}: {exc}")
        return []


def load_files(
    file_paths: list[Path],
    whitelist: dict[str, list[str]],
    report: ValidationReport,
    label_prefix: str,
    load_failures_as_warnings: bool = False,
) -> list[dict[str, Any]]:
    """Load and validate a list of files, returning a flat list of rules.

    Each file is validated independently; per-file errors/warnings are forwarded
    to ``report`` with the source filename as a prefix. Files that fail to load
    or contain validation errors are skipped (their rules are not merged).

    When ``load_failures_as_warnings`` is True (used for the deprecated legacy
    chunk format), I/O / JSON / structural load failures become WARN instead of
    ERROR, so a half-written legacy chunk left by a crashed engineer does not
    block the merge. Validation errors (rule schema) remain ERROR in both modes.
    """
    all_new: list[dict[str, Any]] = []
    for path in file_paths:
        file_report = ValidationReport()
        try:
            rules = load_rules(path)
        except (json.JSONDecodeError, ValueError, OSError) as exc:
            channel = report.warn if load_failures_as_warnings else report.error
            channel(f"{label_prefix}{path.name}: cannot load — {exc}")
            continue

        validate_rules(rules, whitelist, file_report)
        for warning in file_report.warnings:
            report.warn(f"{label_prefix}{path.name}: {warning}")
        for error in file_report.errors:
            report.error(f"{label_prefix}{path.name}: {error}")

        if file_report.ok:
            all_new.extend(rules)
    return all_new


def _verify_chunk_id_consistency(
    subdir_name: str,
    rules: list[dict[str, Any]],
    report: ValidationReport,
) -> str | None:
    """Return the ``chunk_id`` shared by all rules in ``subdir_name``.

    Tooling-only metadata (see ``TOOLING_ONLY_FIELDS``) is used by the merge
    report so authors can verify grouping. If rules within the same subdirectory
    declare different ``chunk_id`` values, the inconsistency is surfaced as WARN
    (rather than silently picking the first one) so authors can fix the grouping.
    """
    observed: list[str] = []
    for rule in rules:
        if not isinstance(rule, dict):
            continue
        cid = rule.get("chunk_id")
        if isinstance(cid, str) and cid and cid not in observed:
            observed.append(cid)
    if not observed:
        return None
    if len(observed) > 1:
        report.warn(
            f"subdir {subdir_name!r} contains rules with different chunk_id values: {observed}"
        )
    return observed[0]


def _strip_tooling_fields(rule: dict[str, Any]) -> dict[str, Any]:
    """Return a shallow copy of ``rule`` without tooling-only metadata.

    ``chunk_id`` and ``lesson`` are merge-time helpers (grouping + sort key)
    and are not part of the Rust ``GrammarRule`` struct. Keeping them out of
    the canonical store preserves schema homogeneity with the existing N5/N4
    rules, which never carried these fields.
    """
    return {k: v for k, v in rule.items() if k not in TOOLING_ONLY_FIELDS}


def patch_or_append_rules(
    existing: list[dict[str, Any]],
    new_rules: list[dict[str, Any]],
    report: ValidationReport,
) -> tuple[list[dict[str, Any]], dict[int, dict[str, Any]]]:
    """Patch existing rules by ``rule_id`` or append new ones; hard error on dup.

    Returns ``(appended_rules, patch_index_map)`` where ``patch_index_map``
    maps a base-list index to the rule replacing it (used by
    :func:`apply_patches_in_place` to mutate the base in place). The patched
    count is ``len(patch_index_map)`` at the call site.

    - A new rule whose ``rule_id`` matches an existing base rule PATCHES that
      base rule (the ``rules/`` version wins). This lets the per-rule files act
      as source-of-truth for legacy N5/N4 rules that live in ``grammar.json``.
    - A new rule whose ``rule_id`` is not in the base is appended.
    - A ``rule_id`` repeated across two new rule files is a hard error (the
      author must disambiguate) — the second occurrence is dropped.
    """
    existing_ids: dict[str, int] = {
        rule["rule_id"]: idx
        for idx, rule in enumerate(existing)
        if isinstance(rule, dict) and isinstance(rule.get("rule_id"), str)
    }
    seen_new: set[str] = set()
    appended: list[dict[str, Any]] = []
    patch_index_map: dict[int, dict[str, Any]] = {}
    for rule in new_rules:
        rule_id = rule.get("rule_id") if isinstance(rule, dict) else None
        if not isinstance(rule_id, str):
            appended.append(rule)
            continue
        if rule_id in seen_new:
            report.error(
                f"Duplicate rule_id {rule_id!r} in two new rule files — disambiguate before merge"
            )
            continue
        seen_new.add(rule_id)
        if rule_id in existing_ids:
            patch_index_map[existing_ids[rule_id]] = rule
        else:
            appended.append(rule)
    return appended, patch_index_map


def apply_patches_in_place(
    existing: list[dict[str, Any]],
    patch_index_map: dict[int, dict[str, Any]],
) -> None:
    """Replace base rules in-place according to ``patch_index_map``.

    The ``_in_place`` suffix surfaces the side effect at the call site so
    readers do not assume ``existing`` is unchanged. Mutating in place preserves
    the original N5/N4 ordering — the patched rule lands exactly where the
    legacy rule used to sit, rather than being moved to the appended tail.
    """
    for idx, replacement in patch_index_map.items():
        existing[idx] = replacement


def warn_on_duplicate_titles(
    new_rules: list[dict[str, Any]],
    report: ValidationReport,
) -> None:
    """WARN on duplicate ``(level, title)`` pairs within new rules.

    Cross-file duplicates between two per-rule files would otherwise be missed
    because ``validate_rules`` only sees one file at a time.
    """
    seen: dict[tuple[str, str], str] = {}
    for rule in new_rules:
        if not isinstance(rule, dict):
            continue
        level = rule.get("level")
        content = rule.get("content")
        title = content.get("English", {}).get("title") if isinstance(content, dict) else None
        if not isinstance(level, str) or not isinstance(title, str) or not title:
            continue
        key = (level, title)
        if key in seen:
            report.warn(
                f"Duplicate (level={level}, title={title!r}) — first seen rule_id={seen[key]!r}, "
                f"now rule_id={rule.get('rule_id')!r}; mutate title to disambiguate"
            )
        else:
            seen[key] = str(rule.get("rule_id", "?"))


def _sort_key(rule: dict[str, Any]) -> tuple[int, int, str]:
    """Sort by level rank → optional ``lesson`` index → EN title.

    The ``lesson`` field is chunk-local metadata (not part of the Rust
    ``GrammarRule`` struct); absent or non-int values sort last within a level.
    """
    level = rule.get("level", "")
    level_rank = LEVEL_ORDER.get(level, 99)
    lesson = rule.get("lesson")
    lesson_rank = lesson if isinstance(lesson, int) else 999
    title = ""
    content = rule.get("content")
    if isinstance(content, dict):
        english = content.get("English")
        if isinstance(english, dict):
            title = english.get("title", "")
    return (level_rank, lesson_rank, title)


def sort_merged(
    existing: list[dict[str, Any]],
    appended_rules: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    """Existing rules keep their order; appended rules sorted by level → lesson → title."""
    return existing + sorted(appended_rules, key=_sort_key)


def write_grammar(path: Path, rules: list[dict[str, Any]]) -> None:
    """Atomically write the merged store as pretty JSON (UTF-8, no BOM).

    Tooling-only fields (``chunk_id``, ``lesson``) are stripped before writing
    so the canonical store keeps the same schema as the existing N5/N4 rules.
    Uses temp-file + rename so a crash mid-write cannot corrupt the canonical
    grammar store (which lives in gitignored ``cdn/`` and cannot be restored
    from version control).
    """
    path.parent.mkdir(parents=True, exist_ok=True)
    cleaned = [_strip_tooling_fields(rule) for rule in rules]
    payload = {"grammar": cleaned}
    tmp = path.with_suffix(".json.tmp")
    with tmp.open("w", encoding="utf-8", newline="\n") as handle:
        json.dump(payload, handle, ensure_ascii=False, indent=2)
        handle.write("\n")
    tmp.replace(path)


def build_report(ctx: MergeReportContext, report: ValidationReport) -> str:
    lines: list[str] = [
        "Grammar merge report",
        f"  Base rules (existing):       {ctx.existing_count}",
    ]
    total_rule_files = sum(len(paths) for paths in ctx.rules_per_chunk.values())
    lines.append(f"  Rule files (new format):     {total_rule_files}")
    for subdir_name, paths in sorted(ctx.rules_per_chunk.items()):
        cid = ctx.chunk_id_per_subdir.get(subdir_name)
        suffix = f" (chunk_id={cid})" if cid and cid != subdir_name else ""
        lines.append(f"    {subdir_name}{suffix}: {len(paths)} rule file(s)")
    if ctx.legacy_chunks:
        lines.append(f"  Legacy chunk files (deprecated): {len(ctx.legacy_chunks)}")
        for path in ctx.legacy_chunks:
            lines.append(f"    {path.name}")
    lines.extend(
        [
            f"  Patched existing rules:      {ctx.patched_count}",
            f"  Appended new rules:          {ctx.new_count}",
            f"  Total after merge:           {ctx.total_count}",
            f"  Errors:                      {len(report.errors)}",
            f"  Warnings:                    {len(report.warnings)}",
        ]
    )
    return "\n".join(lines)


def _describe_action(dry_run: bool, ok: bool) -> str:
    """Pick a verb that reflects what (if anything) will be written.

    Avoids the misleading "writing to" message when validation errors block
    the write — operators should not be told a file was written when it wasn't.
    """
    if dry_run:
        return "[dry-run] would write"
    return "writing to" if ok else "[skipped write due to errors] would have written"


def merge(
    rules_dir: Path,
    chunks_dir: Path,
    output: Path,
    whitelist_source: Path,
    dry_run: bool,
    report: ValidationReport,
) -> list[dict[str, Any]]:
    """Execute the merge pipeline. Returns the merged rule list."""
    try:
        whitelist = parse_format_action_whitelist(whitelist_source)
    except (OSError, ValueError) as exc:
        report.error(f"Cannot parse whitelist from {whitelist_source}: {exc}")
        return []

    rules_per_chunk = collect_rule_files(rules_dir)
    legacy_chunks = collect_legacy_chunks(chunks_dir)
    if legacy_chunks:
        report.warn(
            f"Legacy chunk format detected in {chunks_dir} ({len(legacy_chunks)} file(s)); "
            f"this format is deprecated — migrate to per-rule files under {rules_dir}/<chunk_id>/"
        )

    existing = load_existing_grammar(output, report)

    new_rules: list[dict[str, Any]] = []
    chunk_id_per_subdir: dict[str, str | None] = {}
    for subdir_name, rule_paths in rules_per_chunk.items():
        subdir_rules = load_files(
            rule_paths, whitelist, report, label_prefix=f"rule:{subdir_name}/"
        )
        chunk_id_per_subdir[subdir_name] = _verify_chunk_id_consistency(
            subdir_name, subdir_rules, report
        )
        new_rules.extend(subdir_rules)

    new_rules.extend(
        load_files(
            legacy_chunks,
            whitelist,
            report,
            label_prefix="chunk:",
            load_failures_as_warnings=True,
        )
    )

    warn_on_duplicate_titles(new_rules, report)
    appended, patch_index_map = patch_or_append_rules(existing, new_rules, report)
    apply_patches_in_place(existing, patch_index_map)
    merged = sort_merged(existing, appended)

    ctx = MergeReportContext(
        existing_count=len(existing),
        total_count=len(merged),
        rules_per_chunk=rules_per_chunk,
        chunk_id_per_subdir=chunk_id_per_subdir,
        legacy_chunks=legacy_chunks,
        new_count=len(appended),
        patched_count=len(patch_index_map),
    )
    summary = build_report(ctx, report)
    action = _describe_action(dry_run, report.ok)
    print(f"{summary}\n  {action}: {output}", file=sys.stderr)

    if not dry_run and report.ok:
        write_grammar(output, merged)

    return merged


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Merge grammar per-rule files (and legacy chunks) into grammar.json"
    )
    parser.add_argument("--dry-run", action="store_true", help="Show report without writing")
    parser.add_argument(
        "--rules-dir",
        type=Path,
        default=DEFAULT_RULES_DIR,
        help="Directory of per-rule files (preferred format)",
    )
    parser.add_argument(
        "--chunks-dir",
        type=Path,
        default=DEFAULT_CHUNKS_DIR,
        help="Directory of legacy chunk files (deprecated)",
    )
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument(
        "--whitelist-source",
        type=Path,
        default=DEFAULT_WHITELIST_SOURCE,
        help="Path to grammar.rs for FormatAction whitelist parsing",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = ValidationReport()
    merge(
        args.rules_dir,
        args.chunks_dir,
        args.output,
        args.whitelist_source,
        args.dry_run,
        report,
    )
    report.emit()
    return 0 if report.ok else 1


if __name__ == "__main__":
    sys.exit(main())
