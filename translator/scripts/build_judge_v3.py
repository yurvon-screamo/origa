#!/usr/bin/env python3
"""Judge v3: add programmatic inversion negatives (the exact blind-fail types).

Two synthetic negative families on top of v2:
  1. possessive/person inversions: swap your<->my, твой<->мой, ваш<->свой,
     он<->я, she<->I etc. in the GOLD translation (keeps language natural,
     flips the addressee — the exact ланч-бокс failure)
  2. paren-meta contamination: append "(вероятно, имеется в виду ...)"-style
     junk to gold — the exact нава failure pattern
"""
import json
import os
import random
import re
from pathlib import Path

DATA = Path(os.environ.get("ORIGA_JUDGE_DATA", "data/judge"))
rng = random.Random(77)

SWAPS = {
    "ru": [("ваш ", "свой "), ("Ваш ", "Свой "), ("твою ", "мою "), ("твой ", "мой "),
           ("Ты ", "Я "), ("ты ", "я "), ("он ", "я "), ("она ", "я "), ("ему ", "мне ")],
    "en": [("your ", "my "), ("Your ", "My "), ("you ", "I "), ("You ", "I "),
           ("he ", "I "), ("she ", "I "), ("his ", "my "), ("her ", "my ")],
    "vi": [("bạn ", "tôi "), ("anh ", "tôi "), ("cậu ", "tôi "), ("cô ấy ", "tôi "),
           ("anh ấy ", "tôi "), ("của bạn", "của tôi")],
    "ko": [("너의 ", "내 "), ("당신의 ", "내 "), ("그녀의 ", "내 "), ("그의 ", "내 "),
           ("오빠의 ", "내 "), ("형의 ", "내 ")],
}
META_JUNK = {
    "ru": [" (вероятно, имеется в виду какой-то предмет)",
           " (возможно, это название)", " (так в оригинале)"],
    "en": [" (probably some object)", " (possibly a name)", " (as in the original)"],
    "vi": [" (có lẽ là một vật gì đó)", " (có thể là một cái tên)"],
    "ko": [" (아마 어떤 물건을 의미)", " (아마 이름일 것)"],
}


def main():
    rows = [json.loads(l) for l in open(DATA / "judge_train_v2.jsonl", encoding="utf-8")]
    extra = []
    n_inv = n_meta = 0
    for r in rows:
        if r.get("src") != "synthetic":
            continue                      # phrase rows only (word rows have no pronouns)
        pos = r["positive"]
        # 1) inversions
        for a, b in SWAPS.get(r["lang"], []):
            if a in pos:
                neg = pos.replace(a, b, 1)
                if neg != pos:
                    extra.append({"state": r["state"], "lang": r["lang"],
                                  "positive": pos, "negatives": [neg],
                                  "src": "inversion"})
                    n_inv += 1
                break
        # 2) paren-meta
        if rng.random() < 0.25:
            junk = rng.choice(META_JUNK[r["lang"]])
            extra.append({"state": r["state"], "lang": r["lang"],
                          "positive": pos, "negatives": [pos + junk],
                          "src": "meta_junk"})
            n_meta += 1
    all_rows = rows + extra
    rng.shuffle(all_rows)
    with open(DATA / "judge_train_v3.jsonl", "w", encoding="utf-8") as f:
        for r in all_rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    from collections import Counter
    print(f"v3: {len(all_rows)} rows | {dict(Counter(r.get('src') for r in all_rows))}")
    print(f"added: {n_inv} inversions, {n_meta} meta-junk")


if __name__ == "__main__":
    main()
