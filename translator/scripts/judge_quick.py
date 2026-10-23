#!/usr/bin/env python3
"""Quick encoder benchmark: train judge heads on a small set (default 3k),
evaluate on the golden pairs. One run per encoder.

Pooling / format handling:
  qwen3-embedding, jina-v5      -> last-token pooling, plain text
  e5-large-instruct             -> mean pooling, "Instruct+Query" state, plain candidate
  granite-embedding (BERT-arch) -> mean pooling, plain text
"""

import argparse
import json
import random
import time
from pathlib import Path

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F
from transformers import AutoModel, AutoTokenizer

DATA = Path("/home/yurvon/origa-translator/data/judge")
EPOCHS = 5
LR = 3e-4
SEED = 5

ENCODERS = {
    "qwen": ("Qwen/Qwen3-Embedding-0.6B", "qwen_ins"),
    "e5": ("intfloat/multilingual-e5-large-instruct", "e5"),
    "jina": ("jinaai/jina-embeddings-v5-text-small-text-matching", "last"),
    "granite": ("ibm-granite/granite-embedding-311m-multilingual-r2", "mean"),
    "granite97": ("ibm-granite/granite-embedding-97m-multilingual-r2", "mean"),
    "e5base": ("intfloat/multilingual-e5-base", "e5base"),
    "e5small": ("intfloat/multilingual-e5-small", "e5base"),
    "gemma": ("google/gemma-4-E2B-it", "last"),
}


