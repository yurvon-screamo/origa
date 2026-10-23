#!/usr/bin/env python3
"""Scan the cdn DICTIONARY with judge v3: score every (word, t-variants).

Best-of-t scoring (word is polysemous: any valid translation counts).
Output: per-locale score distributions + dual-confirmation suspects
(percentile-in-locale x reliable algo flags: paren-gigo, semicolon lists,
jp-in-translation, duplicates, runaway). Assessment only — no fixes.
"""

import json
import re
import time
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import numpy as np
import requests

import torch
import torch.nn.functional as F

CDN = Path("/home/yurvon/origa/cdn/dictionary")
OUT = Path("/home/yurvon/origa-translator/data/pipeline")
EMB = "http://127.0.0.1:8090/v1/embeddings"
HEADS = Path("/home/yurvon/origa-translator/model/judge_heads_v3.pt")
LANGS = ("ru", "en", "vi", "ko")
POOL, BATCH, MAXLEN = 32, 64, 1000


class Heads(torch.nn.Module):
    def __init__(self, dim):
        super().__init__()
        self.s = torch.nn.Sequential(
            torch.nn.Linear(dim, 256), torch.nn.GELU(), torch.nn.Linear(256, 256)
        )
        self.a = torch.nn.Sequential(
            torch.nn.Linear(dim, 256), torch.nn.GELU(), torch.nn.Linear(256, 256)
        )

    def forward(self, sv, avs):
        zq = F.normalize(self.s(sv), dim=-1)
        zc = F.normalize(self.a(avs.reshape(-1, avs.shape[-1])), dim=-1).view(
            avs.shape[0], avs.shape[1], -1
        )
        return torch.einsum("bd,bkd->bk", zq, zc)


RE_JP = re.compile(r"[ひ-んア-ン一-龥ぁ-ゖァ-ヺー]")
RE_PAREN = re.compile(r"[({\[].{25,}[)}\]]")
RE_SEMI = re.compile(r".+;.+;.+")
RE_SHORT_PAREN = re.compile(r"[({\[][^)）\]}]{12,}[)}\]]")


def algo_flags(tlist):
    """Reliable dictionary-content flags on the whole t-list of one (word, lang)."""
    f = []
    joined = " | ".join(tlist)
    if any(RE_JP.search(t) for t in tlist):
        f.append("jp_in_t")
    if any(RE_PAREN.search(t) for t in tlist):
        f.append("paren_gigo")
    if any(RE_SEMI.search(t) for t in tlist):
        f.append("semicolon_list")
    if len(set(x.strip() for x in tlist)) < len(tlist):
        f.append("dup_t")
    if len(joined) > 400:
        f.append("runaway")
    return f


