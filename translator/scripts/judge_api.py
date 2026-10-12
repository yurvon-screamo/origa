#!/usr/bin/env python3
"""Jev-like typed-question API on top of judge v3 CLM heads.

Wire format (System One style):
    state: str                                # e.g. the JP phrase
    questions: {qid: {"type": "noul"|"choice"|"score", ...}}

Types:
    noul:   {"instructions": str}
            -> {"noul": P(yes)}               # softmax over yes/no descriptors
    choice: {"instructions": str, "criteria": {key: description}}
            -> {key: P(key)}                   # softmax over option texts
    score:  {"instructions": str, "criteria": [level, ...]}
            -> {"expected": float, "levels": [P per level]}

Backed by: e5b embeddings server (:8090, mean pooling) + judge heads.
NOTE: probabilities are RANKED-CALIBRATED only (relative comparison);
absolute thresholds require the future calibration layer (per-locale).

Usage:
    from judge_api import Judge
    j = Judge()
    j.answer("帰りたい", {"ok": {"type": "noul",
          "instructions": "Is 'I want to go home' a correct translation into English?",
          }, "candidates": ["I want to go home"]})
"""
from __future__ import annotations

import json
import os
from pathlib import Path

import numpy as np
import requests
import torch
import torch.nn.functional as F

EMB_URL = "http://127.0.0.1:8090/v1/embeddings"
HEADS = Path(os.environ.get("ORIGA_JUDGE_HEADS", "model/origa-judge.pt"))
MAX_LEN = 1000  # e5 window guard (512 tokens ~ safe for our locales)


class Heads(torch.nn.Module):
    def __init__(self, dim):
        super().__init__()
        self.s = torch.nn.Sequential(torch.nn.Linear(dim, 256), torch.nn.GELU(), torch.nn.Linear(256, 256))
        self.a = torch.nn.Sequential(torch.nn.Linear(dim, 256), torch.nn.GELU(), torch.nn.Linear(256, 256))

    def forward(self, sv, av):
        zq = F.normalize(self.s(sv), dim=-1)
        zc = F.normalize(self.a(av), dim=-1)
        return (zq * zc).sum(-1)


class Judge:
    def __init__(self, heads_path: str = str(HEADS), emb_url: str = EMB_URL, temperature: float = 0.07):
        ck = torch.load(heads_path, map_location="cpu")
        self.hs = Heads(ck["dim"])
        self.hs.load_state_dict(ck["heads"])
        self.hs.eval()
        self.url = emb_url
        self.tau = temperature

    def _embed(self, texts: list[str]) -> torch.Tensor:
        texts = [t[:MAX_LEN] for t in texts]
        out = []
        for s in range(0, len(texts), 64):
            r = requests.post(self.url, json={"model": "e5b", "input": texts[s:s + 64]}, timeout=300)
            r.raise_for_status()
            out += [d["embedding"] for d in r.json()["data"]]
        return torch.from_numpy(np.asarray(out, dtype=np.float32))

    def _score(self, state_text: str, cand_texts: list[str]) -> np.ndarray:
        st = self._embed([f"query: {state_text}"])
        ct = self._embed([f"passage: {t}" for t in cand_texts])
        with torch.no_grad():
            return self.hs(st.expand(len(cand_texts), -1), ct).numpy()

    def answer(self, state: str, questions: dict) -> dict:
        """One state, many typed questions. CLM layout: context first, question last."""
        answers = {}
        # cache: one state_text per question (instruction differs), candidates per question
        for qid, q in questions.items():
            t = q["type"]
            ins = q.get("instructions", "")
            state_text = f"{state}\n\n{ins}" if ins else state
            if t == "noul":
                cands = [f"Yes. This is true: {ins}", f"No. This is false: {ins}"] if ins else ["true", "false"]
                sc = self._score(state_text, cands)
                p = torch.softmax(torch.from_numpy(sc) / self.tau, dim=0).numpy()
                answers[qid] = {"noul": float(p[0])}
            elif t == "choice":
                crit = q["criteria"]
                keys = list(crit)
                texts = [crit[k] if crit[k] not in (None, "") else k for k in keys]
                sc = self._score(state_text, texts)
                p = torch.softmax(torch.from_numpy(sc) / self.tau, dim=0).numpy()
                answers[qid] = {k: float(v) for k, v in zip(keys, p)}
            elif t == "score":
                crit = q["criteria"]
                sc = self._score(state_text, [str(c) for c in crit])
                p = torch.softmax(torch.from_numpy(sc) / self.tau, dim=0).numpy()
                answers[qid] = {"expected": float(sum(i * v for i, v in enumerate(p))),
                                "levels": [float(v) for v in p]}
            else:
                raise ValueError(f"unknown question type {t!r}")
        return {"answers": answers}


if __name__ == "__main__":
    j = Judge()
    # demo 1: noul — перевод годен?
    r = j.answer("帰りたいなぁ", {
        "ok": {"type": "noul",
               "instructions": "Is 'I want to go home...' a correct and natural English translation of the sentence?"}
    })
    print("noul:", json.dumps(r, ensure_ascii=False))
    # demo 2: choice — какой из переводов лучший?
    r2 = j.answer("その言葉、心に沁みたよ", {
        "pick": {"type": "choice",
                 "instructions": "Which translation is the most correct and natural in Russian?",
                 "criteria": {
                     "good": "Эти слова запали мне в душу.",
                     "echo": "その言葉、心に沁みたよ",
                     "wrong_lang": "These words touched my heart.",
                 }}
    })
    print("choice:", json.dumps(r2, ensure_ascii=False))
