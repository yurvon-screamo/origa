#!/usr/bin/env python3
"""Judge a generated rule against its reference facts (OpenRouter, 1 req).

The judge answers STRICT JSON:
    {"ok": true/false, "confidence": 0..1, "issues": ["…"]}

It checks: usage categories match the reference, the formation is
grammatically correct, the Japanese examples actually use the pattern and
their translations (EN/RU first lines) are faithful. Local checkers
already validated schema/form/copy — the judge covers MEANING.

Usage: python scripts/judge_generation.py <rule_id>
"""

import json
import os
import re
import sys
import time
import urllib.request
from pathlib import Path

STAGING = Path("/tmp/opencode/staging")
API = "https://openrouter.ai/api/v1/chat/completions"
MODEL = "stealth/union-alpha"

JUDGE_SYSTEM = """\
You are a strict Japanese-grammar reviewer. You get reference facts and a
generated rule content (all locales). Verify FACTUAL correctness:
- usage categories / meaning matches the reference (nothing invented),
- the formation table is grammatically correct,
- every Japanese example actually uses the target pattern naturally,
- the English and Russian translations of the examples are faithful,
- no fact contradicts the reference.
Ignore style preferences and locale phrasing of KO/VI.
Answer STRICT JSON: {"ok": <bool>, "confidence": <0..1>, "issues": ["<short>"]}"""


def extract_json(content: str) -> dict:
    s = re.sub(r"^```(?:json)?\s*|\s*```$", "", content.strip(), flags=re.S)
    start, end = s.find("{"), s.rfind("}")
    return json.loads(s[start : end + 1])


def main() -> int:
    rid = sys.argv[1]
    ctx = json.loads((STAGING / "contexts" / f"{rid}.json").read_text(encoding="utf-8"))
    gen = json.loads((STAGING / "gen" / f"{rid}.json").read_text(encoding="utf-8"))

    payload_for_judge = {
        "rule": ctx["rule"],
        "detection": ctx["detection"],
        "reference": ctx["reference"],
        "generated": {
            "jp_examples": gen.get("jp_examples"),
            "English": {
                k: gen.get("English", {}).get(k)
                for k in ("short_description", "explanation", "how_to_form")
            },
            "Russian": {
                k: gen.get("Russian", {}).get(k)
                for k in ("short_description", "explanation")
            },
        },
    }
    user = (
        "Review this generated grammar rule against its reference facts:\n\n"
        + json.dumps(payload_for_judge, ensure_ascii=False, indent=1)
    )

    req = urllib.request.Request(
        API,
        data=json.dumps(
            {
                "model": MODEL,
                "messages": [
                    {"role": "system", "content": JUDGE_SYSTEM},
                    {"role": "user", "content": user},
                ],
                "temperature": 0.1,
                "max_tokens": 1200,
            }
        ).encode(),
        headers={
            "Authorization": f"Bearer {os.environ['OPENROUTER_API_KEY']}",
            "Content-Type": "application/json",
        },
    )
    t0 = time.time()
    try:
        with urllib.request.urlopen(req, timeout=300) as resp:
            body = json.loads(resp.read())
        verdict = extract_json(body["choices"][0]["message"]["content"])
    except Exception as e:  # noqa: BLE001
        with open(STAGING / "request_log.jsonl", "a", encoding="utf-8") as f:
            f.write(json.dumps({"ts": time.time(), "kind": "judge", "rule_id": rid,
                                "ok": False, "info": str(e)[:200]}) + "\n")
        print(f"JUDGE FAILED: {e}", file=sys.stderr)
        return 1

    out = STAGING / "judge" / f"{rid}.json"
    out.parent.mkdir(exist_ok=True)
    out.write_text(json.dumps(verdict, ensure_ascii=False, indent=1), encoding="utf-8")
    with open(STAGING / "request_log.jsonl", "a", encoding="utf-8") as f:
        f.write(json.dumps({"ts": time.time(), "kind": "judge", "rule_id": rid,
                            "ok": verdict.get("ok", False),
                            "info": f"conf={verdict.get('confidence')}"}) + "\n")
    print(f"verdict ({time.time()-t0:.0f}s): {json.dumps(verdict, ensure_ascii=False)}")
    return 0 if verdict.get("ok") else 2


if __name__ == "__main__":
    sys.exit(main())
