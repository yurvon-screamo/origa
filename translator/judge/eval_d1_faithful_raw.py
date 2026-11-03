#!/usr/bin/env python3
"""Faithful (golden) eval via RAW slot readout — bypasses system_one and its
stock calibration.json (fit for base d1, distorting under our adapter).
Protocol identical to previous evals: per candidate noul(Yes/No token probs),
pair wins when good > bad."""
import json
import os
import sys
from pathlib import Path

_GOLDEN_ENV = os.environ.get("D1_GOLDEN")  # d1pkg wipes sys.environ on import

import torch
from peft import PeftModel
from transformers import AutoModel, AutoTokenizer

sys.path.insert(0, "/tmp/opencode/lfm-spike/ab")
from d1pkg.prompt import as_question, render  # noqa: E402

MODEL_ID = "LiquidAI/d1-2B"
GOLDEN = Path("/home/yurvon/origa-translator/data/judge/golden_v4c.jsonl")
FAITHFUL_Q = {
    "type": "noul",
    "instructions": (
        "Is the translation a faithful rendering of the Japanese text into "
        "the target locale (correct meaning, no hallucination, no omission, "
        "no wrong language)?"
    ),
    "criteria": {
        "false": "The translation is NOT faithful: wrong meaning, hallucinated or "
                 "dropped details, meaning inversion, wrong names, or wrong language",
        "true": "The translation IS faithful: exact meaning preserved, nothing "
                "invented or dropped, correct names, fully in the target language",
    },
}
LANG_NAME = {"ru": "Russian", "en": "English", "vi": "Vietnamese", "ko": "Korean",
             "zh-Hans": "Chinese (Simplified)", "zh-Hant-TW": "Traditional Chinese (Taiwan)"}


def main():
    adapter = Path(sys.argv[1])
    golden = Path(_GOLDEN_ENV or GOLDEN)
    pairs = [json.loads(l) for l in open(golden, encoding="utf-8")]
    limit = int(sys.argv[2]) if len(sys.argv) > 2 else 0
    if limit:
        pairs = pairs[:limit]

    tok = AutoTokenizer.from_pretrained(MODEL_ID, trust_remote_code=True)
    tok.truncation_side = "left"
    q = as_question(FAITHFUL_Q)
    yes_id = tok("Yes", add_special_tokens=False)["input_ids"][0]
    no_id = tok("No", add_special_tokens=False)["input_ids"][0]

    model = AutoModel.from_pretrained(MODEL_ID, trust_remote_code=True,
                                      dtype=torch.bfloat16).to("xpu").eval()
    model = PeftModel.from_pretrained(model, adapter).eval()

    def p_yes(state, translation, lang):
        st = json.dumps({"japanese": state, "translation": translation,
                         "target_locale": f"{lang} ({LANG_NAME.get(lang, lang)})"},
                        ensure_ascii=False)
        prompt = render(tokenizer=tok, state=st, q=q, bos=tok.bos_token)
        ids = tok(prompt, truncation=True, max_length=448, add_special_tokens=False,
                  return_tensors="pt").input_ids.to("xpu")
        out = model(input_ids=ids, use_cache=False)
        logits = out.logits[0, -1]
        p = torch.softmax(logits[[no_id, yes_id]].float(), -1)
        return p[1].item()

    results = {}
    with torch.no_grad():
        for i, p in enumerate(pairs):
            for w in ("good", "bad"):
                results[(i, w)] = p_yes(p["state"], p[w], p["lang"])
            if i % 200 == 0:
                torch.xpu.empty_cache()

    wins = sum(1 for i in range(len(pairs)) if results[(i, "good")] > results[(i, "bad")])
    good = [results[(i, "good")] for i in range(len(pairs))]
    bad = [results[(i, "bad")] for i in range(len(pairs))]
    print(f"[{adapter}] RAW pairs {len(pairs)} acc {wins}/{len(pairs)} = {wins/len(pairs)*100:.1f}%  "
          f"good {sum(good)/len(good):.3f} bad {sum(bad)/len(bad):.3f}")
    by = {}
    for i, p in enumerate(pairs):
        by.setdefault(p["lang"], []).append(results[(i, "good")] > results[(i, "bad")])
    print("  " + " · ".join(f"{lg} {sum(ws)}/{len(ws)}" for lg, ws in sorted(by.items())))


if __name__ == "__main__":
    main()
