#!/usr/bin/env python3
"""Translate cdn/examples/data/*.json sentences into ru/vi/ko via origa-translator.

en comes from the Tanaka corpus (already in the data). For ru/vi/ko:
  1. greedy generation via vLLM :8091 (tr600k LoRA), EN instruction,
     language-routing check, JA-instruction retry (prod recipe)
  2. algorithmic flags: empty / equals ja / latin-script impurities
  3. judge v3 score (query=ja, passage=cand vs passage=en) — recorded,
     NOT used to reject: absolute scores are not calibrated across locales.

Checkpointed per block with auto-resume (progress in /tmp/opencode/examples_tr/).
Merged output is written back into cdn/examples/data/s*.json.
"""

from __future__ import annotations

import json
import re
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import numpy as np
import requests
import torch
import torch.nn.functional as F

REPO = Path(__file__).resolve().parent.parent
DATA = REPO / "cdn" / "examples" / "data"
WORK = Path("/tmp/opencode/examples_tr")
TR_URL = "http://127.0.0.1:8091/v1/chat/completions"
EMB_URL = "http://127.0.0.1:8090/v1/embeddings"
HEADS = Path("/home/yurvon/origa-translator/model/judge_heads_v3.pt")
LOCALES = ("ru", "vi", "ko")
BLOCK = 48
CONC = 15

SYS_EN = "You are a skilled Japanese translator. Translate the sentence accurately and naturally."
SYS_JA = "あなたは熟練した日本語翻訳者です。文を正確かつ自然に翻訳してください。"
INSTR_EN = "Translate the following Japanese sentence into {lname}:\n{x}"
INSTR_JA = {"vi": "次の日本語の文をベトナム語に翻訳してください:\n{x}",
            "ru": "次の日本語の文をロシア語に翻訳してください:\n{x}",
            "ko": "次の日本語の文を韓国語に翻訳してください:\n{x}"}
LNAME = {"vi": "Vietnamese", "ru": "Russian", "ko": "Korean"}

RE_HANGUL = re.compile(r"[가-힣]")
RE_CYR = re.compile(r"[А-ЯЁа-яё]")
RE_KANA = re.compile(r"[ひ-んア-ンー]")
RE_CJK = re.compile(r"[一-龥]")
RE_LATIN = re.compile(r"[A-Za-z]")
RE_VI_MARK = re.compile(
    r"[ăâêôơưđĂÂÊÔƠƯĐ]|[áàảãạấầẩẫậắằẳẵặéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ]"
)


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
    """Judge v3 heads: query (ja state) x passage (translation) cosine."""

    def __init__(self, dim):
        super().__init__()
        self.s = torch.nn.Sequential(torch.nn.Linear(dim, 256), torch.nn.GELU(),
                                     torch.nn.Linear(256, 256))
        self.a = torch.nn.Sequential(torch.nn.Linear(dim, 256), torch.nn.GELU(),
                                     torch.nn.Linear(256, 256))

    def forward(self, sv, av):
        zq = F.normalize(self.s(sv), dim=-1)
        zc = F.normalize(self.a(av), dim=-1)
        return (zq * zc).sum(-1)


def gen_one(x: str, lang: str) -> tuple[str, bool, int]:
    """Generate + language check + JA-retry. Returns (text, lang_ok, retries)."""
    def call(system: str, instr: str) -> str:
        r = requests.post(TR_URL, json={
            "model": "tr600k",
            "messages": [{"role": "system", "content": system},
                         {"role": "user", "content": instr}],
            "temperature": 0.0, "max_tokens": 200,
        }, timeout=180)
        r.raise_for_status()
        return r.json()["choices"][0]["message"]["content"].strip()

    out = call(SYS_EN, INSTR_EN.format(lname=LNAME[lang], x=x))
    retries = 0
    if detect_language(out) != lang:
        out2 = call(SYS_JA, INSTR_JA[lang].format(x=x))
        retries = 1
        if detect_language(out2) == lang or len(out2) > len(out):
            out = out2
    return out, detect_language(out) == lang, retries


def embed(texts: list[str], prefix: str) -> dict[str, np.ndarray]:
    """e5b embeddings with the judge-v3 `query:`/`passage:` prefixes."""
    out: dict[str, np.ndarray] = {}
    for s in range(0, len(texts), 64):
        chunk = texts[s : s + 64]
        payload = [(prefix + t)[:1000] for t in chunk]
        r = requests.post(EMB_URL, json={"model": "e5b", "input": payload}, timeout=600)
        r.raise_for_status()
        for t, item in zip(chunk, r.json()["data"]):
            out[t] = np.asarray(item["embedding"], dtype=np.float32)
    return out


