#!/usr/bin/env python3
"""Build judge_train_v62 = v5 pairs + tier block v2 (both swarm waves, no oversampling).

v6.1 lesson applied: 1,617 unique non-OK labels (4.8x of v6), OK downsampled to
~1.5x non-OK (no duplication-memorization), stratified (locale x tier) holdout.
"""

import json
import random
from collections import Counter
from pathlib import Path

OUT = Path("/home/yurvon/origa-translator/data/judge")
WAVES = (
    ("/tmp/opencode/lfm-spike/v6_swarm", range(5)),
    ("/tmp/opencode/lfm-spike/v62_swarm", range(10)),
)
rng = random.Random(20261103)

TIER_QUESTION = {
    "type": "score",
    "instructions": "Rate the translation quality: exact, valid with shifts, or defective?",
    "criteria": [
        "Defective: meaning is wrong, hallucinated, inverted, wrong language or script",
        "Valid with shifts: core meaning kept, but notable shifts of tone, register, or nuance",
        "Exact: full meaning preserved accurately",
    ],
}
(OUT / "tier_question_v6.json").write_text(
    json.dumps(TIER_QUESTION, ensure_ascii=False, indent=1)
)
TIER_OF = {"DEFECT": "0", "PARTIAL": "1", "OK": "2"}


def main():
    items = {}  # (wave_dir, id) -> item — id spaces of the two waves OVERLAP, must not merge!
    for d, shards in WAVES:
        for s in shards:
            for r in map(json.loads, open(f"{d}/shard_{s}.jsonl", encoding="utf-8")):
                items[(d, r["id"])] = r

    pool, alarms = [], []
    for d, shards in WAVES:
        for s in shards:
            for c in map(
                json.loads, open(f"{d}/consensus_s{s}.jsonl", encoding="utf-8")
            ):
                it = items[(d, c["id"])]
                row = {
                    "state": it["state"],
                    "lang": it["lang"],
                    "translation": it["translation"],
                    "tier": c["tier"],
                    "level": c["level"],
                    "wave": d.split("/")[-1],
                }
                if c["defect_alarm"]:
                    alarms.append(row)
                elif c["level"] in ("gold", "silver"):
                    pool.append(row)

    # dedup by (lang, translation) safety (waves were built disjoint, belt & suspenders)
    seen, deduped = set(), []
    for r in pool:
        k = (r["lang"], r["translation"])
        if k in seen:
            continue
        seen.add(k)
        deduped.append(r)
    pool = deduped

    # stratified holdout: 10% per (locale, tier) from GOLD only
    holdout, train = [], []
    by_strat = {}
    for r in pool:
        if r["level"] == "gold":
            by_strat.setdefault((r["lang"], r["tier"]), []).append(r)
        else:
            train.append(r)
    for key, rows in by_strat.items():
        rng.shuffle(rows)
        n_hold = max(1, len(rows) // 10)
        holdout += rows[:n_hold]
        train += rows[n_hold:]

    # downsample OK (no oversampling of minorities)
    by_tier = {
        t: [r for r in train if r["tier"] == t] for t in ("OK", "PARTIAL", "DEFECT")
    }
    n_nonok = len(by_tier["PARTIAL"]) + len(by_tier["DEFECT"])
    n_ok = min(len(by_tier["OK"]), int(n_nonok * 1.5))
    rng.shuffle(by_tier["OK"])
    tier_block = by_tier["OK"][:n_ok] + by_tier["PARTIAL"] + by_tier["DEFECT"]
    rng.shuffle(tier_block)

    v5 = [
        json.loads(line)
        for line in open(OUT / "judge_train_v5.jsonl", encoding="utf-8")
    ]
    for r in v5:
        r["question"] = "faithful"
    out = v5 + [
        {
            "state": r["state"],
            "lang": r["lang"],
            "positive": r["translation"],
            "tier": r["tier"],
            "src": f"tier_{r['wave']}",
            "question": "tier",
        }
        for r in tier_block
    ]
    rng.shuffle(out)
    with open(OUT / "judge_train_v62.jsonl", "w", encoding="utf-8") as f:
        for r in out:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    with open(OUT / "golden_tier_v1.jsonl", "w", encoding="utf-8") as f:
        for r in holdout:
            f.write(
                json.dumps(
                    {
                        "state": r["state"],
                        "lang": r["lang"],
                        "translation": r["translation"],
                        "tier": r["tier"],
                        "level": r["level"],
                    },
                    ensure_ascii=False,
                )
                + "\n"
            )
    with open(
        "/tmp/opencode/lfm-spike/v62_swarm/tier_alarms_all.jsonl", "w", encoding="utf-8"
    ) as f:
        for r in alarms:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")

    print(f"pool gold+silver (dedup): {len(pool)} | alarms: {len(alarms)}")
    print(
        f"tier block: {len(tier_block)} = OK {n_ok} + PARTIAL {len(by_tier['PARTIAL'])} + DEFECT {len(by_tier['DEFECT'])}"
    )
    print(
        f"holdout golden_tier_v1 (rebuilt, stratified): {len(holdout)} — {dict(Counter(r['tier'] for r in holdout))}"
    )
    print(f"judge_train_v62: {len(out)} rows (v5 {len(v5)} + tier {len(tier_block)})")


if __name__ == "__main__":
    main()
