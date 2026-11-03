#!/usr/bin/env python3
"""Swarm consensus aggregator (fixed retroactively — review finding H1).

The exact logic used to build consensus_s{N}.jsonl from out_f{1..5}_s{N}.jsonl
during waves 1-2 (it lived inline in shell; this is its canonical form).

Rules (verified against the existing consensus files byte-for-byte):
- 5 votes per item (frames 1-5)
- tier = majority tier; level: gold=5/5, silver=4/5, weak=3/5, split=<3
- defect_alarm = DEFECT voted >=1 AND majority tier != DEFECT

Usage: aggregate_consensus.py <swarm_dir> <n_shards>
"""

import json
import sys
from collections import Counter
from pathlib import Path


def aggregate(swarm_dir: Path, s: int):
    inp = {
        r["id"]: r
        for r in map(json.loads, open(swarm_dir / f"shard_{s}.jsonl", encoding="utf-8"))
    }
    frames = {
        f: {
            r["id"]: r["tier"]
            for r in map(
                json.loads, open(swarm_dir / f"out_f{f}_s{s}.jsonl", encoding="utf-8")
            )
        }
        for f in range(1, 6)
    }
    out = []
    for i in inp:
        votes = Counter(frames[f][i] for f in frames)
        tier, top = votes.most_common(1)[0]
        vd = {t: votes.get(t, 0) for t in ("OK", "PARTIAL", "DEFECT")}
        level = {5: "gold", 4: "silver", 3: "weak"}.get(top, "split")
        out.append(
            {
                "id": i,
                "lang": inp[i]["lang"],
                "tier": tier,
                "votes": vd,
                "level": level,
                "defect_alarm": vd["DEFECT"] > 0 and tier != "DEFECT",
            }
        )
    with open(swarm_dir / f"consensus_s{s}.jsonl", "w", encoding="utf-8") as fh:
        for c in out:
            fh.write(json.dumps(c, ensure_ascii=False) + "\n")
    return out


if __name__ == "__main__":
    d, n = Path(sys.argv[1]), int(sys.argv[2])
    for s in range(n):
        rows = aggregate(d, s)
        print(f"{d.name}/s{s}: {len(rows)} items", flush=True)
