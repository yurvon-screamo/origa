#!/usr/bin/env python3
"""Rebuild every dataset of the origa-translator project from the source pools.

Pools (data/sft_mono_{en,vi,ru,ko}_full.jsonl, 155,728 rows each) already
exclude the holdout. Everything below is deterministic (fixed seeds).

Outputs:
  sft_multi_c150k.jsonl  600k train set (150k per locale)   <- final model
  sft_multi_c10k.jsonl    40k train set (10k per locale)    <- 40k baseline
  sft_multi_500.jsonl     500 smoke set (125 per locale)
  bench_4k.jsonl          4k bench set (1000 per locale)
  test_20x4 / test_50x4   eval sets built from the holdout
"""

import json
import random
from pathlib import Path

DATA = Path(__file__).resolve().parent.parent / "data"
LOCALES = ("en", "vi", "ru", "ko")


def load(p):
    return [json.loads(l) for l in open(p, encoding="utf-8")]


def locale_of(row):
    u = row["messages"][1]["content"]
    for code, name in (("vi", "Vietnamese"), ("ko", "Korean"), ("ru", "Russian")):
        if name in u:
            return code
    return "en"


def main():
    pools = {L: load(DATA / f"sft_mono_{L}_full.jsonl") for L in LOCALES}
    for L in LOCALES:
        print(f"{L}: {len(pools[L])} rows in pool")

    # train sets: take first N per locale, then shuffle with fixed seed
    for per, fname in (
        (150_000, "sft_multi_c150k.jsonl"),
        (10_000, "sft_multi_c10k.jsonl"),
        (125, "sft_multi_500.jsonl"),
    ):
        out = []
        for L in LOCALES:
            out += pools[L][:per]
        random.Random(43).shuffle(out)
        with open(DATA / fname, "w", encoding="utf-8") as f:
            for r in out:
                f.write(json.dumps(r, ensure_ascii=False) + "\n")
        print(f"{fname}: {len(out)} rows")

    # bench 4k: random 1000/locale from the c10k pool
    c10 = load(DATA / "sft_multi_c10k.jsonl")
    by = {L: [] for L in LOCALES}
    for r in c10:
        by[locale_of(r)].append(r)
    out = []
    for L in LOCALES:
        random.Random(42).shuffle(by[L])
        out += by[L][:1000]
    random.Random(43).shuffle(out)
    with open(DATA / "bench_4k.jsonl", "w", encoding="utf-8") as f:
        for r in out:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(f"bench_4k.jsonl: {len(out)} rows")

    # eval sets from the holdout: first N per locale by id
    hold = load(DATA / "holdout_500x4.jsonl")
    for n, fname in ((20, "test_20x4.jsonl"), (50, "test_50x4.jsonl")):
        out = []
        for L in LOCALES:
            out += sorted([r for r in hold if r["lang"] == L], key=lambda r: r["i"])[:n]
        with open(DATA / fname, "w", encoding="utf-8") as f:
            for r in out:
                f.write(json.dumps(r, ensure_ascii=False) + "\n")
        print(f"{fname}: {len(out)} rows")


if __name__ == "__main__":
    main()
