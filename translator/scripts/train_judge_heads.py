#!/usr/bin/env python3
"""Train CLM-style judge heads (v2): proper InfoNCE + mini-batches.

Embeddings: vLLM pooling server (TRITON_ATTN recipe) with an on-disk cache.
Loss: softmax over [positive, negatives...] scores -> CE(label=positive).
Blind test: manual-review cases at the end.
"""

import json
import os
import random
import time
from pathlib import Path

import numpy as np
import requests
import torch
import torch.nn as nn
import torch.nn.functional as F

DATA = Path(os.environ.get("ORIGA_JUDGE_DATA", "data/judge"))
EMB_URL = "http://127.0.0.1:8090/v1/embeddings"
MODEL = "qwen3-emb"
CACHE_NPZ = Path(os.environ.get("ORIGA_JUDGE_DATA", "data/judge")) / "emb_cache.npz"
EPOCHS = 6
LR = 3e-4
SEED = 5
torch.manual_seed(SEED)
random.seed(SEED)
_sess = requests.Session()


def embed(texts):
    if CACHE_NPZ.exists():
        z = np.load(CACHE_NPZ)
        cache = dict(zip(z["keys"].tolist(), z["vecs"]))
    else:
        cache = {}
    missing = [t for t in dict.fromkeys(texts) if t not in cache]
    for s in range(0, len(missing), 64):
        batch = missing[s : s + 64]
        r = _sess.post(EMB_URL, json={"model": MODEL, "input": batch}, timeout=300)
        r.raise_for_status()
        for t, item in zip(batch, r.json()["data"]):
            cache[t] = np.asarray(item["embedding"], dtype=np.float32)
    np.savez(CACHE_NPZ, keys=np.array(list(cache)), vecs=np.stack(list(cache.values())))
    return {t: torch.from_numpy(cache[t]) for t in texts}


class Heads(nn.Module):
    def __init__(self, dim=1024, width=256):
        super().__init__()
        self.s = nn.Sequential(nn.Linear(dim, width), nn.GELU(), nn.Linear(width, 256))
        self.a = nn.Sequential(nn.Linear(dim, width), nn.GELU(), nn.Linear(width, 256))

    def forward(self, sv, avs):
        # sv [B,d], avs [B,K,d] -> scores [B,K]
        zq = F.normalize(self.s(sv), dim=-1)
        zc = F.normalize(self.a(avs.reshape(-1, avs.shape[-1])), dim=-1)
        zc = zc.view(avs.shape[0], avs.shape[1], -1)
        return torch.einsum("bd,bkd->bk", zq, zc)


