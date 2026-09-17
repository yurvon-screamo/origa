#!/usr/bin/env python3
"""Generate rule content in 4 locales via OpenRouter (smoke: 1 rule).

One request = one rule = all four locales (budget-efficient: 677 requests
for the whole corpus out of the 1000-request union-alpha quota).

Usage:
    python scripts/generate_content.py --rule 01KV2C261V6FFAJNQRWK0WAE63 [--dry-run]
    python scripts/generate_content.py --pattern '～ながら' --level N2 --dry-run

Every call is appended to staging/request_log.jsonl (kind=generate) so the
quota is trackable. Output: staging/gen/<rule_id>.json (the model's JSON).
"""

import argparse
import json
import os
import re
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

STAGING = Path("/tmp/opencode/staging")
API = "https://openrouter.ai/api/v1/chat/completions"
MODEL = "stealth/union-alpha"

SYSTEM_PROMPT = """\
You are a Japanese-grammar textbook editor for a spaced-repetition app.
You rewrite ONE grammar rule's content in four locales (English, Russian,
Korean, Vietnamese), grounded in verified reference facts.

Absolute rules:
1. Write your OWN Japanese example sentences — never copy reference
   examples verbatim (different topic/vocabulary, same grammar).
2. Every Japanese example MUST contain the rule's grammatical form — the
   "detection" field lists keywords; every example has to include at
   least one keyword from EVERY group.
3. Translations are accurate and natural in each locale; all four locales
   carry the same semantic skeleton (same facts, same example set).
4. The explanation follows the reference's usage categories; keep the
   register of the level (N1 bookish, N5 simple).
5. Answer with STRICT JSON only — no markdown fences, no commentary.
"""

OUTPUT_SPEC = """\
Output JSON schema (exactly these keys):
{
  "rule_id": "<the given rule_id>",
  "jp_examples": ["<3-5 original Japanese sentences using the pattern>"],
  "English": {
    "short_description": "<=90 chars, plain meaning headline",
    "explanation": "markdown, 400-900 chars, structured by usage categories",
    "how_to_form": "markdown table: connection patterns with short examples",
    "examples": "for EACH jp_example a fenced block:\\n```\\n<japanese>\\n<translation>\\n```\\nseparated by blank lines",
    "nuances": {"common_mistakes": ["<1-3>"], "notes": ["<1-4>"]},
    "pro_tip": "<memorable usage tip>"
  },
  "Russian": { same fields, Russian },
  "Korean": { same fields, Korean },
  "Vietnamese": { same fields, Vietnamese }
}
"""


def build_user_prompt(ctx: dict) -> str:
    focus = (ctx.get("current", {}).get("English", {}) or {}).get("short_description", "")
    focus_block = (
        f'\nIMPORTANT — RULE FOCUS: this rule covers ONLY this aspect of the '
        f'pattern: "{focus}". The pattern is shared by sibling rules '
        f'(other aspects are separate cards). Do NOT drift into sibling '
        f'usages: every example and the whole explanation must illustrate '
        f'exactly this focus.\n'
        if focus
        else ""
    )
    return (
        "Rewrite this grammar rule. Context follows as JSON:\n\n"
        + json.dumps(ctx, ensure_ascii=False, indent=1)
        + "\n"
        + focus_block
        + "\n"
        + OUTPUT_SPEC
    )


def call_model(system: str, user: str) -> dict:
    payload = json.dumps(
        {
            "model": MODEL,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
            "temperature": float(os.environ.get("GEN_TEMPERATURE", "0.4")),
            "max_tokens": 6000,
        }
    ).encode()
    req = urllib.request.Request(
        API,
        data=payload,
        headers={
            "Authorization": f"Bearer {os.environ['OPENROUTER_API_KEY']}",
            "Content-Type": "application/json",
        },
    )
    with urllib.request.urlopen(req, timeout=300) as resp:
        body = json.loads(resp.read())
    content = body["choices"][0]["message"]["content"]
    return {"content": content, "usage": body.get("usage", {})}


def extract_json(content: str) -> dict:
    s = content.strip()
    s = re.sub(r"^```(?:json)?\s*|\s*```$", "", s, flags=re.S)
    start, end = s.find("{"), s.rfind("}")
    return json.loads(s[start : end + 1])


def log_request(kind: str, rule_id: str, ok: bool, info: str = "") -> None:
    with open(STAGING / "request_log.jsonl", "a", encoding="utf-8") as f:
        f.write(
            json.dumps(
                {"ts": time.time(), "kind": kind, "rule_id": rule_id, "ok": ok, "info": info},
                ensure_ascii=False,
            )
            + "\n"
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rule", help="rule_id")
    parser.add_argument("--pattern", help="find rule by EN pattern")
    parser.add_argument("--level", default=None)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    corpus = json.loads((STAGING / "grammar_v2.pruned.json").read_text(encoding="utf-8"))
    rules = corpus["grammar"]
    rule = None
    if args.rule:
        rule = next((r for r in rules if r["rule_id"] == args.rule), None)
    elif args.pattern:
        rule = next(
            (
                r
                for r in rules
                if r["content"]["English"].get("pattern") == args.pattern
                and (args.level is None or r["level"] == args.level)
            ),
            None,
        )
    if not rule:
        print("rule not found", file=sys.stderr)
        return 1
    rid = rule["rule_id"]

    ctx = json.loads((STAGING / "contexts" / f"{rid}.json").read_text(encoding="utf-8"))
    user_prompt = build_user_prompt(ctx)

    if args.dry_run:
        print(f"--- SYSTEM ---\n{SYSTEM_PROMPT}\n--- USER ({len(user_prompt)} chars) ---")
        print(user_prompt[:3000] + "\n…")
        return 0

    t0 = time.time()
    try:
        result = call_model(SYSTEM_PROMPT, user_prompt)
        parsed = extract_json(result["content"])
    except (urllib.error.URLError, json.JSONDecodeError, KeyError) as e:
        log_request("generate", rid, False, str(e)[:200])
        print(f"FAILED: {e}", file=sys.stderr)
        return 1

    gen_dir = STAGING / "gen"
    gen_dir.mkdir(exist_ok=True)
    (gen_dir / f"{rid}.json").write_text(
        json.dumps(parsed, ensure_ascii=False, indent=1), encoding="utf-8"
    )
    log_request("generate", rid, True, json.dumps(result.get("usage", {})))
    print(
        f"ok: {rid} in {time.time()-t0:.0f}s | tokens: {result['usage']} "
        f"-> staging/gen/{rid}.json"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
