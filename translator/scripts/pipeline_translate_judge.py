#!/usr/bin/env python3
"""Full pipeline: translator (vLLM tr600k) -> algo checks -> judge (e5-base heads).

Per phrase x locale:
  1. greedy generation (EN instruction), language routing check, JA-retry
  2. judge score via e5b embeddings server (query:/passage: prefixes!) + heads
  3. margin = score(gen) - score(cdn_gold)  -> calibrated quality signal
Errors (margin < THRESH or lang-fail) saved as ready pairs for judge v3 training.

Checkpointed by block; auto-resume. Status line every block.
"""
from __future__ import annotations

import json
import os
import re
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import numpy as np
import requests
import torch
import torch.nn.functional as F

CDN = sorted(Path(os.environ.get("ORIGA_CDN_PHRASES", "../../cdn/phrases/data")).glob("p*.json"))
OUT = Path(os.environ.get("ORIGA_PIPELINE_OUT", "data/pipeline"))
OUT.mkdir(parents=True, exist_ok=True)
TR_URL = "http://127.0.0.1:8091/v1/chat/completions"
EMB_URL = "http://127.0.0.1:8090/v1/embeddings"
HEADS = Path(os.environ.get("ORIGA_JUDGE_HEADS", "model/judge_heads_v2.pt"))
LOCALES = ("ru", "en", "vi", "ko")
BLOCK = 64            # phrases per block
CONC = 15             # generation parallelism
ERR_MARGIN = -0.05    # gen clearly worse than gold -> error pair for v3
VALIDATION_N = 1000   # report snapshot after this many phrases

SYS_EN = "You are a skilled Japanese translator. Translate the sentence accurately and naturally."
SYS_JA = "あなたは熟練した日本語翻訳者です。文を正確かつ自然に翻訳してください。"
INSTR_EN = "Translate the following Japanese sentence into {lname}:\n{x}"
INSTR_JA = {"en": "次の日本語の文を英語に翻訳してください:\n{x}",
            "vi": "次の日本語の文をベトナム語に翻訳してください:\n{x}",
            "ru": "次の日本語の文をロシア語に翻訳してください:\n{x}",
            "ko": "次の日本語の文を韓国語に翻訳してください:\n{x}"}
LNAME = {"en": "English", "vi": "Vietnamese", "ru": "Russian", "ko": "Korean"}

RE_HANGUL = re.compile(r"[가-힣]")
RE_CYR = re.compile(r"[А-ЯЁа-яё]")
RE_KANA = re.compile(r"[ひ-んア-ンー]")
RE_CJK = re.compile(r"[一-龥]")
RE_LATIN = re.compile(r"[A-Za-z]")
RE_VI_MARK = re.compile(r"[ăâêôơưđĂÂÊÔƠƯĐ]|[áàảãạấầẩẫậắằẳẵặéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ]")


def detect_language(text: str) -> str:
    t = (text or "").strip()
    if not t:
        return "empty"
    n = max(len(t), 1)
    c = {"ko": len(RE_HANGUL.findall(t)), "ru": len(RE_CYR.findall(t)),
         "ja": len(RE_KANA.findall(t)) + len(RE_CJK.findall(t)),
         "vi": len(RE_VI_MARK.findall(t)), "en": len(RE_LATIN.findall(t))}
    for k in ("ko", "ru", "ja", "vi"):
        if c[k] / n >= 0.25:
            return k
    if c["vi"] >= 2:
        return "vi"
    return "en" if c["en"] / n >= 0.5 else "other"


class Heads(torch.nn.Module):
    def __init__(self, dim):
        super().__init__()
        self.s = torch.nn.Sequential(torch.nn.Linear(dim, 256), torch.nn.GELU(), torch.nn.Linear(256, 256))
        self.a = torch.nn.Sequential(torch.nn.Linear(dim, 256), torch.nn.GELU(), torch.nn.Linear(256, 256))

    def forward(self, sv, avs):
        zq = F.normalize(self.s(sv), dim=-1)
        zc = F.normalize(self.a(avs.reshape(-1, avs.shape[-1])), dim=-1).view(avs.shape[0], avs.shape[1], -1)
        return torch.einsum("bd,bkd->bk", zq, zc)


def gen_one(x: str, lang: str):
    """Generate + language check + JA-retry. Returns (final_text, attempts, lang_ok)."""
    msgs = [{"role": "system", "content": SYS_EN},
            {"role": "user", "content": INSTR_EN.format(lname=LNAME[lang], x=x)}]
    attempts = []

    def call(messages):
        r = requests.post(TR_URL, json={"model": "tr600k", "messages": messages,
                                        "temperature": 0.0, "max_tokens": 200}, timeout=180)
        r.raise_for_status()
        return r.json()["choices"][0]["message"]["content"].strip()

    out = call(msgs)
    attempts.append({"sys": "en", "gen": out, "det": detect_language(out)})
    if attempts[-1]["det"] != lang:
        msgs_ja = [{"role": "system", "content": SYS_JA},
                   {"role": "user", "content": INSTR_JA[lang].format(x=x)}]
        out2 = call(msgs_ja)
        attempts.append({"sys": "ja", "gen": out2, "det": detect_language(out2)})
        if attempts[-1]["det"] == lang or len(attempts[-1]["gen"]) > len(out):
            out = out2
    lang_ok = attempts[-1]["det"] == lang
    return out, attempts, lang_ok