def main():
    rng = random.Random(SEED)
    train = [
        json.loads(l) for l in open(DATA / "judge_train_v3.jsonl", encoding="utf-8")
    ]
    val = [json.loads(l) for l in open(DATA / "judge_val.jsonl", encoding="utf-8")]
    rng.shuffle(train)
    val = rng.sample(val, 800)
    print(f"train {len(train)} | val {len(val)}", flush=True)

    t0 = time.perf_counter()
    texts = []
    for r in train + val:
        texts += [r["state"], r["positive"], *r["negatives"]]
    V = embed(texts)
    print(f"embedded {len(V)} texts in {time.perf_counter() - t0:.0f}s", flush=True)

    def tensors(rows):
        svs, avs, masks, labels = [], [], [], []
        for r in rows:
            cands = [r["positive"]] + r["negatives"]
            K = len(cands)
            svs.append(V[r["state"]])
            avs.append(torch.stack([V[c] for c in cands]))
            masks.append(torch.tensor([True] * K))
            labels.append(0)
        Kmax = max(a.shape[0] for a in avs)
        avs = torch.stack([F.pad(a, (0, 0, 0, Kmax - a.shape[0])) for a in avs])
        masks = torch.stack(
            [F.pad(m, (0, Kmax - m.shape[0]), value=False) for m in masks]
        )
        return (torch.stack(svs).float(), avs.float(), masks)

    TR = tensors(train)
    VA = tensors(val)
    print(
        f"train tensors: states {tuple(TR[0].shape)}, cands {tuple(TR[1].shape)}",
        flush=True,
    )

    hs = Heads()
    opt = torch.optim.AdamW(hs.parameters(), lr=LR, weight_decay=0.01)

    @torch.no_grad()
    def val_acc():
        hs.eval()
        sc = hs(VA[0], VA[1]).masked_fill(~VA[2], -1e9)
        acc = (sc.argmax(1) == 0).float().mean().item()
        hs.train()
        return acc

    print(f"val before: gold-top1 {val_acc():.3f}", flush=True)
    B = 256
    for ep in range(1, EPOCHS + 1):
        idx = torch.randperm(len(TR[0]))
        tot = 0.0
        for s in range(0, len(idx), B):
            b = idx[s : s + B]
            sv, av, msk = TR[0][b], TR[1][b], TR[2][b]
            lb = sv.new_zeros(len(b)).long()
            sc = hs(sv, av).masked_fill(~msk, -1e9) / 0.07
            loss = F.cross_entropy(sc, lb)
            opt.zero_grad()
            loss.backward()
            opt.step()
            tot += loss.item() * len(b)
        print(
            f"epoch {ep}/{EPOCHS}: loss={tot / len(TR[0]):.4f} "
            f"val gold-top1={val_acc():.3f}",
            flush=True,
        )

    torch.save({"heads": hs.state_dict(), "dim": 1024, "width": 256}, "judge_heads.pt")
    print("saved judge_heads.pt", flush=True)

    test50 = [
        json.loads(l)
        for l in open(
            os.environ.get("ORIGA_JUDGE_DATA", "data/judge") + "/../test_50x4.jsonl",
            encoding="utf-8",
        )
    ]
    blind = [
        ("en", "Come on, Mom", "Come on, Mom's not a kid anymore!"),
        (
            "ru",
            "Ого, вот это да! Эта штука, похожая на веревку",
            'Вау, это потрясающе! Эта "нава" (вероятно, имеется в виду какой-то предмет)',
        ),
        ("vi", "Vâng. Xin làm ơn mạnh tay một chút.", "Vâng. Xin hãy dùng Griggir."),
        ("ru", "Нет, я разогрею ваш обед.", "Нет, я просто разогрею свой ланч-бокс."),
        (
            "ko",
            "헤헤~. 나중에 불법으로 팔아넘길 생각이야~",
            "헤헤~. 나중에 다른 사람들에게도 공유할 생각이야~",
        ),
        ("ru", "Добро пожаловать домой.", "С возвращением."),
    ]
    print("\n=== BLIND TEST ===", flush=True)
    need = set()
    for lang, gold, cand in blind:
        row = next(
            (
                t
                for t in test50
                if t["lang"] == lang and t["gold"].startswith(gold[:18])
            ),
            None,
        )
        if row:
            need |= {row["x"], row["gold"], cand}
    V2 = embed(list(need))
    hs.eval()
    ok = 0
    with torch.no_grad():
        for lang, gold, cand in blind:
            row = next(
                (
                    t
                    for t in test50
                    if t["lang"] == lang and t["gold"].startswith(gold[:18])
                ),
                None,
            )
            if row is None:
                print(f"[SKIP] {lang}")
                continue
            sv = V2[row["x"]].unsqueeze(0)
            sc = hs(sv, torch.stack([V2[row["gold"]], V2[cand]]).unsqueeze(0))[0]
            good = sc[0].item() > sc[1].item()
            ok += good
            print(
                f"[{'OK ' if good else 'FAIL'}] {lang} p_gold={sc[0]:.3f} p_cand={sc[1]:.3f} | {cand[:60]!r}"
            )
    print(f"blind: {ok}/{len(blind)} gold preferred", flush=True)


if __name__ == "__main__":
    main()
