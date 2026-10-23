#!/usr/bin/env python3
"""Eval-only: score saved judge heads on a golden file (no retraining).

Usage: eval_only_heads.py --heads model/judge_heads_v11.pt --golden golden_v4c.jsonl
Embeddings via running e5b vLLM server (:8090) with query:/passage: prefixes.
"""

import argparse
import json
from collections import defaultdict
from pathlib import Path

import numpy as np
import requests
import torch
import torch.nn.functional as F

DATA = Path("/home/yurvon/origa-translator/data/judge")
EMB = "http://127.0.0.1:8090/v1/embeddings"


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


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--heads", required=True)
    ap.add_argument("--golden", default="golden_v4c.jsonl")
    a = ap.parse_args()

    ck = torch.load(a.heads, map_location="cpu")
    hs = Heads(ck["dim"])
    hs.load_state_dict(ck["heads"])
    hs.eval()
    rows = [json.loads(line) for line in open(DATA / a.golden, encoding="utf-8")]

    texts = []
    for r in rows:
        texts += [r["state"], r["good"], r["bad"]]
    uniq = list(dict.fromkeys(texts))
    V = {}
    # роли: state -> query:, good/bad -> passage:
    # ВАЖНО (ревью): при коллизии текста (bad == state, jp_echo-кейсы golden)
    # кандидат обязан эмбеддиться как passage — считаем отдельные эмбеддинги ролей.
    states = {r["state"] for r in rows}
    inp = [
        (f"query: {t}", "q") if t in states else (f"passage: {t}", "p") for t in uniq
    ]
    cand_texts = [r["good"] for r in rows] + [r["bad"] for r in rows]
    for t in dict.fromkeys(cand_texts):
        inp.append((f"passage: {t}", "p2"))  # passage-вид даже если текст = state
    inp = [x[0] for x in inp]
    emb_in = list(dict.fromkeys(inp))
    E = {}
    for s in range(0, len(emb_in), 64):
        r = requests.post(
            EMB, json={"model": "e5b", "input": emb_in[s : s + 64]}, timeout=300
        )
        r.raise_for_status()
        for t, item in zip(emb_in[s : s + 64], r.json()["data"]):
            E[t] = np.asarray(item["embedding"], dtype=np.float32)
    for r in rows:
        V[r["state"] + "\x00q"] = E[f"query: {r['state']}"]
        V[r["good"] + "\x00p"] = E[f"passage: {r['good']}"]
        V[r["bad"] + "\x00p"] = E[f"passage: {r['bad']}"]

    by_type = defaultdict(list)
    with torch.no_grad():
        for r in rows:
            sv = torch.from_numpy(V[r["state"] + "\x00q"]).unsqueeze(0)
            av = torch.stack(
                [
                    torch.from_numpy(V[r["good"] + "\x00p"]),
                    torch.from_numpy(V[r["bad"] + "\x00p"]),
                ]
            ).unsqueeze(0)
            sc = hs(sv, av)[0]
            by_type[r["type"]].append(sc[0].item() > sc[1].item())

    total = [v for vs in by_type.values() for v in vs]
    print(
        f"\n=== EVAL {Path(a.heads).name} on {a.golden}: {sum(total)}/{len(total)} "
        f"({100 * sum(total) / len(total):.1f}%) ==="
    )
    for t, vs in sorted(by_type.items()):
        print(f"  {t:<28} {sum(vs)}/{len(vs)}")


if __name__ == "__main__":
    main()
