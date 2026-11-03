#!/usr/bin/env python3
"""d1-2B judge v6 fine-tune: v5 pairs (noul-faithful) + tier block (score 0-2).

Two row kinds in judge_train_v6.jsonl:
  question="faithful": pair rows -> positive=>Yes, negative=>No (noul w/ criteria)
  question="tier":     item rows  => digit completion ("0" defective / "1" valid / "2" exact)

Same recipe as the v5/60k runs: LoRA r16 a32, bs4 x accum4, LR 1e-4 cosine,
length-bucketing, empty_cache/20 steps, ckpt+auto-resume/1000 steps, XPU.
"""
import json
import math
import random
import shutil
import sys
import time
from pathlib import Path

import torch
import torch.nn.functional as F
from huggingface_hub import snapshot_download
from peft import LoraConfig, PeftModel, get_peft_model
from transformers import AutoModel, AutoTokenizer

DATA = Path("/home/yurvon/origa-translator/data/judge/judge_train_v7.jsonl")
SNAP = Path(snapshot_download("LiquidAI/d1-2B"))
PKG = Path("/tmp/opencode/lfm-spike/ab/d1pkg")
PKG.mkdir(parents=True, exist_ok=True)
for f in SNAP.glob("*.py"):
    shutil.copy(f, PKG / f.name)
sys.path.insert(0, str(PKG.parent))
from d1pkg.prompt import as_question, render  # noqa: E402

OUT = Path("/home/yurvon/xpu-model-bench/runs/lfm3-ab/d1_2b_judge_v7")
MODEL_ID = "LiquidAI/d1-2B"

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
TIER_Q = json.loads(Path("/home/yurvon/origa-translator/data/judge/tier_question_v6.json").read_text())
TIER_OF = {"DEFECT": "0", "PARTIAL": "1", "OK": "2"}

LANG_NAME = {"ru": "Russian", "en": "English", "vi": "Vietnamese", "ko": "Korean",
             "zh-Hans": "Chinese (Simplified)", "zh-Hant-TW": "Traditional Chinese (Taiwan)"}

EPOCHS, BS, ACCUM, LR, WARMUP, SEED, MAXLEN = 1, 4, 4, 1e-4, 0.03, 42, 448
CKPT_EVERY = 1000

random.seed(SEED)
torch.manual_seed(SEED)


