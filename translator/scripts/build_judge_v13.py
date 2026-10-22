#!/usr/bin/env python3
"""Build judge_train_v13: v11 + production-fix pairs (phrases + dictionary).

New real-world pairs from the deployed cleanups:
  phrases: 2 066 (state=JP, positive=cleaned gen/gold, negative=old defective)
  dict jp_in_t: 1 246 (state=word, positive=new_t list items, negatives=old items)
  dict low_pct: 447 (same shape; old wrong meaning -> correct one)
Leak guard vs golden_v4c states. Balance: new pairs capped at 30% share.
"""
import json
import random
from pathlib import Path

DATA = Path("/home/yurvon/origa-translator/data")
OUT = DATA / "judge"
PIPE = DATA / "pipeline"
rng = random.Random(20261019)


def main():
    v11 = [json.loads(l) for l in open(OUT / "judge_train_v11.jsonl", encoding="utf-8")]
    golden_states = {(json.loads(l)["state"], json.loads(l)["lang"])
                     for l in open(OUT / "golden_v4c.jsonl", encoding="utf-8")}

    pairs = []

    # 1) phrase fixes: one line per fix
    for l in open(PIPE / "cdn_fixes_suspects.jsonl", encoding="utf-8"):
        f = json.loads(l)
        if f["action"] != "fix":
            continue
        # find the phrase JP by id
        pairs.append({"kind": "phrase", "i": f["i"], "lang": f["lang"],
                      "good": f["new"], "bad": f["old"]})

    # 2) dict jp_in_t (final reviewed)
    jpint = json.load(open(PIPE / "dict_jpint_final.json", encoding="utf-8"))
    for k, new_t in jpint.items():
        w, L = k.split("|", 1)
        pairs.append({"kind": "dict_jpint", "i": w, "lang": L, "new_t": new_t})

    # old_t для словарей: из reviewed (old_t там есть)
    reviewed_old = {}
    for l in open(PIPE / "dict_jpint_reviewed.jsonl", encoding="utf-8"):
        r = json.loads(l)
        reviewed_old[(r["word"], r["lang"])] = r["old_t"]

    # 3) dict low_pct fixes
    for l in open(PIPE / "dict_lowpct_fixes.jsonl", encoding="utf-8"):
        f = json.loads(l)
        if f["verdict"] != "fix":
            continue
        pairs.append({"kind": "dict_lowpct", "i": f["word"], "lang": f["lang"],
                      "new_t": f["new_t"], "old_t": f["old_t"]})

    print(f"сырых пар: {len(pairs)} "
          f"(phrase {sum(1 for p in pairs if p['kind']=='phrase')}, "
          f"dict_jpint {sum(1 for p in pairs if p['kind']=='dict_jpint')}, "
          f"dict_lowpct {sum(1 for p in pairs if p['kind']=='dict_lowpct')})")

    # phrase -> нужны JP-тексты (state): подтянуть из cdn по i
    import glob
    id2x = {}
    for f in sorted(glob.glob("/home/yurvon/origa/cdn/phrases/data/p*.json")):
        for r in json.load(open(f, encoding="utf-8")):
            id2x[r["i"]] = r["x"]

    rows = []
    skipped = 0
    for p in pairs:
        if p["kind"] == "phrase":
            x = id2x.get(p["i"])
            if not x or p["good"].strip() == p["bad"].strip():
                skipped += 1
                continue
            if (x, p["lang"]) in golden_states:
                skipped += 1
                continue
            rows.append({"state": x, "lang": p["lang"], "positive": p["good"],
                         "negatives": [p["bad"]], "src": "prodfix_phrase"})
        else:
            state = f"Japanese word: {p['i']}"
            old = p.get("old_t") or reviewed_old.get((p["i"], p["lang"]))
            if not old or not p["new_t"]:
                skipped += 1
                continue
            if (state, p["lang"]) in golden_states:
                skipped += 1
                continue
            # positive: лучший чистый вариант; negatives: старые (реально дефектные)
            old_items = [t for t in old if t and t.strip()]
            new_items = [t for t in p["new_t"] if t and t.strip()]
            bad = [t for t in old_items if t not in new_items][:3]
            if not new_items or not bad:
                skipped += 1
                continue
            rows.append({"state": state, "lang": p["lang"],
                         "positive": new_items[0], "negatives": bad,
                         "src": f"prodfix_{p['kind']}"})
    print(f"пар после фильтров: {len(rows)} | пропущено: {skipped}")

    cap = int(len(v11) * 30 / 70)
    rng.shuffle(rows)
    take = rows[:cap]
    print(f"баланс-кап {cap} -> берём {len(take)}")

    out = list(v11) + take
    rng.shuffle(out)
    with open(OUT / "judge_train_v13.jsonl", "w", encoding="utf-8") as f:
        for r in out:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    from collections import Counter
    print(f"train_v13: {len(out)} | {dict(Counter(r['src'] for r in out))}")


if __name__ == "__main__":
    main()
