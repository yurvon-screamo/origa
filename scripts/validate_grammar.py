"""Validate a grammar JSON file (rule, chunk, or final grammar.json) against schema rules.

Reads the FormatAction whitelist by parsing ``origa/src/dictionary/grammar.rs``
directly (no build.rs dependency), then validates every rule's structure,
content parity (RU/EN), format_map actions, and keywords.

Three input formats are auto-detected from the top-level keys:
  - Final store:    ``{"grammar": [...]}``
  - Legacy chunk:   ``{"metadata": {...}, "rules": [...]}``
  - Single rule:    ``{"rule_id": "...", "content": {...}, ...}``

The ``--file`` argument also accepts a directory: every ``*.json`` (recursively)
under it is validated as one of the formats above.

Exit code: 0 if no errors (warnings do not affect exit code), 1 otherwise.

Usage:
    python scripts/validate_grammar.py --file cdn/grammar/chunks/n3_01.json
    python scripts/validate_grammar.py --file cdn/grammar/grammar.json \\
        --whitelist-source origa/src/dictionary/grammar.rs
    python scripts/validate_grammar.py --file cdn/grammar/rules/_example/rule_01.json
    python scripts/validate_grammar.py --file cdn/grammar/rules/n3_lessons_01-04/

Requirements: Python 3.10+ (stdlib only).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, TextIO

ULID_PATTERN = re.compile(r"^[0-9A-HJKMNP-TV-Z]{26}$")
VALID_LEVELS = {"N5", "N4", "N3", "N2", "N1"}
STRICT_LEVELS = {"N3", "N2", "N1"}
MIN_EXPECTED_VARIANTS = 30
REQUIRED_CONTENT_FIELDS = (
    "title",
    "short_description",
    "explanation",
    "how_to_form",
    "examples",
    "nuances",
    "pro_tip",
)
OPTIONAL_CONTENT_FIELDS = ("related_patterns",)
VALID_POS_KEYS = {"Verb", "IAdjective", "NaAdjective"}
SUPPORTED_LANGS = ("Russian", "English")

DEFAULT_WHITELIST_SOURCE = Path("origa/src/dictionary/grammar.rs")


@dataclass
class ValidationReport:
    """Collects errors and warnings produced during validation."""

    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)

    def error(self, message: str) -> None:
        self.errors.append(message)

    def warn(self, message: str) -> None:
        self.warnings.append(message)

    @property
    def ok(self) -> bool:
        return not self.errors

    def emit(self, stream_err: TextIO = sys.stderr) -> None:
        for warning in self.warnings:
            print(f"WARN: {warning}", file=stream_err)
        for error in self.errors:
            print(f"ERROR: {error}", file=stream_err)


def parse_format_action_whitelist(grammar_rs_path: Path) -> dict[str, list[str]]:
    """Parse the ``FormatAction`` enum and return ``{variant: [field_names]}``.

    Raises ``FileNotFoundError`` / ``ValueError`` when the enum cannot be located
    or yields fewer variants than expected (guards against silent partial parses
    caused by formatting changes in the Rust source).
    """
    content = grammar_rs_path.read_text(encoding="utf-8")
    enum_match = re.search(r"enum\s+FormatAction\s*\{(.+?)\n\}", content, re.DOTALL)
    if not enum_match:
        raise ValueError(f"FormatAction enum not found in {grammar_rs_path}")

    whitelist: dict[str, list[str]] = {}
    for match in re.finditer(r"(\w+)\s*\{([^}]*)\}", enum_match.group(1)):
        variant_name = match.group(1)
        fields_block = match.group(2).strip()
        field_names: list[str] = []
        if fields_block:
            for raw_field in fields_block.split(","):
                raw_field = raw_field.strip()
                if ":" in raw_field:
                    field_names.append(raw_field.split(":")[0].strip())
        whitelist[variant_name] = field_names

    _assert_whitelist_sane(whitelist, grammar_rs_path)
    return whitelist


def _assert_whitelist_sane(
    whitelist: dict[str, list[str]],
    source: Path,
) -> None:
    """Reject obviously incomplete parses (empty or suspiciously small)."""
    if not whitelist:
        raise ValueError(f"No FormatAction variants parsed from {source}")
    if len(whitelist) < MIN_EXPECTED_VARIANTS:
        raise ValueError(
            f"Only {len(whitelist)} FormatAction variants parsed from {source} "
            f"(expected at least {MIN_EXPECTED_VARIANTS}) — regex may need updating"
        )


def load_rules(path: Path) -> list[dict]:
    """Load grammar rules from a file or directory.

    Accepts:
      - A directory: every ``*.json`` (recursively) is loaded and concatenated.
      - Final store:  ``{"grammar": [...]}``.
      - Legacy chunk: ``{"metadata": {...}, "rules": [...]}``.
      - Single rule:  ``{"rule_id": "...", "content": {...}, ...}`` — wrapped as
        ``[rule]`` so callers always receive a list.

    Raises ``ValueError`` if the structure is unrecognized.
    """
    if path.is_dir():
        rules: list[dict] = []
        for json_file in sorted(path.rglob("*.json")):
            rules.extend(load_rules(json_file))
        return rules

    with path.open(encoding="utf-8") as handle:
        data = json.load(handle)

    if isinstance(data, dict) and "grammar" in data:
        rules = data["grammar"]
        if not isinstance(rules, list):
            raise ValueError("'grammar' must be a list")
        return rules

    if isinstance(data, dict) and "rules" in data:
        rules = data["rules"]
        if not isinstance(rules, list):
            raise ValueError("'rules' must be a list")
        return rules

    if isinstance(data, dict) and "rule_id" in data:
        return [data]

    keys = list(data.keys()) if isinstance(data, dict) else type(data).__name__
    raise ValueError(
        f"Unrecognized structure: expected 'grammar', 'rules', or 'rule_id' key, got {keys}"
    )


def _check_ulid(rule_id: Any, report: ValidationReport, index: int) -> None:
    if not isinstance(rule_id, str) or not ULID_PATTERN.match(rule_id):
        report.error(f"rule[{index}]: rule_id is not a valid ULID (26-char Crockford base32): {rule_id!r}")


def _check_level(level: Any, report: ValidationReport, index: int) -> None:
    if level not in VALID_LEVELS:
        report.error(f"rule[{index}]: level {level!r} not in {sorted(VALID_LEVELS)}")


def _check_content_fields(
    content: dict,
    lang: str,
    index: int,
    report: ValidationReport,
) -> None:
    for field_name in REQUIRED_CONTENT_FIELDS:
        value = content.get(field_name)
        if not isinstance(value, str) or not value.strip():
            report.error(f"rule[{index}]: content.{lang}.{field_name} missing or empty")


def _check_related_patterns_parity(
    ru: dict,
    en: dict,
    index: int,
    report: ValidationReport,
) -> None:
    ru_has = "related_patterns" in ru
    en_has = "related_patterns" in en
    if ru_has != en_has:
        report.error(
            f"rule[{index}]: 'related_patterns' parity broken — present in one language but not the other"
        )
    for lang, content in (("Russian", ru), ("English", en)):
        value = content.get("related_patterns")
        if value is not None and (not isinstance(value, str) or not value.strip()):
            report.error(f"rule[{index}]: content.{lang}.related_patterns must be a non-empty string")


def _check_title_parity(
    ru: dict,
    en: dict,
    level: str | None,
    index: int,
    report: ValidationReport,
) -> None:
    ru_title = ru.get("title")
    en_title = en.get("title")
    if not (isinstance(ru_title, str) and isinstance(en_title, str)) or ru_title == en_title:
        return

    message = (
        f"rule[{index}]: title differs between languages "
        f"(RU={ru_title!r}, EN={en_title!r})"
    )
    if level in STRICT_LEVELS:
        report.error(message)
    else:
        report.warn(message + " — accepted for legacy N5/N4, fix when revising")


def _check_content(
    rule: dict,
    level: str | None,
    index: int,
    report: ValidationReport,
) -> tuple[str, str]:
    """Validate bilingual content block. Returns ``(en_title, ru_title)`` for cross-rule checks."""
    content = rule.get("content")
    if not isinstance(content, dict):
        report.error(f"rule[{index}]: 'content' missing or not an object")
        return ("", "")

    missing = [lang for lang in SUPPORTED_LANGS if lang not in content]
    if missing:
        report.error(f"rule[{index}]: content missing languages: {missing}")
        return ("", "")

    ru = content["Russian"]
    en = content["English"]
    if not isinstance(ru, dict) or not isinstance(en, dict):
        report.error(f"rule[{index}]: content.Russian / content.English must be objects")
        return ("", "")

    _check_content_fields(ru, "Russian", index, report)
    _check_content_fields(en, "English", index, report)
    _check_related_patterns_parity(ru, en, index, report)
    _check_title_parity(ru, en, level, index, report)

    return (en.get("title", ""), ru.get("title", ""))


def _check_format_map_entry(
    action: Any,
    whitelist: dict[str, list[str]],
    index: int,
    pos_key: str,
    action_no: int,
    report: ValidationReport,
) -> None:
    if not isinstance(action, dict) or len(action) != 1:
        report.error(
            f"rule[{index}]: format_map[{pos_key}][{action_no}] must be an object with exactly one key"
        )
        return

    action_name = next(iter(action))
    fields = action.get(action_name)
    if action_name not in whitelist:
        report.error(
            f"rule[{index}]: format_map[{pos_key}][{action_no}] uses unknown action '{action_name}'"
        )
        return

    if not isinstance(fields, dict):
        report.error(
            f"rule[{index}]: format_map[{pos_key}][{action_no}].{action_name} must be an object"
        )
        return

    required_fields = whitelist[action_name]
    for req in required_fields:
        value = fields.get(req)
        if not isinstance(value, str) or not value:
            report.error(
                f"rule[{index}]: format_map[{pos_key}][{action_no}].{action_name} missing required string field '{req}'"
            )


def _check_format_map(
    rule: dict,
    whitelist: dict[str, list[str]],
    index: int,
    report: ValidationReport,
) -> bool:
    """Validate format_map structure. Returns True if format_map is present and non-empty."""
    format_map = rule.get("format_map")
    if format_map is None:
        return False
    if not isinstance(format_map, dict) or not format_map:
        report.error(f"rule[{index}]: 'format_map' present but empty (remove it or add actions)")
        return False

    for pos_key, actions in format_map.items():
        if pos_key not in VALID_POS_KEYS:
            report.error(f"rule[{index}]: format_map key {pos_key!r} not in {sorted(VALID_POS_KEYS)}")
            continue
        if not isinstance(actions, list) or not actions:
            report.error(f"rule[{index}]: format_map[{pos_key}] must be a non-empty list")
            continue
        for action_no, action in enumerate(actions):
            _check_format_map_entry(action, whitelist, index, pos_key, action_no, report)
    return True


def _check_keywords(rule: dict, index: int, report: ValidationReport) -> bool:
    """Validate keywords structure. Returns True if keywords is present and non-empty."""
    keywords = rule.get("keywords")
    if keywords is None:
        return False
    if not isinstance(keywords, list) or not keywords:
        report.error(f"rule[{index}]: 'keywords' present but empty (remove it or add groups)")
        return False

    for group_no, group in enumerate(keywords):
        if not isinstance(group, list) or not group:
            report.error(f"rule[{index}]: keywords[{group_no}] must be a non-empty list of strings")
            continue
        for item_no, item in enumerate(group):
            if not isinstance(item, str) or not item:
                report.error(f"rule[{index}]: keywords[{group_no}][{item_no}] must be a non-empty string")
    return True


def _check_detection_anchor(
    rule: dict,
    level: str,
    has_format_map: bool,
    has_keywords: bool,
    index: int,
    report: ValidationReport,
) -> None:
    """Enforce text-detection anchor (format_map or keywords) per level.

    - N3/N2/N1: error if neither is present — every higher-level rule must be
      detectable in running text.
    - N5/N4: SUPPRESSED (no error, no warning). N5/N4 carry reference rules
      (basic particles は/が/を/に, categories like 敬語/可能形の文) that are
      pedagogical reference material rather than text-detection targets;
      keywords for basic particles would explode into false positives (は
      occurs in nearly every Japanese sentence). This is an architecturally
      intentional carve-out — do not re-enable without a content review.
    """
    if has_format_map or has_keywords:
        return
    if level in STRICT_LEVELS:
        report.error(
            f"rule[{index}]: neither 'format_map' nor 'keywords' present "
            f"(required for level {level})"
        )


def validate_rule(
    rule: dict,
    whitelist: dict[str, list[str]],
    index: int,
    report: ValidationReport,
) -> tuple[str | None, str | None]:
    """Validate a single rule. Returns ``(level, en_title)`` for cross-rule dedup checks."""
    if not isinstance(rule, dict):
        report.error(f"rule[{index}]: not an object")
        return (None, None)

    _check_ulid(rule.get("rule_id"), report, index)
    level = rule.get("level")
    _check_level(level, report, index)

    en_title, _ru_title = _check_content(rule, level, index, report)
    has_format_map = _check_format_map(rule, whitelist, index, report)
    has_keywords = _check_keywords(rule, index, report)

    if isinstance(level, str):
        _check_detection_anchor(rule, level, has_format_map, has_keywords, index, report)

    return (level if isinstance(level, str) else None, en_title if isinstance(en_title, str) else None)


def _check_duplicate_rule_ids(
    rules: list[dict[str, Any]],
    report: ValidationReport,
) -> None:
    """Hard-fail on repeated ``rule_id`` within the loaded set.

    Catches the common failure of copying a per-rule template without
    regenerating the ULID. The check is positional (first wins) so the second
    and subsequent occurrences are reported with their index.
    """
    seen: dict[str, int] = {}
    for index, rule in enumerate(rules):
        if not isinstance(rule, dict):
            continue
        rule_id = rule.get("rule_id")
        if not isinstance(rule_id, str):
            continue
        if rule_id in seen:
            report.error(
                f"rule[{index}]: duplicate rule_id {rule_id!r} "
                f"also seen at rule[{seen[rule_id]}]"
            )
        else:
            seen[rule_id] = index


def _check_duplicate_titles(
    rules: list[dict[str, Any]],
    report: ValidationReport,
) -> None:
    seen: dict[tuple[str, str], int] = {}
    for index, rule in enumerate(rules):
        if not isinstance(rule, dict):
            continue
        level = rule.get("level")
        content = rule.get("content")
        en_title = content.get("English", {}).get("title") if isinstance(content, dict) else None
        if not isinstance(level, str) or not isinstance(en_title, str) or not en_title:
            continue
        key = (level, en_title)
        if key in seen:
            report.warn(
                f"rule[{index}]: duplicate (level={level}, title={en_title!r}) "
                f"also seen at rule[{seen[key]}] — mutate title to disambiguate"
            )
        else:
            seen[key] = index


def validate_rules(
    rules: list[dict[str, Any]],
    whitelist: dict[str, list[str]],
    report: ValidationReport,
) -> None:
    for index, rule in enumerate(rules):
        validate_rule(rule, whitelist, index, report)
    _check_duplicate_rule_ids(rules, report)
    _check_duplicate_titles(rules, report)


def validate_file(
    path: Path,
    whitelist_source: Path = DEFAULT_WHITELIST_SOURCE,
    report: ValidationReport | None = None,
) -> ValidationReport:
    """Validate a single grammar file. Returns the populated report."""
    report = report or ValidationReport()

    try:
        rules = load_rules(path)
    except (json.JSONDecodeError, ValueError) as exc:
        report.error(f"Failed to load {path}: {exc}")
        return report
    except OSError as exc:
        report.error(f"Cannot read file {path}: {exc}")
        return report

    try:
        whitelist = parse_format_action_whitelist(whitelist_source)
    except (OSError, ValueError) as exc:
        report.error(f"Cannot parse whitelist from {whitelist_source}: {exc}")
        return report

    validate_rules(rules, whitelist, report)
    return report


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate a grammar JSON file against the schema")
    parser.add_argument(
        "--file",
        required=True,
        type=Path,
        help="Path to a grammar JSON file (final store / legacy chunk / single rule) or directory",
    )
    parser.add_argument(
        "--whitelist-source",
        type=Path,
        default=DEFAULT_WHITELIST_SOURCE,
        help="Path to grammar.rs for FormatAction whitelist parsing",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = validate_file(args.file, args.whitelist_source)
    report.emit()
    print(
        f"\n{args.file}: {len(report.errors)} error(s), {len(report.warnings)} warning(s)",
        file=sys.stderr,
    )
    return 0 if report.ok else 1


if __name__ == "__main__":
    sys.exit(main())