class Heads(nn.Module):
    def __init__(self, dim):
        super().__init__()
        self.s = nn.Sequential(nn.Linear(dim, 256), nn.GELU(), nn.Linear(256, 256))
        self.a = nn.Sequential(nn.Linear(dim, 256), nn.GELU(), nn.Linear(256, 256))

    def forward(self, sv, avs):
        zq = F.normalize(self.s(sv), dim=-1)
        zc = F.normalize(self.a(avs.reshape(-1, avs.shape[-1])), dim=-1).view(
            avs.shape[0], avs.shape[1], -1
        )
        return torch.einsum("bd,bkd->bk", zq, zc)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--encoder", required=True, choices=ENCODERS)
    ap.add_argument("--train-n", type=int, default=3000)
    ap.add_argument("--train-file", default="judge_train_v3.jsonl")
    ap.add_argument("--golden-file", default="golden_pairs.jsonl")
    ap.add_argument("--save-heads", action="store_true")
    a = ap.parse_args()
    torch.manual_seed(SEED)
    rng = random.Random(SEED)

    model_id, mode = ENCODERS[a.encoder]
    tok = AutoTokenizer.from_pretrained(model_id)
    if a.encoder == "gemma":
        model = AutoModel.from_pretrained(
            model_id, torch_dtype=torch.bfloat16, device_map={"": "xpu"}
        ).eval()
        for name in ("vision_tower", "audio_tower"):
            for holder in (model, getattr(model, "model", None)):
                tower = getattr(holder, name, None) if holder is not None else None
                if tower is not None:
                    tower.to("cpu")
                    break
    else:
        try:
            model = AutoModel.from_pretrained(
                model_id, dtype=torch.bfloat16, device_map={"": "xpu"}
            ).eval()
        except TypeError:  # transformers regression: dtype kwarg rejected by XLMRoberta
            model = (
                AutoModel.from_pretrained(model_id).to(torch.bfloat16).to("xpu").eval()
            )
    E5Q = (
        "Instruct: Given a Japanese sentence, select its correct translation "
        "among candidates\nQuery: {t}"
    )
    QWQ = (
        "Instruct: Given a Japanese sentence, select its correct translation "
        "among candidates\nQuery:{t}"
    )

    @torch.no_grad()
    def embed(texts, is_state):
        out = []
        for s in range(0, len(texts), 32):
            chunk = []
            for t in texts[s : s + 32]:
                if mode == "e5" and is_state:
                    t = E5Q.format(t=t)
                elif mode == "e5base":
                    t = ("query: " if is_state else "passage: ") + t
                elif mode == "qwen_ins" and is_state:
                    t = QWQ.format(t=t)
                chunk.append(t)
            enc = tok(
                chunk,
                return_tensors="pt",
                padding=True,
                truncation=True,
                max_length=512,
            ).to("xpu")
            h = model(**enc).last_hidden_state
            if mode in ("last", "qwen_ins"):
                idx = enc["attention_mask"].sum(1) - 1
                vecs = h[torch.arange(len(idx)), idx]
            elif mode == "cls":
                vecs = h[:, 0]
            else:  # mean
                m = enc["attention_mask"].unsqueeze(-1).to(h.dtype)
                vecs = (h * m).sum(1) / m.sum(1).clamp(min=1)
            vecs = F.normalize(
                vecs.float(), dim=-1
            )  # L2: raw norms (30-90+) explode heads
            out += [v.cpu().numpy() for v in vecs]
        return out

    train = [json.loads(line) for line in open(DATA / a.train_file, encoding="utf-8")]
    rng.shuffle(train)
    train = train[: a.train_n]
    golden = [json.loads(line) for line in open(DATA / a.golden_file, encoding="utf-8")]
    print(f"[{a.encoder}] train {len(train)} | golden {len(golden)}", flush=True)

    t0 = time.perf_counter()
    states, cand_texts, cand_of = [], [], []
    for r in train:
        cand_of.append(len(cand_texts))
        states.append(r["state"])
        cand_texts += [r["positive"], *r["negatives"]]
    g_states = [r["state"] for r in golden]
    g_good = [r["good"] for r in golden]
    g_bad = [r["bad"] for r in golden]

    SV = {t: v for t, v in zip(states, embed(states, True))}
    CV = {t: v for t, v in zip(cand_texts, embed(cand_texts, False))}
    GS = {t: v for t, v in zip(g_states, embed(g_states, True))}
    GG = {t: v for t, v in zip(g_good, embed(g_good, False))}
    GB = {t: v for t, v in zip(g_bad, embed(g_bad, False))}
    print(f"embedded in {time.perf_counter() - t0:.0f}s", flush=True)
    dim = len(next(iter(CV.values())))

    # tensors
    def pack(rows):
        svs, avs, msk = [], [], []
        T = lambda x: torch.from_numpy(np.asarray(x))  # noqa: E731
        for r in rows:
            cands = [r["positive"]] + r["negatives"]
            svs.append(T(SV[r["state"]]))
            avs.append(torch.stack([T(CV[c]) for c in cands]))
            msk.append(torch.tensor([True] * len(cands)))
        K = max(x.shape[0] for x in avs)
        avs = torch.stack([F.pad(x, (0, 0, 0, K - x.shape[0])) for x in avs])
        msk = torch.stack([F.pad(m, (0, K - m.shape[0]), value=False) for m in msk])
        return torch.stack(svs).float(), avs.float(), msk

    TR = pack(train)
    hs = Heads(dim)
    opt = torch.optim.AdamW(hs.parameters(), lr=LR, weight_decay=0.01)
    for ep in range(1, EPOCHS + 1):
        idx = torch.randperm(len(TR[0]))
        tot = 0.0
        for s in range(0, len(idx), 256):
            b = idx[s : s + 256]
            sv, av, msk = TR[0][b], TR[1][b], TR[2][b]
            sc = hs(sv, av).masked_fill(~msk, -1e9) / 0.07
            loss = F.cross_entropy(sc, sv.new_zeros(len(b)).long())
            opt.zero_grad()
            loss.backward()
            opt.step()
            tot += loss.item() * len(b)
        print(f"  epoch {ep}: loss={tot / len(TR[0]):.4f}", flush=True)

    # golden eval
    hs.eval()
    ok_by_type = {}
    with torch.no_grad():
        for r in golden:
            sv = torch.from_numpy(GS[r["state"]]).unsqueeze(0)
            av = torch.stack(
                [torch.from_numpy(GG[r["good"]]), torch.from_numpy(GB[r["bad"]])]
            ).unsqueeze(0)
            sc = hs(sv, av)[0]
            good = sc[0].item() > sc[1].item()
            ok_by_type.setdefault(r["type"], []).append(good)
    total = [v for vs in ok_by_type.values() for v in vs]
    print(f"\n=== [{a.encoder}] GOLDEN: {sum(total)}/{len(total)} ===", flush=True)
    for t, vs in sorted(ok_by_type.items()):
        print(f"  {t:<18} {sum(vs)}/{len(vs)}")
    hard = [
        v
        for t, vs in ok_by_type.items()
        if t not in ("cross_locale", "jp_echo", "truncate", "blind_soft")
        for v in vs
    ]
    print(f"  HARD-subset: {sum(hard)}/{len(hard)}", flush=True)
    if a.save_heads:
        torch.save(
            {
                "heads": hs.state_dict(),
                "dim": dim,
                "encoder": model_id,
                "pooling": mode,
            },
            f"judge_heads_{a.encoder}_full.pt",
        )
        print(f"saved judge_heads_{a.encoder}_full.pt", flush=True)


if __name__ == "__main__":
    main()