def state_of(state, translation, lang):
    return json.dumps({"japanese": state, "translation": translation,
                       "target_locale": f"{lang} ({LANG_NAME.get(lang, lang)})"}, ensure_ascii=False)


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    golden_states = set()
    for g in ("golden_v4c", "golden_zh_v1", "golden_tier_v1"):
        p = Path("/home/yurvon/origa-translator/data/judge") / f"{g}.jsonl"
        if not p.exists():
            continue
        for l in open(p, encoding="utf-8"):
            r = json.loads(l)
            golden_states.add((r["state"], r.get("lang")))
    # golden_tier rows leak-guard by exact (state, translation)
    tier_holdout = set()
    p = Path("/home/yurvon/origa-translator/data/judge/golden_tier_v1.jsonl")
    if p.exists():
        for l in open(p, encoding="utf-8"):
            r = json.loads(l)
            tier_holdout.add((r["state"], r["translation"]))

    rows = []
    for l in open(DATA, encoding="utf-8"):
        r = json.loads(l)
        if (r["state"], r["lang"]) in golden_states:
            continue
        neg = next((n for n in r["negatives"] if n and n != r["positive"]), None)
        if neg is None:
            continue
        rows.append(("faithful", r["state"], r["positive"], r["lang"], "Yes"))
        rows.append(("faithful", r["state"], neg, r["lang"], "No"))
    random.Random(7).shuffle(rows)
    val_rows = rows[:600]
    train_rows = rows[600:]
    print(f"rows: train {len(train_rows)} val {len(val_rows)} "
          f"(tier rows: {sum(1 for r in rows if r[0]=='tier')})", flush=True)

    tok = AutoTokenizer.from_pretrained(MODEL_ID, trust_remote_code=True)
    tok.truncation_side = "left"
    q_faithful = as_question(FAITHFUL_Q)
    q_tier = as_question(TIER_Q)
    assert len(tok("Yes", add_special_tokens=False)["input_ids"]) == 1
    for d in "012":
        assert len(tok(d, add_special_tokens=False)["input_ids"]) == 1

    def encode_row(kind, state, translation, lang, completion):
        if kind == "tier":
            q = q_tier
        else:
            q = q_faithful
        prompt = render(tokenizer=tok, state=state_of(state, translation, lang), q=q,
                        bos=tok.bos_token)
        ids = tok(prompt, truncation=True, max_length=MAXLEN, add_special_tokens=False)["input_ids"]
        comp = tok(completion, add_special_tokens=False)["input_ids"]
        return ids + comp, [-100] * len(ids) + comp

    t0 = time.perf_counter()
    base = AutoModel.from_pretrained(MODEL_ID, trust_remote_code=True,
                                     dtype=torch.bfloat16, attn_implementation="sdpa").to("xpu")
    print(f"loaded {time.perf_counter()-t0:.0f}s", flush=True)

    ckpts = sorted(OUT.glob("ckpt_*"), key=lambda p: int(p.name.split("_")[1]))
    start_step = 0
    if ckpts:
        ck = ckpts[-1]
        model = PeftModel.from_pretrained(base, ck / "adapter", is_trainable=True)
        meta = json.loads((ck / "meta.json").read_text())
        start_step = meta["step"]
        print(f"RESUME {ck.name}: step {start_step}", flush=True)
    else:
        targets = [n for n, m in base.named_modules()
                   if n.split(".")[-1] in ("q_proj", "k_proj", "v_proj", "out_proj", "in_proj",
                                           "w1", "w2", "w3")
                   and type(m).__name__ == "Linear"
                   and getattr(m, "weight", None) is not None and "vision" not in n]
        print(f"lora targets: {len(targets)}", flush=True)
        model = get_peft_model(base, LoraConfig(
            r=16, lora_alpha=32, lora_dropout=0.05, bias="none",
            task_type="CAUSAL_LM", target_modules=targets))
    model.train()
    model.enable_input_require_grads()
    model.print_trainable_parameters()

    opt = torch.optim.AdamW([p for p in model.parameters() if p.requires_grad],
                            lr=LR, weight_decay=0.01)
    if ckpts:
        opt.load_state_dict(torch.load(ckpts[-1] / "opt.pt", map_location="xpu", weights_only=True))
        torch.xpu.empty_cache()

    def masked_ce(ids, labels, attn):
        out = model(input_ids=ids, attention_mask=attn, use_cache=False)
        logits = out.logits if hasattr(out, "logits") else out[0]
        sl = labels[:, 1:]
        m = sl != -100
        return F.cross_entropy(logits[:, :-1, :][m].float(), sl[m])

    encoded = [encode_row(*r) for r in train_rows]
    print(f"pretokenize done: {len(encoded)}", flush=True)
    val_encoded = [encode_row(*r) for r in val_rows]

    order = sorted(range(len(encoded)), key=lambda i: len(encoded[i][0]))
    batch_idx = [order[s:s + BS] for s in range(0, len(order), BS)]
    random.Random(12345).shuffle(batch_idx)
    skip_batches = 0
    if ckpts:
        skip_batches = json.loads((ckpts[-1] / "meta.json").read_text())["batches"]
        print(f"RESUME skip {skip_batches} batches", flush=True)

    steps_total = math.ceil(len(batch_idx) / ACCUM) * EPOCHS
    warm = int(steps_total * WARMUP)

    def lr_at(s):
        if s < warm:
            return LR * s / max(1, warm)
        p = (s - warm) / max(1, steps_total - warm)
        return LR * 0.5 * (1 + math.cos(math.pi * min(p, 1.0)))

    def run_val():
        model.eval()
        tot, n = 0.0, 0
        with torch.no_grad():
            for s in range(0, len(val_encoded), BS):
                enc = val_encoded[s:s + BS]
                ml = max(len(ids) for ids, _ in enc)
                ii = [ids + [tok.pad_token_id] * (ml - len(ids)) for ids, _ in enc]
                ll = [lab + [-100] * (ml - len(lab)) for _, lab in enc]
                aa = [[1] * len(ids) + [0] * (ml - len(ids)) for ids, _ in enc]
                loss = masked_ce(torch.tensor(ii).to("xpu"), torch.tensor(ll).to("xpu"),
                                 torch.tensor(aa).to("xpu"))
                tot += loss.item() * len(ii)
                n += len(ii)
        model.train()
        torch.xpu.empty_cache()
        return tot / n

    step, t0 = start_step, time.perf_counter()
    opt.zero_grad(set_to_none=True)
    micro = 0
    for bi, idxs in enumerate(batch_idx[skip_batches:], start=skip_batches):
        enc = [encoded[i] for i in idxs]
        ml = max(len(ids) for ids, _ in enc)
        ii = [ids + [tok.pad_token_id] * (ml - len(ids)) for ids, _ in enc]
        ll = [lab + [-100] * (ml - len(lab)) for _, lab in enc]
        aa = [[1] * len(ids) + [0] * (ml - len(ids)) for ids, _ in enc]
        loss = masked_ce(torch.tensor(ii).to("xpu"), torch.tensor(ll).to("xpu"),
                         torch.tensor(aa).to("xpu"))
        (loss / ACCUM).backward()
        micro += 1
        batches_done = bi + 1
        if micro == ACCUM or bi + 1 >= len(batch_idx):
            for g in opt.param_groups:
                g["lr"] = lr_at(step)
            opt.step()
            opt.zero_grad(set_to_none=True)
            micro = 0
            step += 1
            if step % 20 == 0:
                torch.xpu.empty_cache()
            if step % 100 == 0:
                done = batches_done / len(batch_idx)
                eta = (time.perf_counter() - t0) / max(1, step - start_step) * (steps_total - step)
                print(f"step {step}/{steps_total} ({done:.0%}) loss {loss.item():.4f} "
                      f"eta {eta/60:.0f}m", flush=True)
            if step % CKPT_EVERY == 0 or step >= steps_total:
                ck = OUT / f"ckpt_{step:05d}"
                (ck / "adapter").mkdir(parents=True, exist_ok=True)
                model.save_pretrained(ck / "adapter")
                torch.save(opt.state_dict(), ck / "opt.pt")
                (ck / "meta.json").write_text(json.dumps({"step": step, "batches": batches_done}))
                torch.xpu.empty_cache()
                print(f"checkpoint {ck.name} (val_loss {run_val():.4f})", flush=True)

    print(f"FINAL val_loss {run_val():.4f}", flush=True)
    (OUT / "adapter").mkdir(parents=True, exist_ok=True)
    model.save_pretrained(OUT / "adapter")
    print("saved final adapter", flush=True)


if __name__ == "__main__":
    main()
