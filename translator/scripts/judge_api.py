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
Calibration layer (issue #713): Judge.verdict() gives an absolute per-locale
go/no-go from thresholds fitted on native-labeled pairs (calibrate_judge.py).
It is meant for auto-pipeline accept decisions on fresh translations.
Production scans keep the double-confirmation rule (percentile x algorithmic
flag) — phonetic-loanword false negatives survive any threshold. Uncalibrated
locale or mismatched heads -> CalibrationError (fail loud, never silently rank).

Usage:
    from judge_api import Judge
    j = Judge()
    j.answer("帰りたい", {"ok": {"type": "noul",
          "instructions": "Is 'I want to go home' a correct translation into English?",
          }, "candidates": ["I want to go home"]})
"""
from __future__ import annotations

import json
from pathlib import Path

import numpy as np
import requests
import torch
import torch.nn.functional as F

import judge_calibration
from judge_calibration import Calibration, verify_head_hash

EMB_URL = "http://127.0.0.1:8090/v1/embeddings"
HEADS = judge_calibration.heads_path_from_env()
CALIBRATION = judge_calibration.calibration_path_from_env()
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
    def __init__(self, heads_path: str = str(HEADS), emb_url: str = EMB_URL,
                 temperature: float = 0.07, calibration_path: Path = CALIBRATION):
        ck = torch.load(heads_path, map_location="cpu")
        self.hs = Heads(ck["dim"])
        self.hs.load_state_dict(ck["heads"])
        self.hs.eval()
        self.url = emb_url
        self.tau = temperature
        self.heads_path = Path(heads_path)
        self.calibration_path = Path(calibration_path)
        self._calibration: Calibration | None = None
        self.embedding_model: str | None = None  # served model id, captured on first response

    def _embed(self, texts: list[str]) -> torch.Tensor:
        texts = [t[:MAX_LEN] for t in texts]
        out = []
        for s in range(0, len(texts), 64):
            r = requests.post(self.url, json={"model": "e5b", "input": texts[s:s + 64]}, timeout=300)
            r.raise_for_status()
            payload = r.json()
            if self.embedding_model is None:
                self.embedding_model = payload.get("model")
            out += [d["embedding"] for d in payload["data"]]
        return torch.from_numpy(np.asarray(out, dtype=np.float32))

    def _score(self, state_text: str, cand_texts: list[str]) -> np.ndarray:
        st = self._embed([f"query: {state_text}"])
        ct = self._embed([f"passage: {t}" for t in cand_texts])
        with torch.no_grad():
            return self.hs(st.expand(len(cand_texts), -1), ct).numpy()

    def score_pairs(self, pairs: list[tuple[str, str]]) -> list[float]:
        """Batched raw pair scores for [(state, translation), …]."""
        st = self._embed([f"query: {state}" for state, _ in pairs])
        ct = self._embed([f"passage: {t}" for _, t in pairs])
        with torch.no_grad():
            scores = self.hs(st, ct)
        return [float(v) for v in scores]

    def raw_score(self, state: str, translation: str) -> float:
        """Raw pair score (cosine through the heads). Higher = better fit.

        Comparable only within one ranking call — never a cutoff by itself;
        for absolute decisions use verdict().
        """
        return self.score_pairs([(state, translation)])[0]

    def _load_calibration(self) -> Calibration:
        if self._calibration is None:
            calibration = Calibration.load(self.calibration_path)
            verify_head_hash(calibration.doc["heads_sha256"], self.heads_path)
            self._calibration = calibration
        return self._calibration

    def verdict(self, state: str, translation: str, locale: str) -> dict:
        """Absolute per-locale go/no-go from fitted thresholds (issue #713).

        Raises CalibrationError when the calibration file is missing, the
        locale is unfitted, or the heads bytes differ from the fitted ones.
        """
        calibration = self._load_calibration()
        entry = calibration.entry(locale)
        score = self.raw_score(state, translation)
        return {"ok": calibration.decide(locale, score), "score": score,
                "threshold": entry["threshold"], "locale": locale}

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