def main() -> int:
    WORK.mkdir(parents=True, exist_ok=True)
    files = sorted(DATA.glob("s*.json"))
    rows: list[dict] = []
    for f in files:
        rows += json.loads(f.read_text(encoding="utf-8"))
    print(f"sentences: {len(rows)}", flush=True)

    done_f = WORK / "translations.jsonl"
    done: dict[tuple[int, str], dict] = {}
    if done_f.exists():
        for line in done_f.read_text(encoding="utf-8").splitlines():
            if line.strip():
                d = json.loads(line)
                done[(d["i"], d["lang"])] = d
    print(f"resume: {len(done)} translations already present", flush=True)

    judge_ok = False
    judge_retry_at = 0
    JUDGE_RETRY_BLOCKS = 20  # probe the embeddings server every N blocks
    heads = None
    try:
        ck = torch.load(HEADS, map_location="cpu")
        heads = Heads(ck["dim"])
        heads.load_state_dict(ck["heads"])
        heads.eval()
        judge_ok = True
        print("judge v3 heads loaded", flush=True)
    except Exception as e:  # judge is optional: embeddings server may be down
        print(f"judge unavailable ({e}); running generation-only", flush=True)

    pending = [
        (r, L) for r in rows for L in LOCALES if (r["i"], L) not in done
    ]
    print(f"to translate: {len(pending)}", flush=True)
    t0 = time.perf_counter()
    n_done = n_retry = n_langfail = n_flag = 0

    out_f = open(done_f, "a", encoding="utf-8")
    try:
        for start in range(0, len(pending), BLOCK):
            block = pending[start : start + BLOCK]
            with ThreadPoolExecutor(CONC) as ex:
                gens = list(ex.map(lambda j: gen_one(j[0]["x"], j[1]), block))

            scores: dict[tuple[int, str], tuple[float, float]] = {}
            if judge_ok:
                try:
                    queries = list(dict.fromkeys(j[0]["x"] for j in block))
                    passages = list(
                        dict.fromkeys(
                            [g for g, _, _ in gens] + [j[0]["en"] for j in block]
                        )
                    )
                    Vq = embed(queries, "query: ")
                    Vp = embed(passages, "passage: ")
                    with torch.no_grad():
                        for (r, L), (gen, _, _) in zip(block, gens):
                            q = torch.from_numpy(Vq[r["x"]]).unsqueeze(0)
                            cands = torch.stack([
                                torch.from_numpy(Vp[gen]),
                                torch.from_numpy(Vp[r["en"]]),
                            ])
                            sc = heads(q, cands)
                            scores[(r["i"], L)] = (float(sc[0]), float(sc[1]))
                except (requests.RequestException, RuntimeError) as e:
                    print(f"judge unavailable ({e.__class__.__name__}); "
                          "continuing generation-only, will retry later", flush=True)
                    judge_ok = False
                    judge_retry_at = n_done + JUDGE_RETRY_BLOCKS * BLOCK
            elif not judge_ok and n_done >= judge_retry_at:
                # embeddings server may have come up mid-run
                try:
                    embed(["テスト"], "query: ")
                    ck = torch.load(HEADS, map_location="cpu")
                    heads = Heads(ck["dim"])
                    heads.load_state_dict(ck["heads"])
                    heads.eval()
                    judge_ok = True
                    print("judge reconnected", flush=True)
                except Exception:
                    judge_retry_at = n_done + JUDGE_RETRY_BLOCKS * BLOCK

            for (r, L), (gen, lang_ok, retries) in zip(block, gens):
                flags = []
                if not gen:
                    flags.append("empty")
                if gen.strip() == r["x"].strip():
                    flags.append("equals_ja")
                det = detect_language(gen)
                if det != L:
                    flags.append(f"lang:{det}")
                if L in ("ru", "ko"):
                    # Vietnamese IS latin script; for ru/ko flag heavy latin
                    # admixtures (names/brands below the threshold are fine).
                    latin_words = len(re.findall(r"[A-Za-z]{2,}", gen))
                    if latin_words >= 3:
                        flags.append("latin_mix")
                m = scores.get((r["i"], L))
                rec = {
                    "i": r["i"], "lang": L, "gen": gen, "lang_ok": lang_ok,
                    "retries": retries, "flags": flags,
                    "score_gen": round(m[0], 4) if m else None,
                    "score_en": round(m[1], 4) if m else None,
                }
                done[(r["i"], L)] = rec
                out_f.write(json.dumps(rec, ensure_ascii=False) + "\n")
                n_done += 1
                n_retry += retries
                if not lang_ok:
                    n_langfail += 1
                if flags:
                    n_flag += 1
            out_f.flush()
            n_gen = len(done)
            if n_gen % 480 < BLOCK:
                el = time.perf_counter() - t0
                print(f"progress {n_gen}/{len(pending) + len(done)} "
                      f"({n_gen * 1.0 / max(el, 1):.1f}/s, "
                      f"langfail={n_langfail}, flagged={n_flag})", flush=True)
    finally:
        out_f.close()

    # merge into data files
    by_file: dict[Path, list[dict]] = {f: json.loads(f.read_text(encoding="utf-8")) for f in files}
    for f, part in by_file.items():
        changed = False
        for r in part:
            for L in LOCALES:
                rec = done.get((r["i"], L))
                if rec and rec["gen"] and not rec["flags"]:
                    r[L] = rec["gen"]
                    changed = True
        if changed:
            f.write_text(json.dumps(part, ensure_ascii=False, separators=(",", ":")),
                         encoding="utf-8")
    filled = sum(
        1 for part in by_file.values() for r in part if all(r.get(L) for L in LOCALES)
    )
    print(f"merged; sentences with all 3 translations: {filled}/{len(rows)}", flush=True)
    print(f"stats: retried={n_retry} langfail={n_langfail} flagged={n_flag}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
