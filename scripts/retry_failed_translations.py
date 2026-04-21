#!/usr/bin/env python3
"""Retry failed translations for phrases with empty/missing translation_ru or translation_en.

Handles edge cases that caused failures in batch mode:
  - Processes phrases one-by-one (batch_size=1) to avoid padding issues
  - Uses max_length=512 to avoid truncation of longer phrases
  - Retries with a forced prefix if the model echoes the source text

Usage:
    python retry_failed_translations.py --input phrase_dataset.json
    python retry_failed_translations.py --input phrase_dataset.json --dry-run
    python retry_failed_translations.py --input phrase_dataset.json --max-retries 3
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

import torch
from transformers import AutoModelForSeq2SeqLM, AutoTokenizer

MODEL_NAME = "facebook/nllb-200-distilled-600M"
LANG_JA = "jpn_Jpan"
LANG_RU = "rus_Cyrl"
LANG_EN = "eng_Latn"

PREFIX_MAP = {
    LANG_RU: "Translate to Russian: ",
    LANG_EN: "Translate to English: ",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Retry failed translations (empty RU/EN) one-by-one with extended limits"
    )
    parser.add_argument("--input", type=Path, required=True, help="Path to phrase_dataset.json")
    parser.add_argument("--output", type=Path, default=None, help="Output path (default: overwrite input)")
    parser.add_argument("--model", type=str, default=MODEL_NAME)
    parser.add_argument("--max-length", type=int, default=512)
    parser.add_argument("--max-retries", type=int, default=2)
    parser.add_argument("--dry-run", action="store_true", help="Only print stats, no translation")
    return parser.parse_args()


def load_dataset(path: Path) -> dict:
    if not path.exists():
        print(f"ERROR: File not found: {path}", file=sys.stderr)
        sys.exit(1)

    with path.open("r", encoding="utf-8") as f:
        data = json.load(f)

    if not isinstance(data, dict) or "phrases" not in data:
        print("ERROR: Expected JSON with 'phrases' key", file=sys.stderr)
        sys.exit(1)

    return data


def save_dataset(dataset: dict, path: Path) -> None:
    tmp_path = path.with_suffix(".json.tmp")
    with tmp_path.open("w", encoding="utf-8") as f:
        json.dump(dataset, f, ensure_ascii=False, indent=2)
        f.write("\n")
    tmp_path.replace(path)


def find_failed_phrases(phrases: list[dict]) -> list[tuple[int, list[str]]]:
    """Return (index, missing_fields) for phrases with empty translations."""
    failed: list[tuple[int, list[str]]] = []
    for i, p in enumerate(phrases):
        missing: list[str] = []
        ru = p.get("translation_ru")
        if not ru or not ru.strip():
            missing.append("translation_ru")
        en = p.get("translation_en")
        if not en or not en.strip():
            missing.append("translation_en")
        if missing:
            failed.append((i, missing))
    return failed


def translate_single(
    text: str,
    target_lang: str,
    tokenizer: AutoTokenizer,
    model: AutoModelForSeq2SeqLM,
    device: torch.device,
    max_length: int,
    use_prefix: bool = False,
) -> str:
    src = (PREFIX_MAP.get(target_lang, "") + text) if use_prefix else text
    tokenizer.src_lang = LANG_JA
    inputs = tokenizer(src, return_tensors="pt", truncation=True, max_length=max_length)
    inputs = {k: v.to(device) for k, v in inputs.items()}
    forced_bos_token_id = tokenizer.convert_tokens_to_ids(target_lang)

    with torch.no_grad():
        generated = model.generate(
            **inputs,
            forced_bos_token_id=forced_bos_token_id,
            max_length=max_length,
        )

    decoded = tokenizer.decode(generated[0], skip_special_tokens=True).strip()
    if use_prefix and decoded.lower().startswith("translate to"):
        decoded = decoded.split(":", 1)[-1].strip()
    return decoded


def try_translate(
    text: str,
    target_lang: str,
    field: str,
    tokenizer: AutoTokenizer,
    model: AutoModelForSeq2SeqLM,
    device: torch.device,
    max_length: int,
    max_retries: int,
) -> str:
    for attempt in range(max_retries):
        try:
            use_prefix = attempt > 0
            decoded = translate_single(
                text, target_lang, tokenizer, model, device, max_length, use_prefix=use_prefix,
            )
            if decoded and decoded != text.strip():
                return decoded
        except Exception as exc:
            print(f"    Attempt {attempt + 1} error ({field}): {exc}", file=sys.stderr)

    return ""


def main() -> None:
    args = parse_args()
    output_path: Path = args.output or args.input

    dataset = load_dataset(args.input)
    phrases: list[dict] = dataset["phrases"]
    print(f"Loaded {len(phrases):,} phrases from {args.input}")

    failed = find_failed_phrases(phrases)
    print(f"Phrases with missing translations: {len(failed):,}")

    if not failed:
        print("All phrases translated. Nothing to do.")
        return

    print("\nFirst 10 examples:")
    for idx, missing in failed[:10]:
        p = phrases[idx]
        ru = p.get("translation_ru", "") or "(empty)"
        en = p.get("translation_en", "") or "(empty)"
        print(f"  [{idx}] text={p['text'][:60]!r}  ru={ru[:40]!r}  en={en[:40]!r}  missing={missing}")

    if args.dry_run:
        print("\n--dry-run: stopping here.")
        return

    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    if device.type == "cuda":
        print(f"\nDevice: {torch.cuda.get_device_name(0)}")
    else:
        print(f"\nDevice: {device} (no CUDA)")

    print(f"Loading model: {args.model} ...")
    t0 = time.monotonic()
    tokenizer = AutoTokenizer.from_pretrained(args.model)
    model = AutoModelForSeq2SeqLM.from_pretrained(
        args.model, torch_dtype=torch.float32, use_safetensors=True,
    ).to(device)
    model.eval()
    print(f"Model ready in {time.monotonic() - t0:.1f}s")

    torch.cuda.reset_peak_memory_stats()
    t_start = time.monotonic()

    total = len(failed)
    translated = 0
    still_failed = 0

    for count, (idx, missing) in enumerate(failed, 1):
        p = phrases[idx]
        text = p["text"]
        any_change = False

        for target_lang, field in [(LANG_RU, "translation_ru"), (LANG_EN, "translation_en")]:
            if field not in missing:
                continue

            result = try_translate(
                text, target_lang, field, tokenizer, model, device,
                args.max_length, args.max_retries,
            )
            p[field] = result
            any_change = any_change or bool(result)

        if any_change:
            translated += 1
        else:
            still_failed += 1

        if count % 10 == 0 or count == total:
            elapsed = time.monotonic() - t_start
            rate = count / elapsed * 60 if elapsed > 0 else 0
            gpu_mb = torch.cuda.max_memory_allocated() / 1e9 if device.type == "cuda" else 0
            print(
                f"  [{count}/{total}] "
                f"translated={translated} still_failed={still_failed} "
                f"rate={rate:.0f}/min gpu={gpu_mb:.1f}GB"
            )

    elapsed = time.monotonic() - t_start
    peak_gpu = torch.cuda.max_memory_allocated() / 1e9 if device.type == "cuda" else 0

    print()
    print("=" * 60)
    print(f"  Total processed:  {total}")
    print(f"  Fixed:            {translated}")
    print(f"  Still failed:     {still_failed}")
    print(f"  Time:             {elapsed:.1f}s")
    print(f"  GPU Memory:       {peak_gpu:.1f} GB peak")
    print("=" * 60)

    save_dataset(dataset, output_path)
    print(f"Saved to: {output_path}")

    if still_failed > 0:
        print(f"\nRemaining {still_failed} phrases that still failed:")
        shown = 0
        for idx, missing in failed:
            p = phrases[idx]
            if not p.get("translation_ru") or not p.get("translation_en"):
                print(
                    f"  [{idx}] text={p['text'][:80]!r}  "
                    f"ru={p.get('translation_ru', '')[:40]!r}  "
                    f"en={p.get('translation_en', '')[:40]!r}"
                )
                shown += 1
                if shown >= 20:
                    break
        print(f"  ... ({still_failed} total)")


if __name__ == "__main__":
    main()
