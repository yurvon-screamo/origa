#!/usr/bin/env python3
"""Eval wrapper: single-mode generation + language routing check + retry.

No chrF — output is built for MANUAL review (gens.jsonl with all attempts).

Flow per phrase:
  1. generate with EN-instruction (trained format)
  2. detect output language by script/diacritics
  3. if detected != target -> retry with JA-instruction
  4. log every attempt, mark final verdict

Works with any base (bf16 HF or QAT w4a16-ct compressed-tensors) + LoRA adapter.

Usage:
  eval_wrapper.py --base google/gemma-4-E2B-it-qat-w4a16-ct \
      --adapter runs/gemma-scale/40k/ep1 --name qat-40k --data data/test_20x4.jsonl
"""

from __future__ import annotations

import argparse
import json
import os
import re
import time
from pathlib import Path

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import PeftModel

ROOT = Path(os.environ.get("ORIGA_WORK_DIR", "."))
SYSTEM_EN = "You are a skilled Japanese translator. Translate the sentence accurately and naturally."
SYSTEM_JA = "あなたは熟練した日本語翻訳者です。文を正確かつ自然に翻訳してください。"
INSTR_EN = "Translate the following Japanese sentence into {lname}:\n{x}"
INSTR_JA = {
    "en": "次の日本語の文を英語に翻訳してください:\n{x}",
    "vi": "次の日本語の文をベトナム語に翻訳してください:\n{x}",
    "ru": "次の日本語の文をロシア語に翻訳してください:\n{x}",
    "ko": "次の日本語の文を韓国語に翻訳してください:\n{x}",
}
LNAME = {"en": "English", "vi": "Vietnamese", "ru": "Russian", "ko": "Korean"}

RE_HANGUL = re.compile(r"[가-힣]")
RE_CYR = re.compile(r"[А-ЯЁа-яё]")
RE_KANA = re.compile(r"[ひ-んア-ンー]")
RE_CJK = re.compile(r"[一-龥]")
RE_LATIN = re.compile(r"[A-Za-z]")
# Vietnamese-specific: diacritics beyond plain ASCII latin
RE_VI_MARK = re.compile(
    r"[ăâêôơưđĂÂÊÔƠƯĐ]"
    r"|[áàảãạấầẩẫậắằẳẵặéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ]"
)


def detect_language(text: str) -> str:
    """Script-based language guess: en / vi / ru / ko / ja / other."""
    t = text.strip()
    if not t:
        return "empty"
    n = max(len(t), 1)
    counts = {
        "hangul": len(RE_HANGUL.findall(t)),
        "cyr": len(RE_CYR.findall(t)),
        "kana": len(RE_KANA.findall(t)),
        "cjk": len(RE_CJK.findall(t)),
        "latin": len(RE_LATIN.findall(t)),
    }
    vi_marks = len(RE_VI_MARK.findall(t))
    words = [w for w in re.split(r"\s+", t) if w]
    vi_word_share = (
        (sum(1 for w in words if RE_VI_MARK.search(w)) / len(words)) if words else 0
    )

    if counts["hangul"] / n > 0.2:
        return "ko"
    if counts["kana"] / n > 0.15 or counts["cjk"] / n > 0.3:
        return "ja"
    if counts["cyr"] / n > 0.4:
        return "ru"
    if counts["latin"] / n > 0.4:
        if vi_word_share > 0.3 or vi_marks > 4:
            return "vi"
        return "en"
    return "other"


def build_prompt(tok, lang: str, x: str, fmt: str) -> str:
    if fmt == "en":
        msgs = [
            {"role": "system", "content": SYSTEM_EN},
            {"role": "user", "content": INSTR_EN.format(lname=LNAME[lang], x=x)},
        ]
    else:
        msgs = [
            {"role": "system", "content": SYSTEM_JA},
            {"role": "user", "content": INSTR_JA[lang].format(x=x)},
        ]
    return tok.apply_chat_template(
        msgs, tokenize=False, add_generation_prompt=True, enable_thinking=False
    )


