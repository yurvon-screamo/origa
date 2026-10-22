#!/usr/bin/env python3
"""Post-process the zh corpus: purge the known garbage era (lines 1024-1215
of the a5c18bc run: bit-graph era before --enforce-eager fix) and regenerate
those phrases cleanly; re-verify zh_ok with detect_zh v2.

Run ONLY after the bulk completes (no concurrent writers).
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from translate_zh_bulk import CDN, detect_zh, gen_one  # noqa: E402

OUT = Path("/home/yurvon/origa-translator/data/pipeline/zh")
GARBAGE_LO, GARBAGE_HI = 1024, 1216  # 0-based [lo, hi) — bit-graph era


def main():
    rows = [json.loads(l) for l in open(OUT / "zh_translations.jsonl", encoding="utf-8")]
    print(f"загружено {len(rows)}; мусорная эпоха [{GARBAGE_LO}:{GARBAGE_HI})")

    # порядок = порядок cdn (bulk шёл по списку) — сверяем id
    phrases = []
    for f in CDN:
        phrases += json.load(open(f, encoding="utf-8"))
    bad_zone = rows[GARBAGE_LO:GARBAGE_HI]
    for r, p in zip(bad_zone, phrases[GARBAGE_LO:GARBAGE_HI]):
        assert r["i"] == p["i"], f"рассинхрон зоны на {r['i']} vs {p['i']}"

    # перегенерация эпохи
    fixed = 0
    for k in range(GARBAGE_LO, GARBAGE_HI):
        p = phrases[k]
        zh, retr = gen_one(p["x"])
        rows[k] = {"i": p["i"], "x": p["x"], "zh": zh, "retries": retr,
                   "zh_ok": detect_zh(zh, p["x"]), "era": "regen"}
        fixed += 1

    # перемаркировка zh_ok v2 по всему корпусу (без перегенерации)
    remark = sum(1 for r in rows if r["zh_ok"] and not detect_zh(r["zh"], r["x"]))
    for r in rows:
        r["zh_ok"] = detect_zh(r["zh"], r["x"])

    with open(OUT / "zh_translations.jsonl", "w", encoding="utf-8") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    json.dump({"done": len(rows)}, open(OUT / "progress.json", "w"))
    fails = sum(1 for r in rows if not r["zh_ok"])
    print(f"перегенерировано {fixed}; перемаркировано v2-детектором {remark}; "
          f"итоговых zh_ok=False: {fails}/{len(rows)} ({100*fails/len(rows):.2f}%)")


if __name__ == "__main__":
    main()
