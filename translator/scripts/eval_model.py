#!/usr/bin/env python3
"""Evaluate a trained adapter: generate translations on holdout, score chrF.

Usage:
  eval_model.py --base <hf_id> --adapter <dir> --name <run-name> [--n 100]

- holdout: data/holdout_500x4.jsonl (never trained on), --n per locale
- thinking mode disabled for Qwen-family templates (enable_thinking=False,
  both Qwen3 and Qwen3.5 semantics); residual <think> blocks stripped
- batched generation, left padding
- metrics: chrF per locale + micro-avg; outputs eval/<name>/gens.jsonl + summary.json
"""

from __future__ import annotations

import argparse
import json
import os
import re
import time
from collections import defaultdict
from pathlib import Path

import sacrebleu
import torch
from peft import PeftModel
from transformers import AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig

ROOT = Path(os.environ.get("ORIGA_WORK_DIR", "."))
LOCALES = ("en", "vi", "ru", "ko")


def strip_think(text: str) -> str:
    """Remove any residual <think>...</think> blocks and dangling open tag."""
    text = re.sub(r"<think>.*?</think>", "", text, flags=re.DOTALL)
    text = re.sub(r"<think>.*$", "", text, flags=re.DOTALL)
    return text.strip()


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", required=True)
    ap.add_argument("--adapter", required=True)
    ap.add_argument("--name", required=True)
    ap.add_argument("--n", type=int, default=100, help="samples per locale")
    ap.add_argument(
        "--batch",
        type=int,
        default=1,
        help="generation batch; batch=1 is the only reliable mode on XPU "
        "(left-padded batched generation corrupts weak modes)",
    )
    a = ap.parse_args()

    rows = [
        json.loads(l) for l in open(ROOT / "data/holdout_500x4.jsonl", encoding="utf-8")
    ]
    by_lang: dict[str, list] = defaultdict(list)
    for r in rows:
        by_lang[r["lang"]].append(r)
    sample = []
    for lang in LOCALES:
        pool = sorted(by_lang[lang], key=lambda r: r["i"])  # deterministic
        sample += pool[: a.n]
    print(f"[{a.name}] eval sample: {len(sample)} ({a.n}/locale)", flush=True)

    tok = AutoTokenizer.from_pretrained(a.base)
    tok.padding_side = "left"
    if tok.pad_token is None:
        tok.pad_token = tok.eos_token

    bnb = BitsAndBytesConfig(
        load_in_4bit=True,
        bnb_4bit_quant_type="nf4",
        bnb_4bit_compute_dtype=torch.bfloat16,
        bnb_4bit_use_double_quant=True,
    )
    model = AutoModelForCausalLM.from_pretrained(
        a.base, quantization_config=bnb, device_map={"": "xpu"}, dtype=torch.bfloat16
    )
    model = PeftModel.from_pretrained(model, a.adapter).eval()

    out_dir = ROOT / "eval" / a.name
    out_dir.mkdir(parents=True, exist_ok=True)

    # build prompts once; enable_thinking=False covers Qwen3 (opt-in thinking)
    # and Qwen3.5 (default-off, kwarg harmless); ignored by LFM templates
    prompts = []
    for r in sample:
        msgs = [
            {
                "role": "system",
                "content": "You are a skilled Japanese translator. Translate the sentence accurately and naturally.",
            },
            {
                "role": "user",
                "content": f"Translate the following Japanese sentence into {r['lang'].upper()}:\n{r['x']}",
            },
        ]
        prompts.append(
            tok.apply_chat_template(
                msgs, tokenize=False, add_generation_prompt=True, enable_thinking=False
            )
        )

    t0 = time.perf_counter()
    gens: list[str] = []
    for i in range(0, len(prompts), a.batch):
        chunk = prompts[i : i + a.batch]
        enc = tok(
            chunk, return_tensors="pt", padding=True, truncation=True, max_length=512
        ).to("xpu")
        with torch.no_grad():
            out = model.generate(
                **enc,
                max_new_tokens=128,
                do_sample=False,
                pad_token_id=tok.pad_token_id,
            )
        for j in range(len(chunk)):
            gen = tok.decode(
                out[j][enc["input_ids"].shape[1] :], skip_special_tokens=True
            )
            gens.append(strip_think(gen))
        done = i + len(chunk)
        print(
            f"  {done}/{len(prompts)} ({done / (time.perf_counter() - t0):.1f} rec/s)",
            flush=True,
        )

    # per-locale metrics
    summary = {
        "name": a.name,
        "base": a.base,
        "adapter": a.adapter,
        "n_per_locale": a.n,
        "chrf": {},
        "chrf_avg": 0.0,
        "sec": round(time.perf_counter() - t0, 1),
    }
    all_hyp, all_ref = [], []
    with open(out_dir / "gens.jsonl", "w", encoding="utf-8") as f:
        for r, gen in zip(sample, gens):
            f.write(json.dumps({**r, "gen": gen}, ensure_ascii=False) + "\n")
    for lang in LOCALES:
        idx = [k for k, r in enumerate(sample) if r["lang"] == lang]
        hyps = [gens[k] for k in idx]
        refs = [sample[k]["gold"] for k in idx]
        chrf = sacrebleu.corpus_chrf(hyps, [refs]).score
        summary["chrf"][lang] = round(chrf, 2)
        all_hyp += hyps
        all_ref += refs
        print(f"  chrF {lang}: {chrf:.2f}", flush=True)
    summary["chrf_avg"] = round(sacrebleu.corpus_chrf(all_hyp, [all_ref]).score, 2)
    print(
        f"[{a.name}] chrF avg: {summary['chrf_avg']}  ({summary['sec']}s)", flush=True
    )

    with open(out_dir / "summary.json", "w", encoding="utf-8") as f:
        json.dump(summary, f, ensure_ascii=False, indent=1)


if __name__ == "__main__":
    main()