def embed(texts: list[str]) -> dict[str, np.ndarray]:
    out = {}
    for s in range(0, len(texts), 64):
        chunk = texts[s:s + 64]
        r = requests.post(EMB_URL, json={"model": "e5b", "input": chunk}, timeout=300)
        r.raise_for_status()
        for t, item in zip(chunk, r.json()["data"]):
            out[t] = np.asarray(item["embedding"], dtype=np.float32)
    return out


def main():
    rows_all = []
    for f in CDN:
        rows_all += json.load(open(f, encoding="utf-8"))
    print(f"phrases total: {len(rows_all)}", flush=True)

    ck = torch.load(HEADS, map_location="cpu")
    hs = Heads(ck["dim"])
    hs.load_state_dict(ck["heads"])
    hs.eval()

    run_f = OUT / "full_run.jsonl"
    err_f = OUT / "errors_for_judge_v3.jsonl"
    prog_f = OUT / "progress.json"
    done = 0
    if prog_f.exists():
        done = json.load(open(prog_f))["done"]
        print(f"resume from phrase {done}", flush=True)
    run_fh = open(run_f, "a", encoding="utf-8")
    err_fh = open(err_f, "a", encoding="utf-8")

    t0 = time.perf_counter()
    n_gen = n_retry = n_langfail = n_err = n_margin_neg = 0
    reported = done >= VALIDATION_N  # snapshot already printed earlier?

    for start in range(done, len(rows_all), BLOCK):
        block = rows_all[start:start + BLOCK]
        # 1) generation (parallel over phrase x locale)
        jobs = [(r, L) for r in block for L in LOCALES]
        with ThreadPoolExecutor(CONC) as ex:
            gens = list(ex.map(lambda j: gen_one(j[0]["x"], j[1]), jobs))

        # 2) judge: unique texts
        uniq = set()
        for r in block:
            uniq.add(r["x"])
            for L in LOCALES:
                uniq.add(r[L])
        for (out_t, _a, _ok) in gens:
            uniq.add(out_t)
        V = embed(sorted(uniq))

        # 3) score + rows
        lines, errs = [], []
        with torch.no_grad():
            for i, r in enumerate(block):
                sv = torch.from_numpy(V[r["x"]]).unsqueeze(0)
                for L in LOCALES:
                    out_t, attempts, lang_ok = gens[i * 4 + LOCALES.index(L)]
                    cands = [out_t, r[L]]
                    av = torch.stack([torch.from_numpy(V[c]) for c in cands]).unsqueeze(0)
                    sc = hs(sv, av)[0]
                    s_gen, s_gold = sc[0].item(), sc[1].item()
                    margin = s_gen - s_gold
                    n_gen += 1
                    n_retry += len(attempts) - 1
                    if not lang_ok:
                        n_langfail += 1
                    if margin < 0:
                        n_margin_neg += 1
                    verdict = "pass" if (lang_ok and margin >= 0) else "review"
                    row = {"i": r["i"], "x": r["x"], "lang": L, "gen": out_t, "gold": r[L],
                           "lang_ok": lang_ok, "retries": len(attempts) - 1,
                           "attempts": attempts, "score_gen": round(s_gen, 4),
                           "score_gold": round(s_gold, 4), "margin": round(margin, 4),
                           "verdict": verdict}
                    lines.append(row)
                    if (not lang_ok) or margin < ERR_MARGIN:
                        n_err += 1
                        errs.append({"state": r["x"], "lang": L, "good": r[L], "bad": out_t,
                                     "margin": round(margin, 4), "lang_ok": lang_ok,
                                     "src": "pipeline_full"})
        for l in lines:
            run_fh.write(json.dumps(l, ensure_ascii=False) + "\n")
        for e in errs:
            err_fh.write(json.dumps(e, ensure_ascii=False) + "\n")
        run_fh.flush()
        err_fh.flush()
        done += len(block)
        json.dump({"done": done, "ts": time.strftime("%H:%M:%S")}, open(prog_f, "w"))

        dt = time.perf_counter() - t0
        if (start // BLOCK) % 5 == 0:
            print(f"[{time.strftime('%H:%M:%S')}] {done}/{len(rows_all)} phrases "
                  f"| {n_gen} gens | {dt/60:.1f} min | {n_gen/max(dt,1):.2f} gen/s "
                  f"| retry {n_retry} langfail {n_langfail} "
                  f"| margin<0: {n_margin_neg} ({100*n_margin_neg/max(n_gen,1):.1f}%) "
                  f"| errors {n_err}", flush=True)

        if not reported and done >= VALIDATION_N:
            reported = True
            vals = [json.loads(l) for l in open(run_f, encoding="utf-8")][-VALIDATION_N * 4:]
            ms = [v["margin"] for v in vals]
            print("\n" + "=" * 60, flush=True)
            print(f"VALIDATION SNAPSHOT @ {VALIDATION_N} phrases ({len(vals)} rows)", flush=True)
            print(f"  margin: mean {np.mean(ms):.4f} | median {np.median(ms):.4f} "
                  f"| p5 {np.percentile(ms,5):.4f} | min {min(ms):.4f}", flush=True)
            bylang = {}
            for v in vals:
                bylang.setdefault(v["lang"], []).append(v["margin"])
            for L, mv in sorted(bylang.items()):
                print(f"  [{L}] margin mean {np.mean(mv):.4f} | neg {sum(1 for m in mv if m<0)}/{len(mv)}", flush=True)
            print(f"  lang_ok: {100*sum(v['lang_ok'] for v in vals)/len(vals):.2f}% | "
                  f"retries: {sum(v['retries'] for v in vals)} | errors: {n_err}", flush=True)
            print("=" * 60 + "\n", flush=True)

    print("FULL RUN COMPLETE", flush=True)


if __name__ == "__main__":
    main()
