#!/usr/bin/env python3
"""SFT LoRA on the QAT w4a16-ct Gemma (dequantized-on-load, QAT weight values).

Same recipe as train_sft_ru.py but:
- base = google/gemma-4-E2B-it-qat-w4a16-ct loaded via
  CompressedTensorsConfig(run_compressed=False): packed int4 -> real nn.Linear
  with bf16 QAT-trained weight values (no bnb on top)
- LoRA targets are collected from actual module types (text stack only:
  vision/audio towers excluded)
- masked CE (262k vocab), AdamW + cosine, val every N steps
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

EPOCHS = 1
BS = 4
ACCUM = 8
LR = 2e-4
WARMUP = 0.03
MAXLEN = 256
VAL_EVERY = 250
VAL_N = 400
SEED = 42

random.seed(SEED)
torch.manual_seed(SEED)

PROJ = ("q_proj", "k_proj", "v_proj", "o_proj", "gate_proj", "up_proj", "down_proj")


def load_jsonl(p: Path) -> list[dict]:
    return [json.loads(line) for line in open(p, encoding="utf-8")]


def encode_example(tok, messages):
    full = tok.apply_chat_template(messages, tokenize=False)
    ids = tok(full, truncation=True, max_length=MAXLEN, add_special_tokens=False)[
        "input_ids"
    ]
    prefix = tok.apply_chat_template(
        messages[:-1], tokenize=False, add_generation_prompt=True
    )
    plen = len(
        tok(prefix, truncation=True, max_length=MAXLEN, add_special_tokens=False)[
            "input_ids"
        ]
    )
    labels = [-100] * min(plen, len(ids)) + ids[min(plen, len(ids)) :]
    return ids, labels


class Batcher:
    """Deterministic order (own RNG seed) for auto-resume; optional pre-encoding."""

    def __init__(self, rows, tok, bs, shuffle=True, pretokenize=False, skip_rows=0):
        self.rows, self.tok, self.bs, self.shuffle = rows, tok, bs, shuffle
        self.skip_rows = min(skip_rows, len(rows))
        if pretokenize:
            t0 = time.perf_counter()
            self.enc = [encode_example(tok, r["messages"]) for r in rows]
            print(
                f"pretokenize: {len(rows)} rows in {time.perf_counter() - t0:.1f}s",
                flush=True,
            )
        else:
            self.enc = None

    def epoch(self):
        order = list(range(len(self.rows)))
        if self.shuffle:
            random.Random(12345).shuffle(
                order
            )  # fixed independent seed -> reproducible
        order = order[self.skip_rows :] if self.skip_rows else order
        for s in range(0, len(order), self.bs):
            batch = [self.rows[i] for i in order[s : s + self.bs]]
            enc = [
                self.enc[i]
                if self.enc is not None
                else encode_example(self.tok, r["messages"])
                for i, r in zip(order[s : s + self.bs], batch)
            ]
            ml = max(len(ids) for ids, _ in enc)
            input_ids, labels, attn = [], [], []
            for ids, labels_ in enc:
                pad = ml - len(ids)
                input_ids.append(ids + [self.tok.pad_token_id] * pad)
                labels.append(labels_ + [-100] * pad)
                attn.append([1] * len(ids) + [0] * pad)
            yield (torch.tensor(input_ids), torch.tensor(labels), torch.tensor(attn))

    def __len__(self):
        return math.ceil((len(self.rows) - self.skip_rows) / self.bs)

    def __iter__(self):
        return self.epoch()


def main():
    import argparse

    ap = argparse.ArgumentParser()
    ap.add_argument("--data", default=str(DATA / "sft_multi_c10k.jsonl"))
    ap.add_argument("--epochs", type=int, default=EPOCHS)
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--out", default="qat-scale/40k")
    ap.add_argument("--bs", type=int, default=BS)
    ap.add_argument("--accum", type=int, default=ACCUM)
    ap.add_argument("--resume", default="", help="adapter dir to continue from")
    ap.add_argument(
        "--save-every-rows",
        type=int,
        default=0,
        help="save adapter every N data rows (0 = epoch-end only)",
    )
    ap.add_argument(
        "--skip-rows",
        type=int,
        default=0,
        help="skip first N rows of deterministic order (auto-resume)",
    )
    ap.add_argument(
        "--start-step",
        type=int,
        default=0,
        help="optimizer step counter start (auto-resume)",
    )
    ap.add_argument("--pretokenize", action="store_true")
    ap.add_argument("--towers-cpu", action="store_true")
    a = ap.parse_args()
    ckpt = Path(os.environ.get("ORIGA_RUNS_DIR", "runs")) / a.out
    ckpt.mkdir(parents=True, exist_ok=True)
    torch.manual_seed(SEED)
    train_rows = load_jsonl(Path(a.data))
    if a.limit:
        train_rows = train_rows[: a.limit]
    val_rows = random.sample(load_jsonl(DATA / "sft_ru_val.jsonl"), VAL_N)
    print(
        f"train {len(train_rows)} | val-sample {len(val_rows)} | "
        f"skip {a.skip_rows} | start-step {a.start_step}",
        flush=True,
    )

    tok = AutoTokenizer.from_pretrained(QAT_PATH)
    if tok.pad_token is None:
        tok.pad_token = tok.eos_token

    print("loading QAT (dequantize-on-load, thick) ...", flush=True)
    t0 = time.perf_counter()
    model = AutoModelForCausalLM.from_pretrained(
        QAT_PATH,
        dtype=torch.bfloat16,
        low_cpu_mem_usage=False,
        attn_implementation="sdpa",
        quantization_config=CompressedTensorsConfig(run_compressed=False),
    )
    model = model.to("xpu")
    print(f"loaded in {time.perf_counter() - t0:.0f}s", flush=True)

    if a.towers_cpu:
        moved = []
        for name in ("vision_tower", "audio_tower"):
            tower = getattr(model.model, name, None)
            if tower is not None:
                tower.to("cpu")
                moved.append(name)
        print(f"towers->cpu: {moved}", flush=True)

    targets = [
        n
        for n, m in model.named_modules()
        if n.split(".")[-1] in PROJ
        and type(m).__name__ == "Linear"
        and getattr(m, "weight", None) is not None
        and "vision" not in n
        and "audio" not in n
        and "multimodal" not in n
    ]
    print(f"lora targets: {len(targets)}", flush=True)

    if a.resume:
        from peft import PeftModel

        model = PeftModel.from_pretrained(model, a.resume, is_trainable=True)
        print(f"resumed from {a.resume}", flush=True)
    else:
        model = get_peft_model(
            model,
            LoraConfig(
                r=16,
                lora_alpha=32,
                lora_dropout=0.05,
                bias="none",
                task_type="CAUSAL_LM",
                target_modules=targets,
            ),
        )
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

    train_loader = Batcher(
        train_rows, tok, a.bs, pretokenize=a.pretokenize, skip_rows=a.skip_rows
    )
    val_loader = Batcher(val_rows, tok, a.bs, shuffle=False)
    steps_per_epoch = math.ceil(len(train_loader) / a.accum)
    total_steps = steps_per_epoch * a.epochs
    warmup_steps = int((total_steps + a.start_step) * WARMUP)

    # checkpoint cadence in optimizer steps
    save_every_steps = (
        math.ceil(a.save_every_rows / (a.bs * a.accum)) if a.save_every_rows else 0
    )

    opt = torch.optim.AdamW(
        [p for p in model.parameters() if p.requires_grad], lr=LR, weight_decay=0.01
    )

    def lr_at(step):
        if step < warmup_steps:
            return LR * step / max(1, warmup_steps)
        p = (step - warmup_steps) / max(1, total_steps - warmup_steps)
        return LR * 0.5 * (1 + math.cos(math.pi * min(p, 1.0)))

    def run_val():
        model.eval()
        tot, n = 0.0, 0
        with torch.no_grad():
            for ids, labels, attn in val_loader:
                ids, labels, attn = ids.to("xpu"), labels.to("xpu"), attn.to("xpu")
                loss = masked_ce(ids, labels, attn)
                tot += loss.item() * len(ids)
                n += len(ids)
        model.train()
        return tot / max(1, n)

    print(
        f"steps/epoch {steps_per_epoch} | total {total_steps} | "
        f"start {a.start_step} | save_every_steps {save_every_steps}",
        flush=True,
    )
    step = a.start_step
    base_step = a.start_step
    lr_now = lr_at(max(step, 1))
    rows_done_base = a.skip_rows
    t0 = time.perf_counter()
    for ep in range(1, a.epochs + 1):
        run_loss, rn = 0.0, 0
        opt.zero_grad()
        micro = 0
        for ids, labels, attn in train_loader.epoch():
            ids, labels, attn = ids.to("xpu"), labels.to("xpu"), attn.to("xpu")
            out_loss = masked_ce(ids, labels, attn)
            (out_loss / a.accum).backward()
            run_loss += out_loss.item() * len(ids)
            rn += len(ids)
            micro += 1
            if micro % a.accum == 0:
                step += 1
                lr_now = lr_at(step)
                for g in opt.param_groups:
                    g["lr"] = lr_now
                torch.nn.utils.clip_grad_norm_(
                    [p for p in model.parameters() if p.requires_grad], 1.0
                )
                opt.step()
                opt.zero_grad()
                if step % 20 == 0:
                    torch.xpu.empty_cache()  # every 20 steps: pool bloats to 28GB otherwise (iGPU swap)
                if save_every_steps and step % save_every_steps == 0:
                    rows_done = rows_done_base + (step - base_step) * a.bs * a.accum
                    ck_mid = ckpt / f"rows{rows_done}"
                    model.save_pretrained(ck_mid)
                    with open(ckpt / "progress.json", "w") as pf:
                        json.dump(
                            {"step": step, "rows": rows_done, "adapter": str(ck_mid)},
                            pf,
                        )
                    print(f"saved {ck_mid} (rows {rows_done})", flush=True)
            if step > 0 and step % 25 == 0:
                mem = torch.xpu.memory_allocated() / 2**30
                gf, gt = torch.xpu.mem_get_info()
                rate = (step - base_step) / (time.perf_counter() - t0)
                eta = (total_steps - step) / max(rate, 0.1) / 60
                print(
                    f"ep{ep} step {step}/{total_steps} "
                    f"loss={run_loss / rn:.4f} lr={lr_now:.2e} "
                    f"{rate:.2f} it/s eta={eta:.0f}min mem={mem:.1f}GB "
                    f"pool_free={gf / 2**30:.1f}GB",
                    flush=True,
                )
            if step % VAL_EVERY == 0 and step > 0:
                print(f"  val_loss={run_val():.4f}", flush=True)
        vl = run_val()
        print(f"== epoch {ep} done: val_loss={vl:.4f} ==", flush=True)
        ck = ckpt / f"ep{ep}"
        model.save_pretrained(ck)
        with open(ckpt / "progress.json", "w") as pf:
            json.dump(
                {
                    "step": step,
                    "rows": rows_done_base + (step - base_step) * a.bs * a.accum,
                    "adapter": str(ck),
                },
                pf,
            )
        print(f"saved {ck}", flush=True)

    print(f"DONE in {(time.perf_counter() - t0) / 60:.0f} min", flush=True)


if __name__ == "__main__":
    main()
