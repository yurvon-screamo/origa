"""Split grammar.json md_description into named section blocks.

Migrates each grammar rule's monolithic ``md_description`` field (per language)
into 5–6 separate named fields:

- explanation    (📖)
- how_to_form    (🔧)
- examples       (🗣️)
- nuances        (💡)
- pro_tip        (🌸)
- related_patterns (🔄, optional)

The ``md_description`` key is removed from the output.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

GRAMMAR_PATH = Path(__file__).parent.parent / "cdn" / "grammar" / "grammar.json"

EMOJI_TO_FIELD: dict[str, str] = {
    "\U0001f4d6": "explanation",       # 📖
    "\U0001f527": "how_to_form",       # 🔧
    "\U0001f5e3": "examples",          # 🗣 (base char, works with or without FE0F)
    "\U0001f4a1": "nuances",           # 💡
    "\U0001f338": "pro_tip",           # 🌸
    "\U0001f504": "related_patterns",  # 🔄
}

STANDARD_FIELDS: set[str] = {"explanation", "how_to_form", "examples", "nuances", "pro_tip"}

SECTION_SPLIT_RE = re.compile(r"^### ", re.MULTILINE)


def identify_field(heading_line: str) -> str | None:
    """Return the field name for a ``### <emoji> ...`` heading, or *None*."""
    for emoji, field in EMOJI_TO_FIELD.items():
        if emoji in heading_line[:10]:
            return field
    return None


def split_md_description(md: str) -> dict[str, str]:
    """Split a single ``md_description`` into ``{field: content}`` dict."""
    parts = SECTION_SPLIT_RE.split(md)

    sections: dict[str, str] = {}

    for part in parts:
        stripped = part.strip()
        if not stripped:
            continue

        first_line = stripped.split("\n", 1)[0]
        field = identify_field(first_line)
        if field is None:
            # Preamble (H2 title) or unknown — skip
            continue

        sections[field] = stripped.rstrip()

    return sections


def transform_rule(rule: dict) -> dict:
    """Transform a single grammar rule: split all language ``md_description`` values."""
    content: dict = rule["content"]

    for lang_data in content.values():
        if "md_description" not in lang_data:
            continue

        sections = split_md_description(lang_data["md_description"])
        del lang_data["md_description"]
        lang_data.update(sections)

    return rule


def validate_report(data: dict) -> bool:
    """Print a validation report and return *False* if any rule is invalid."""
    rules: list[dict] = data["grammar"]
    total = len(rules)
    print(f"\n{'=' * 60}")
    print(f"Total rules processed: {total}")
    print(f"{'=' * 60}\n")

    has_error = False
    rules_with_related: list[str] = []
    variant_wrench: list[tuple[str, str, str]] = []  # (rule_id, lang, header)

    for idx, rule in enumerate(rules):
        rule_id: str = rule["rule_id"]
        content: dict = rule["content"]

        for lang, lang_data in content.items():
            # Check which fields are present
            found_fields: set[str] = set()
            empty_fields: list[str] = []
            missing_fields: list[str] = []

            for field in STANDARD_FIELDS:
                value = lang_data.get(field, "")
                if not value or not value.strip():
                    missing_fields.append(field)
                else:
                    found_fields.add(field)
                    # Check if section body is empty (header only)
                    lines = value.split("\n")
                    if len(lines) <= 1 or not "\n".join(lines[1:]).strip():
                        empty_fields.append(field)

            if "related_patterns" in lang_data:
                val = lang_data["related_patterns"]
                if val and val.strip():
                    rules_with_related.append(f"{rule_id} ({lang})")

            # Detect variant 🔧 headers
            how_to_form_val = lang_data.get("how_to_form", "")
            if how_to_form_val:
                header = how_to_form_val.split("\n", 1)[0]
                # Standard headers contain "How to form" or "Как образуется"
                is_standard = (
                    "How to form" in header or "Как образуется" in header
                )
                if not is_standard:
                    variant_wrench.append((rule_id, lang, header))

            # Report per-rule issues
            issues: list[str] = []
            if missing_fields:
                issues.append(f"MISSING: {', '.join(missing_fields)}")
            if empty_fields:
                issues.append(f"EMPTY: {', '.join(empty_fields)}")

            if issues:
                has_error = True
                print(f"  ❌ Rule {idx} ({rule_id}) [{lang}]: {'; '.join(issues)}")
            elif found_fields and len(found_fields) == len(STANDARD_FIELDS):
                pass  # OK, silent
            else:
                has_error = True
                print(
                    f"  ❌ Rule {idx} ({rule_id}) [{lang}]: "
                    f"found {len(found_fields)}/{len(STANDARD_FIELDS)} standard sections"
                )

    # Summary
    print(f"\n{'─' * 40}")
    print(f"Rules with related_patterns (🔄): {len(rules_with_related)}")
    for entry in rules_with_related:
        print(f"  🔄 {entry}")

    print(f"\nVariant 🔧 headers: {len(variant_wrench)}")
    for rule_id, lang, header in variant_wrench:
        print(f"  🔧 {rule_id} [{lang}]: {header}")

    if has_error:
        print(f"\n❌ VALIDATION FAILED — some rules have missing or empty sections.")
    else:
        print(f"\n✅ All {total} rules validated successfully.")

    return not has_error


def main() -> None:
    if not GRAMMAR_PATH.exists():
        print(f"Error: {GRAMMAR_PATH} not found.", file=sys.stderr)
        sys.exit(1)

    data: dict = json.loads(GRAMMAR_PATH.read_text(encoding="utf-8"))

    rules: list[dict] = data["grammar"]
    for rule in rules:
        transform_rule(rule)

    GRAMMAR_PATH.write_text(
        json.dumps(data, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )

    ok = validate_report(data)
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
