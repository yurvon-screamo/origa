#!/usr/bin/env python3
"""Scan the live cdn phrase set with judge v3: score every (x, gold) pair.

Pure assessment, no fixes. Output: score distribution per locale,
bottom-percentile suspects, full scores saved for future analysis.
"""
import json
import time
from collections import defaultdict
from pathlib import Path

import numpy as np
import requests
from concurrent.futures import ThreadPoolExecutor

import torch
import torch.nn.functional as F

CDN = sorted(Path("/home/yurvon/origa/cdn/phrases/data").glob("p*.json"))
OUT = Path("/home/yurvon/origa-translator/data/pipeline")
EMB = "http://127.0.0.1:8090/v1/embeddings"
HEADS = Path("/home/yurvon/origa-translator/model/judge_heads_v3.pt")
LOCALES = ("ru", "en", "vi", "ko")
POOL = 32
BATCH = 64


class Heads(torch.nn.Module):
    def __init__(self, dim):
        super().__init__()
        self.s = torch.nn.Sequential(torch.nn.Linear(dim, 256), torch.nn.GELU(), torch.nn.Linear(256, 256))
        self.a = torch.nn.Sequential(torch.nn.Linear(dim, 256), torch.nn.GELU(), torch.nn.Linear(256, 256))

    def forward(self, sv, av):
        zq = F.normalize(self.s(sv), dim=-1)
        zc = F.normalize(self.a(av), dim=-1)
        return (zq * zc).sum(-1)


def main():
    phrases = []
    for f in CDN:
        phrases += json.load(open(f, encoding="utf-8"))
    print(f"phrases: {len(phrases)}", flush=True)

    ck = torch.load(HEADS, map_location="cpu")
    hs = Heads(ck["dim"])
    hs.load_state_dict(ck["heads"])
    hs.eval()

    # unique texts: states (query:) + translations (passage:)
    states = [p["x"] for p in phrases]
    state_set = set(states)
    trans = []
    for p in phrases:
        for L in LOCALES:
            t = (p.get(L) or "").strip()
            if t and t not in state_set:
                trans.append(t)
    trans = list(dict.fromkeys(trans))
    print(f"unique: {len(state_set)} states + {len(trans)} translations", flush=True)

    t0 = time.perf_counter()
    V = {}

    def fetch(prefixed_chunk):
        # e5 max_len 512 токенов: длинные runaway-тексты усекаем (как truncation в judge_quick)
        payload = [(t[:1000] if len(t) <= 1000 else t[:1000]) for t, _ in prefixed_chunk]
        r = requests.post(EMB, json={"model": "e5b", "input": payload}, timeout=600)
        r.raise_for_status()
        return [(t[len(pfx):], np.asarray(d["embedding"], dtype=np.float32))
                for t, pfx, d in zip(payload, [p for _, p in prefixed_chunk], r.json()["data"])]

    # build jobs
    jobs = [("query: " + s, "query: ") for s in states] + [("passage: " + t, "passage: ") for t in trans]
    chunks = [jobs[i:i + BATCH] for i in range(0, len(jobs), BATCH)]
    done = 0
    with ThreadPoolExecutor(POOL) as ex:
        for res in ex.map(fetch, chunks):
            for text, vec in res:
                V[text] = vec
            done += 1
            if done % 1000 == 0:
                el = time.perf_counter() - t0
                print(f"  {done*BATCH}/{len(jobs)} ({el/60:.1f} мин, {done*BATCH/el:.0f} т/с)", flush=True)
    print(f"embedded {len(V)} in {(time.perf_counter()-t0)/60:.1f} min", flush=True)

    # score
    scores = defaultdict(list)
    rows_out = []
    with torch.no_grad():
        B = 512
        for s in range(0, len(phrases), B):
            block = phrases[s:s + B]
            svs, avs, meta = [], [], []
            for p in block:
                sv = V.get(p["x"])
                if sv is None:
                    continue
                for L in LOCALES:
                    t = (p.get(L) or "").strip()
                    av = V.get(t)
                    if not t or av is None:
                        continue
                    svs.append(sv)
                    avs.append(av)
                    meta.append((p["i"], p["x"], L, t))
            if not svs:
                continue
            sv = torch.from_numpy(np.stack(svs))
            av = torch.from_numpy(np.stack(avs))
            sc = hs(sv, av).numpy()
            for (i, x, L, t), v in zip(meta, sc):
                scores[L].append(float(v))
                rows_out.append({"i": i, "lang": L, "score": round(float(v), 4)})
            if (s // B) % 40 == 0:
                print(f"  scored {len(rows_out)}/{len(phrases)*4}", flush=True)

    with open(OUT / "cdn_phrase_scores_v3.jsonl", "w", encoding="utf-8") as f:
        for r in rows_out:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")

    print(f"\n=== СУДЬЯ v3 × CDN-ФРАЗЫ ({len(rows_out)} пар) ===", flush=True)
    for L in LOCALES:
        v = np.array(scores[L])
        print(f"  [{L}] n={len(v)} | mean {v.mean():.3f} | p50 {np.median(v):.3f} "
              f"| p25 {np.percentile(v,25):.3f} | p5 {np.percentile(v,5):.3f} "
              f"| p1 {np.percentile(v,1):.3f} | min {v.min():.3f}", flush=True)
    allv = np.concatenate([np.array(scores[L]) for L in LOCALES])
    print(f"  [ALL] mean {allv.mean():.3f} | p5 {np.percentile(allv,5):.3f} | p1 {np.percentile(allv,1):.3f}", flush=True)


if __name__ == "__main__":
    main()
