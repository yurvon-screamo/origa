#!/usr/bin/env python3
"""Tier holdout eval via RAW slot readout (my training render; NO system_one
calibration) — the honest lens for the adapter."""

import json
import os
import sys
from collections import Counter
from pathlib import Path

import torch
from peft import PeftModel
from transformers import AutoModel, AutoTokenizer

_GOLDEN_ENV = os.environ.get("D1_GOLDEN")  # d1pkg wipes sys.environ on import

sys.path.insert(0, "/tmp/opencode/lfm-spike/ab")
from d1pkg.prompt import as_question, render  # noqa: E402

MODEL_ID = "LiquidAI/d1-2B"
GOLDEN = Path("/home/yurvon/origa-translator/data/judge/golden_tier_v1.jsonl")
TIER_Q = json.loads(
    Path("/home/yurvon/origa-translator/data/judge/tier_question_v6.json").read_text()
)
LANG_NAME = {
    "ru": "Russian",
    "en": "English",
    "vi": "Vietnamese",
    "ko": "Korean",
    "zh-Hans": "Chinese (Simplified)",
    "zh-Hant-TW": "Traditional Chinese (Taiwan)",
}
TIER_OF = {"DEFECT": 0, "PARTIAL": 1, "OK": 2}


def main():
    adapter = Path(sys.argv[1])
    rows = [json.loads(line) for line in open(GOLDEN, encoding="utf-8")]
    if len(sys.argv) > 2:
        rows = rows[: int(sys.argv[2])]

    tok = AutoTokenizer.from_pretrained(MODEL_ID, trust_remote_code=True)
    tok.truncation_side = "left"
    q = as_question(TIER_Q)
    digits = [tok(v, add_special_tokens=False)["input_ids"][0] for v in "012"]

    model = (
        AutoModel.from_pretrained(
            MODEL_ID, trust_remote_code=True, dtype=torch.bfloat16
        )
        .to("xpu")
        .eval()
    )
    model = PeftModel.from_pretrained(model, adapter).eval()

    confusion = Counter()
    probs_dump = []
    with torch.no_grad():
        for r in rows:
            st = json.dumps(
                {
                    "japanese": r["state"],
                    "translation": r["translation"],
                    "target_locale": f"{r['lang']} ({LANG_NAME.get(r['lang'], r['lang'])})",
                },
                ensure_ascii=False,
            )
            prompt = render(tokenizer=tok, state=st, q=q, bos=tok.bos_token)
            ids = tok(
                prompt,
                truncation=True,
                max_length=448,
                add_special_tokens=False,
                return_tensors="pt",
            ).input_ids.to("xpu")
            out = model(input_ids=ids, use_cache=False)
            logits = out.logits[0, -1]
            p = torch.softmax(logits[digits].float(), -1)
            pred = int(p.argmax())
            gold = TIER_OF[r["tier"]]
            confusion[(gold, pred)] += 1
            probs_dump.append(
                {
                    "lang": r["lang"],
                    "gold": r["tier"],
                    "p": [round(x, 3) for x in p.tolist()],
                }
            )
            if len(probs_dump) % 100 == 0:
                torch.xpu.empty_cache()

    Path("/tmp/opencode/lfm-spike/ab/tier_raw_probs.json").write_text(
        json.dumps(probs_dump, ensure_ascii=False)
    )
    labels = (0, 1, 2)
    print(
        f"RAW-READOUT tier holdout n={len(rows)} adapter={adapter} (0=defect 1=partial 2=exact)"
    )
    print("confusion (gold\\pred): " + "  ".join(f"pred{label}" for label in labels))
    for g in labels:
        print(
            f"  gold{g}: " + "  ".join(f"{confusion.get((g, p), 0):>6}" for p in labels)
        )
    acc = sum(v for (g, p), v in confusion.items() if g == p) / len(rows)
    print(f"accuracy {acc:.1%}")
    for g in labels:
        tot = sum(v for (gg, p), v in confusion.items() if gg == g)
        print(f"  recall gold{g}: {confusion.get((g, g), 0)}/{tot}")


if __name__ == "__main__":
    main()