def main():
    words = {}
    for cf in sorted(CDN.glob("chunk_*.json")):
        words.update(json.load(open(cf, encoding="utf-8")))
    print(f"dictionary: {len(words)} words", flush=True)

    ck = torch.load(HEADS, map_location="cpu")
    hs = Heads(ck["dim"])
    hs.load_state_dict(ck["heads"])
    hs.eval()

    # collect pairs
    meta = []  # (word, lang, tlist)
    for w, entry in words.items():
        for L in LANGS:
            tl = (entry.get(L) or {}).get("t") or []
            tl = [t.strip() for t in tl if t and t.strip()]
            if tl:
                meta.append((w, L, tl))
    print(f"pairs: {len(meta)}", flush=True)

    # unique texts
    uniq_states = {f"Japanese word: {w}" for w, _, _ in meta}
    uniq_cands = {t for _, _, tl in meta for t in tl}
    print(f"unique: {len(uniq_states)} states + {len(uniq_cands)} cands", flush=True)

    t0 = time.perf_counter()
    cache_p = OUT / "dict_emb_cache_v3.npz"
    V = None
    if cache_p.exists():
        z = np.load(cache_p, allow_pickle=True)
        V = {k: v.astype(np.float32) for k, v in zip(z["keys"].tolist(), z["vecs"])}
        print(f"cache loaded: {len(V)} vectors", flush=True)
    if V is None:
        state_keys = list(uniq_states)
        cand_keys = list(uniq_cands)
        jobs = [("query: " + s, s) for s in state_keys] + [
            ("passage: " + t, t) for t in cand_keys
        ]

        def fetch(chunk):
            payload = [p[:MAXLEN] for p, _ in chunk]
            r = requests.post(EMB, json={"model": "e5b", "input": payload}, timeout=600)
            r.raise_for_status()
            return [
                (raw, np.asarray(d["embedding"], dtype=np.float32))
                for (_, raw), d in zip(chunk, r.json()["data"])
            ]

        chunks = [jobs[i : i + BATCH] for i in range(0, len(jobs), BATCH)]
        V = {}
        with ThreadPoolExecutor(POOL) as ex:
            for res in ex.map(fetch, chunks):
                for k, v in res:
                    V[k] = v
        print(
            f"embedded {len(V)} in {(time.perf_counter() - t0) / 60:.1f} min",
            flush=True,
        )
        np.savez(
            cache_p,
            keys=np.array(list(V.keys())),
            vecs=np.stack(list(V.values())).astype(np.float16),
        )
        print(f"cache saved: {cache_p.name}", flush=True)

    # score: best-of-t
    scores = defaultdict(list)
    rows = []
    with torch.no_grad():
        B = 512
        for s in range(0, len(meta), B):
            block = meta[s : s + B]
            svs, avs, msk = [], [], []
            for w, L, tl in block:
                svs.append(V[f"Japanese word: {w}"])
                avs.append(np.stack([V[t] for t in tl]))
                msk.append([True] * len(tl))
            K = max(a.shape[0] for a in avs)
            sv = torch.from_numpy(np.stack(svs))
            av = torch.stack(
                [F.pad(torch.from_numpy(a), (0, 0, 0, K - a.shape[0])) for a in avs]
            )
            m = torch.stack(
                [F.pad(torch.tensor(x), (0, K - len(x)), value=False) for x in msk]
            )
            sc = hs(sv, av).masked_fill(~m, -1e9)
            best = sc.max(1).values.numpy()
            for (w, L, tl), b in zip(block, best):
                scores[L].append(float(b))
                rows.append({"word": w, "lang": L, "score": round(float(b), 4)})
            if (s // B) % 30 == 0:
                print(f"  scored {len(rows)}/{len(meta)}", flush=True)

    with open(OUT / "dict_scores_v3.jsonl", "w", encoding="utf-8") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")

    print(f"\n=== СУДЬЯ v3 × СЛОВАРЬ ({len(rows)} пар) ===", flush=True)
    pct = {}
    for L in LANGS:
        v = np.array(sorted(scores[L]))
        pct[L] = v
        print(
            f"  [{L}] n={len(v)} | mean {v.mean():.3f} | p50 {np.median(v):.3f} "
            f"| p5 {np.percentile(v, 5):.3f} | p1 {np.percentile(v, 1):.3f} | min {v.min():.3f}",
            flush=True,
        )

    # dual confirmation
    def prank(L, v):
        return 100.0 * np.searchsorted(pct[L], v) / len(pct[L])

    word_lang_flags = {}
    suspects = []
    for r in rows:
        w, L = r["word"], r["lang"]
        key = (w, L)
        if key not in word_lang_flags:
            entry = words.get(w, {}).get(L) or {}
            tl = [t.strip() for t in (entry.get("t") or []) if t and t.strip()]
            word_lang_flags[key] = algo_flags(tl)
        flags = word_lang_flags[key]
        rank = prank(L, r["score"])
        if rank < 1.5 or flags:
            suspects.append(
                {
                    "word": w,
                    "lang": L,
                    "rank": round(float(rank), 2),
                    "score": r["score"],
                    "flags": flags or ["low_pct"],
                }
            )

    both = [x for x in suspects if x["rank"] < 1.5 and x["flags"] != ["low_pct"]]
    print(
        f"\nподозрений (p<1.5 ИЛИ флаг): {len(suspects)} ({100 * len(suspects) / len(rows):.1f}%)"
    )
    print(f"ДВОЙНОЕ подтверждение: {len(both)}")
    print("по флагам:", dict(Counter(f for x in suspects for f in x["flags"])))
    print("по локалям (двойные):", dict(Counter(x["lang"] for x in both)))
    json.dump(
        suspects,
        open(OUT / "dict_suspects_v3.json", "w", encoding="utf-8"),
        ensure_ascii=False,
        indent=1,
    )

    print("\nглаза: 6 двойных:")
    seen = set()
    n = 0
    for x in both:
        k = (x["word"], x["lang"])
        if k in seen:
            continue
        seen.add(k)
        if n >= 6:
            break
        entry = words.get(x["word"], {}).get(x["lang"]) or {}
        print(f"  [{x['lang']}] rank={x['rank']} {x['flags']}")
        print(f"    word: {x['word']}")
        print(f"    t: {str(entry.get('t'))[:80]}")
        n += 1


if __name__ == "__main__":
    main()