def generate_one(model, tok, prompt: str, max_new_tokens: int = 96) -> str:
    enc = tok(prompt, return_tensors="pt").to("xpu")
    with torch.no_grad():
        out = model.generate(
            **enc,
            max_new_tokens=max_new_tokens,
            do_sample=False,
            pad_token_id=tok.pad_token_id
            if tok.pad_token_id is not None
            else tok.eos_token_id,
        )
    return tok.decode(
        out[0][enc["input_ids"].shape[1] :], skip_special_tokens=True
    ).strip()


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--base", required=True, help="HF repo id or local path (bf16 or QAT w4a16-ct)"
    )
    ap.add_argument("--adapter", default="", help="optional LoRA adapter dir")
    ap.add_argument("--name", required=True, help="run name -> eval/<name>/")
    ap.add_argument("--data", default=str(ROOT / "data" / "test_20x4.jsonl"))
    ap.add_argument("--max-new", type=int, default=96)
    a = ap.parse_args()

    rows = [json.loads(l) for l in open(a.data, encoding="utf-8")]
    out_dir = ROOT / "eval" / a.name
    out_dir.mkdir(parents=True, exist_ok=True)

    t0 = time.time()
    print(f"loading {a.base} ...", flush=True)
    tok = AutoTokenizer.from_pretrained(a.base)
    if tok.pad_token is None:
        tok.pad_token = tok.eos_token
    load_kw = {"device_map": {"": "xpu"}, "dtype": torch.bfloat16}
    if "qat" in a.base.lower():
        try:
            from transformers import CompressedTensorsConfig
        except ImportError:
            from compressed_tensors.config import CompressedTensorsConfig
        load_kw["low_cpu_mem_usage"] = False
        load_kw["quantization_config"] = CompressedTensorsConfig(run_compressed=False)
        print(
            "QAT mode: dequantize-on-load (QAT weight values -> bf16 Linear)",
            flush=True,
        )
    try:
        model = AutoModelForCausalLM.from_pretrained(a.base, **load_kw)
    except Exception as e:  # noqa: BLE001 - report load failure clearly
        print(f"LOAD FAILED on xpu: {type(e).__name__}: {e}", flush=True)
        raise SystemExit(2)
    if a.adapter:
        model = PeftModel.from_pretrained(model, a.adapter)
    model.eval()
    print(
        f"loaded in {time.time() - t0:.0f}s, adapter={a.adapter or 'none'}", flush=True
    )

    stats = {"first_try_ok": 0, "retry_ok": 0, "both_failed": 0, "by_lang": {}}
    results = []
    for n, r in enumerate(rows, 1):
        lang = r["lang"]
        attempts = []
        detected = ""
        gen = ""
        for fmt in ("en", "ja"):
            prompt = build_prompt(tok, lang, r["x"], fmt)
            gen = generate_one(model, tok, prompt, a.max_new)
            detected = detect_language(gen)
            attempts.append({"fmt": fmt, "gen": gen, "detected": detected})
            if detected == lang:
                break
        verdict = (
            "first_try"
            if attempts[0]["detected"] == lang
            else ("retry_ok" if detected == lang else "routing_failed")
        )
        stats_key = {
            "first_try": "first_try_ok",
            "retry_ok": "retry_ok",
            "routing_failed": "both_failed",
        }[verdict]
        stats[stats_key] += 1
        stats["by_lang"].setdefault(lang, {"first_try": 0, "retry_ok": 0, "failed": 0})
        lang_st = stats["by_lang"][lang]
        if verdict == "first_try":
            lang_st["first_try"] += 1
        elif verdict == "retry_ok":
            lang_st["retry_ok"] += 1
        else:
            lang_st["failed"] += 1
        results.append(
            {
                "i": r["i"],
                "lang": lang,
                "x": r["x"],
                "gold": r["gold"],
                "attempts": attempts,
                "verdict": verdict,
                "final_detected": detected,
            }
        )
        print(f"  {n}/{len(rows)} [{lang}] {verdict} (detected={detected})", flush=True)

    with open(out_dir / "gens.jsonl", "w", encoding="utf-8") as f:
        for r in results:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    with open(out_dir / "routing.json", "w", encoding="utf-8") as f:
        json.dump(stats, f, ensure_ascii=False, indent=2)

    print("\n=== ROUTING SUMMARY (no chrF by design — manual review only) ===")
    print(f"first try ok : {stats['first_try_ok']}/{len(rows)}")
    print(f"retry fixed  : {stats['retry_ok']}/{len(rows)}")
    print(f"both failed  : {stats['both_failed']}/{len(rows)}")
    for lang, st in stats["by_lang"].items():
        print(
            f"  {lang}: first={st['first_try']} retry={st['retry_ok']} failed={st['failed']}"
        )
    print(f"\ngens -> {out_dir / 'gens.jsonl'}")


if __name__ == "__main__":
    main()
