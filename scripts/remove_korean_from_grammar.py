"""Remove Korean (Hangul) artifacts from grammar rules (#178 W-11).

LLM-generated grammar content occasionally has Korean words mixed into Japanese
example sentences. These are translation artifacts (Korean was apparently used
as a pivot language during generation) and never belonged in a Japanese learning
app.

This script applies curated, hand-verified replacements keyed by rule_id. Each
replacement is the minimum needed to restore the intended Japanese; surrounding
text is left untouched. Both grammar.json (compiled/deployed form) and the
per-rule files under rules/* (source form) are updated so the fix survives
recompilation.

Run:
    python scripts/remove_korean_from_grammar.py --cdn cdn
    python scripts/remove_korean_from_grammar.py --cdn cdn --dry-run
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

# Per-rule replacements. Keys are rule_ids; values are (old, new) substring
# pairs applied to every string value in that rule (across all languages and
# fields, including nested examples / nuances / how_to_form).
#
# Verified meanings (Hangul → intended Japanese):
#   학교 (Korean "school")      → 学校
#   인가 (Korean question ptcl) → か (the actual particle this rule teaches)
#   사람 (Korean "person")      → 人
#   무엇 (Korean "what")        → 何
#   같습니다 (Korean "is same") → 同じです
#   금은 (Korean "money+topic") → the source had お금は mixing Japanese honorific
#                                  o- with Korean 금(geum)=money; correct is お金は
#   있었습니다 (Korean past "was")  → ありました (the verb this rule conjugates)
#   있습니다 (Korean "is/exist")   → あります (the verb the rule warns against)
#   차는 (Korean "car+topic")   → 車は
#   공부 (Korean "study")       → 勉強
REPLACEMENTS: dict[str, list[tuple[str, str]]] = {
    "01G00000000000000028000000": [("학교", "学校")],
    "01G0000000000000004W000000": [("인가", "か")],
    "01G0000000000000008G000000": [("사람", "人")],
    "01G00000000000000090000000": [("무엇", "何")],
    "01G0000000000000009R000000": [("같습니다", "同じです")],
    "01G000000000000000J0000000": [("お금은", "お金は")],
    "01G000000000000000KC000000": [("있었습니다", "ありました")],
    "01G000000000000000MC000000": [("있습니다", "あります")],
    "01G000000000000000MM000000": [("차는", "車は")],
    "01G000000000000000SR000000": [("공부", "勉強")],
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cdn",
        required=True,
        help="Path to the cdn/ directory (containing grammar/grammar.json and grammar/rules/)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show planned replacements without writing files",
    )
    return parser.parse_args()


def apply_replacements_to_value(value: Any, replacements: list[tuple[str, str]]) -> Any:
    """Recursively apply substring replacements inside strings within value."""
    if isinstance(value, str):
        new = value
        for old, repl in replacements:
            new = new.replace(old, repl)
        return new
    if isinstance(value, dict):
        return {k: apply_replacements_to_value(v, replacements) for k, v in value.items()}
    if isinstance(value, list):
        return [apply_replacements_to_value(v, replacements) for v in value]
    return value


def apply_to_rule(rule: dict, log_prefix: str) -> int:
    """Apply this rule's replacements in place. Returns count of fields changed."""
    rule_id = rule.get("rule_id", "?")
    replacements = REPLACEMENTS.get(rule_id)
    if not replacements:
        return 0
    fields_changed = 0
    for key, val in list(rule.items()):
        new_val = apply_replacements_to_value(val, replacements)
        if new_val != val:
            rule[key] = new_val
            fields_changed += 1
    if fields_changed:
        print(f"  {log_prefix} rule_id={rule_id}: {fields_changed} top-level field(s) updated")
    return fields_changed


def process_grammar_json(path: Path, dry_run: bool) -> int:
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    rules = data.get("grammar", [])
    changed = sum(apply_to_rule(rule, "grammar.json") for rule in rules)
    if changed and not dry_run:
        with open(path, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, separators=(",", ":"))
    return changed


def process_rules_dir(rules_dir: Path, dry_run: bool) -> int:
    total_changed = 0
    for path in sorted(rules_dir.rglob("*.json")):
        with open(path, encoding="utf-8") as f:
            data = json.load(f)
        before = json.dumps(data, ensure_ascii=False)
        if isinstance(data, dict) and "rule_id" in data:
            apply_to_rule(data, f"rules/{path.relative_to(rules_dir)}")
        elif isinstance(data, dict) and "grammar" in data:
            for rule in data["grammar"]:
                apply_to_rule(rule, f"rules/{path.relative_to(rules_dir)}")
        after = json.dumps(data, ensure_ascii=False)
        if before != after:
            total_changed += 1
            if not dry_run:
                with open(path, "w", encoding="utf-8") as f:
                    json.dump(data, f, ensure_ascii=False, separators=(",", ":"))
    return total_changed


def main() -> int:
    args = parse_args()
    cdn = Path(args.cdn)
    grammar_path = cdn / "grammar" / "grammar.json"
    rules_dir = cdn / "grammar" / "rules"
    if not grammar_path.exists():
        print(f"Error: {grammar_path} not found")
        return 1
    if not rules_dir.exists():
        print(f"Error: {rules_dir} not found")
        return 1

    print(f"Processing {grammar_path}...")
    grammar_changed = process_grammar_json(grammar_path, args.dry_run)
    print(f"Processing {rules_dir}...")
    rules_changed = process_rules_dir(rules_dir, args.dry_run)

    print()
    print(f"grammar.json top-level fields modified: {grammar_changed}")
    print(f"rules/* files modified: {rules_changed}")
    if args.dry_run:
        print("\n--dry-run: no files modified.")
    else:
        print("\nReplacements applied.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
