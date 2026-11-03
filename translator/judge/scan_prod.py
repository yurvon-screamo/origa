#!/usr/bin/env python3
"""FULL PROD SCAN: v6.2 judge over all cdn phrases x 5 locales (RAW readout, batched).

781k items; length-bucketed batches of 32, logits gathered per-row at the last
real token (right-pad + attention mask). Resumable (progress file). Output:
scan_results.jsonl + suspects (calibrated per-locale thresholds) + summary.
"""
import glob
import json
import sys
import time
from collections import Counter
from pathlib import Path

sys.path.insert(0, "/tmp/opencode/lfm-spike/ab")
from d1pkg.prompt import as_question, render  # noqa: E402
from d1pkg.slots import output_head, text_backbone  # noqa: E402

import torch
from peft import PeftModel
from transformers import AutoModel, AutoTokenizer

MODEL_ID = "LiquidAI/d1-2B"
ADAPTER = Path("/home/yurvon/xpu-model-bench/runs/lfm3-ab/d1_2b_judge_v62/adapter")
OUT_DIR = Path("/home/yurvon/xpu-model-bench/runs/lfm3-ab/prod_scan")
BS = 16
PROGRESS_EVERY = 50  # batches -> flush results + progress
TH = {"ru": 0.80, "en": 0.80, "vi": 0.85, "ko": 0.90, "zh-Hans": 0.98}

FAITHFUL_Q = {
    "type": "noul",
    "instructions": (
        "Is the translation a faithful rendering of the Japanese text into "
        "the target locale (correct meaning, no hallucination, no omission, "
        "no wrong language)?"
    ),
    "criteria": {
        "false": "The translation is NOT faithful: wrong meaning, hallucinated or "
                 "dropped details, meaning inversion, wrong names, or wrong language",
        "true": "The translation IS faithful: exact meaning preserved, nothing "
                "invented or dropped, correct names, fully in the target language",
    },
}
LANG_NAME = {"ru": "Russian", "en": "English", "vi": "Vietnamese", "ko": "Korean",
             "zh-Hans": "Chinese (Simplified)", "zh-Hant-TW": "Traditional Chinese (Taiwan)"}


def load_items():
    jobs = []
    for f in sorted(glob.glob("/home/yurvon/origa/cdn/phrases/data/p*.json")):
        for p in json.load(open(f, encoding="utf-8")):
            for lg in ("ru", "en", "vi", "ko"):
                if p.get(lg):
                    jobs.append({"i": p["i"], "lang": lg, "x": p["x"], "t": p[lg]})
    for l in open("/home/yurvon/origa-translator/data/pipeline/zh/zh_translations.jsonl", encoding="utf-8"):
        r = json.loads(l)
        if r.get("zh_ok"):
            jobs.append({"i": r["i"], "lang": "zh-Hans", "x": r["x"], "t": r["zh"]})
    return jobs


def main():
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    res_path = OUT_DIR / "scan_results.jsonl"
    done_path = OUT_DIR / "progress.json"

    jobs = load_items()
    done = 0
    if done_path.exists():
        done = json.loads(done_path.read_text())["done"]
    print(f"items: {len(jobs)} | resume from {done}", flush=True)
    if done >= len(jobs):
        print("already complete")
        return
    jobs = jobs[done:]

    tok = AutoTokenizer.from_pretrained(MODEL_ID, trust_remote_code=True)
    tok.truncation_side = "left"
    q = as_question(FAITHFUL_Q)
    yes_id = tok("Yes", add_special_tokens=False)["input_ids"][0]
    no_id = tok("No", add_special_tokens=False)["input_ids"][0]
    assert len(tok("Yes", add_special_tokens=False)["input_ids"]) == 1

    model = AutoModel.from_pretrained(MODEL_ID, trust_remote_code=True,
                                      dtype=torch.bfloat16).to("xpu").eval()
    model = PeftModel.from_pretrained(model, ADAPTER).eval()

    backbone = text_backbone(model)
    head = output_head(model)

    def batch_prompts(items):
        sts = [json.dumps({"japanese": it["x"], "translation": it["t"],
                           "target_locale": f"{it['lang']} ({LANG_NAME[it['lang']]})"},
                          ensure_ascii=False) for it in items]
        prompts = [render(tokenizer=tok, state=st, q=q, bos=tok.bos_token) for st in sts]
        encs = [tok(p, truncation=True, max_length=448, add_special_tokens=False)["input_ids"]
                for p in prompts]
        ml = max(len(e) for e in encs)
        ids = torch.tensor([e + [tok.pad_token_id] * (ml - len(e)) for e in encs]).to("xpu")
        attn = torch.tensor([[1] * len(e) + [0] * (ml - len(e)) for e in encs]).to("xpu")
        last = torch.tensor([len(e) - 1 for e in encs]).to("xpu")
        return ids, attn, last

    t0 = time.perf_counter()
    n_done_run = 0
    out_f = open(res_path, "a", encoding="utf-8")
    # length-ish bucketing: sort chunk-wise by rough prompt size, keep phrase order inside chunks
    CHUNK = BS * 50
    with torch.no_grad():
        for cs in range(0, len(jobs), CHUNK):
            chunk = jobs[cs:cs + CHUNK]
            chunk.sort(key=lambda it: len(it["t"]) + len(it["x"]))
            for s in range(0, len(chunk), BS):
                items = chunk[s:s + BS]
                ids, attn, last = batch_prompts(items)
                hidden = backbone(input_ids=ids, attention_mask=attn, use_cache=False).last_hidden_state
                rows = hidden[torch.arange(len(items), device="xpu"), last]
                row_logits = head(rows)
                p = torch.softmax(row_logits[:, [no_id, yes_id]].float(), -1)[:, 1]
                for it, pv in zip(items, p.tolist()):
                    out_f.write(json.dumps({"i": it["i"], "lang": it["lang"],
                                            "faithful": round(pv, 4)}, ensure_ascii=False) + "\n")
                n_done_run += len(items)
                if (n_done_run // BS) % PROGRESS_EVERY == 0 and n_done_run % (BS * PROGRESS_EVERY) < BS:
                    out_f.flush()
                    done_path.write_text(json.dumps({"done": done + n_done_run}))
                    rate = n_done_run / (time.perf_counter() - t0)
                    eta = (len(jobs) - n_done_run) / max(0.1, rate) / 3600
                    print(f"{done + n_done_run}/{done + len(jobs)} {rate:.1f}/s eta {eta:.1f}h", flush=True)
                    torch.xpu.empty_cache()

    out_f.flush()
    done_path.write_text(json.dumps({"done": done + n_done_run}))
    print(f"SCAN DONE: {done + n_done_run} items", flush=True)


if __name__ == "__main__":
    main()
