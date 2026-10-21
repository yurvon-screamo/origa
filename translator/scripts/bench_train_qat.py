#!/usr/bin/env python3
"""Benchmark trainer variants for the 600k run (QAT base, LoRA).

Toggles:
  --bs N            micro-batch (4 = current baseline)
  --towers-cpu      move vision/audio towers to CPU (frees ~1.5GB VRAM)
  --pretokenize     encode the whole dataset once (no jinja in the loop)
  --attn IMPL       attn_implementation: sdpa | eager (default: sdpa)

Measures: optimizer-step time, throughput rows/hour, peak mem.
"""
import json
import os
import math
import random
import time
from pathlib import Path

import torch
import torch.nn.functional as F
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import LoraConfig, get_peft_model

try:
    from transformers import CompressedTensorsConfig
except ImportError:
    from compressed_tensors.config import CompressedTensorsConfig

DATA = Path(os.environ.get("ORIGA_DATA_DIR", "data"))
QAT_PATH = "google/gemma-4-E2B-it-qat-w4a16-ct"

LR = 2e-4
MAXLEN = 256
ACCUM = 8
SEED = 42
PROJ = ("q_proj", "k_proj", "v_proj", "o_proj", "gate_proj", "up_proj", "down_proj")


def load_jsonl(p: Path) -> list[dict]:
    return [json.loads(l) for l in open(p, encoding="utf-8")]


def encode_example(tok, messages):
    full = tok.apply_chat_template(messages, tokenize=False)
    ids = tok(full, truncation=True, max_length=MAXLEN, add_special_tokens=False)["input_ids"]
    prefix = tok.apply_chat_template(messages[:-1], tokenize=False, add_generation_prompt=True)
    plen = len(tok(prefix, truncation=True, max_length=MAXLEN, add_special_tokens=False)["input_ids"])
    labels = [-100] * min(plen, len(ids)) + ids[min(plen, len(ids)):]
    return ids, labels


class Batcher:
    """pre-encode once, then just pad/shuffle per epoch."""

    def __init__(self, rows, tok, bs, shuffle=True, pretokenize=False):
        self.bs, self.shuffle = bs, shuffle
        if pretokenize:
            t0 = time.perf_counter()
            self.enc = [encode_example(tok, r["messages"]) for r in rows]
            print(f"  pretokenize: {len(rows)} rows in {time.perf_counter()-t0:.1f}s", flush=True)
        else:
            self.enc = None
        self.rows = rows
        self.tok = tok

    def epoch(self):
        order = list(range(len(self.rows)))
        if self.shuffle:
            random.shuffle(order)
        for s in range(0, len(order), self.bs):
            idx = order[s:s + self.bs]
            enc = [self.enc[i] if self.enc else encode_example(self.tok, self.rows[i]["messages"])
                   for i in idx]
            ml = max(len(ids) for ids, _ in enc)
            input_ids, labels, attn = [], [], []
            for ids, labels_ in enc:
                pad = ml - len(ids)
                input_ids.append(ids + [self.tok.pad_token_id] * pad)
                labels.append(labels_ + [-100] * pad)
                attn.append([1] * len(ids) + [0] * pad)
            yield (torch.tensor(input_ids), torch.tensor(labels), torch.tensor(attn))

    def __len__(self):
        return math.ceil(len(self.rows) / self.bs)


def main():
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=str(DATA / "bench_4k.jsonl"))
    ap.add_argument("--bs", type=int, default=4)
    ap.add_argument("--towers-cpu", action="store_true")
    ap.add_argument("--pretokenize", action="store_true")
    ap.add_argument("--attn", default="sdpa")
    ap.add_argument("--label", default="run")
    a = ap.parse_args()

    torch.manual_seed(SEED)
    random.seed(SEED)
    train_rows = load_jsonl(Path(a.data))

    tok = AutoTokenizer.from_pretrained(QAT_PATH)
    if tok.pad_token is None:
        tok.pad_token = tok.eos_token

    t0 = time.perf_counter()
    model = AutoModelForCausalLM.from_pretrained(
        QAT_PATH, dtype=torch.bfloat16, low_cpu_mem_usage=False,
        attn_implementation=a.attn,
        quantization_config=CompressedTensorsConfig(run_compressed=False))
    model = model.to("xpu")
    print(f"[{a.label}] attn={model.config._attn_implementation} loaded {time.perf_counter()-t0:.0f}s", flush=True)

    if a.towers_cpu:
        moved = []
        for name in ("vision_tower", "audio_tower"):
            tower = getattr(model.model, name, None)
            if tower is not None:
                tower.to("cpu")
                moved.append(name)
        print(f"[{a.label}] towers->cpu: {moved}", flush=True)

    targets = [n for n, m in model.named_modules()
               if n.split(".")[-1] in PROJ and type(m).__name__ == "Linear"
               and getattr(m, "weight", None) is not None
               and "vision" not in n and "audio" not in n and "multimodal" not in n]
    model = get_peft_model(model, LoraConfig(
        r=16, lora_alpha=32, lora_dropout=0.05, bias="none",
        task_type="CAUSAL_LM", target_modules=targets))
    model.train()
    model.enable_input_require_grads()
    core = model.get_base_model()
    backbone, lm_head = core.model, core.get_output_embeddings()

    def masked_ce(ids, labels, attn):
        hidden = backbone(input_ids=ids, attention_mask=attn).last_hidden_state
        sh, sl = hidden[:, :-1, :], labels[:, 1:]
        m = sl != -100
        logits = lm_head(sh[m]).float()
        return F.cross_entropy(logits, sl[m])

    loader = Batcher(train_rows, tok, a.bs, pretokenize=a.pretokenize)
    steps_total = math.ceil(len(loader) / ACCUM)
    opt = torch.optim.AdamW([p for p in model.parameters() if p.requires_grad],
                            lr=LR, weight_decay=0.01)

    f0, t0 = torch.xpu.mem_get_info()
    t0 = time.perf_counter()
    step, micro = 0, 0
    opt.zero_grad()
    last_loss = 0.0
    for ids, labels, attn in loader.epoch():
        ids, labels, attn = ids.to("xpu"), labels.to("xpu"), attn.to("xpu")
        loss = masked_ce(ids, labels, attn)
        (loss / ACCUM).backward()
        last_loss = loss.item()
        micro += 1
        if micro % ACCUM == 0:
            step += 1
            opt.step(); opt.zero_grad()
            if step % 50 == 0:
                torch.xpu.empty_cache()
    dt = time.perf_counter() - t0
    f1, t1 = torch.xpu.mem_get_info()
    rows_per_hour = len(train_rows) / dt * 3600
    print(f"[{a.label}] bs={a.bs} towers_cpu={a.towers_cpu} pretok={a.pretokenize} "
          f"attn={model.config._attn_implementation}", flush=True)
    print(f"[{a.label}] steps={steps_total} time={dt/60:.1f}min "
          f"({dt/steps_total:.2f}s/step) | rows/hour={rows_per_hour:,.0f} "
          f"| mem_used={(t1-f1)/2**30:.1f}GB | last_loss={last_loss:.3f}", flush=True)


if __name__ == "__main__":
    main()
