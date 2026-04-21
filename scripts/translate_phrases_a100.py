#!/usr/bin/env python3
"""Translate 197k Japanese phrases to Russian and English on NVIDIA A100 GPU.

Uses facebook/nllb-200-distilled-600M with torch.compile(), bf16,
large batch sizes, and DataLoader for maximum throughput.

Usage:
    python scripts/translate_phrases_a100.py
    python scripts/translate_phrases_a100.py --batch-size 128 --dtype bf16
    python scripts/translate_phrases_a100.py --skip-existing
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

import torch
from torch.utils.data import DataLoader, Dataset
from tqdm import tqdm
from transformers import AutoModelForSeq2SeqLM, AutoTokenizer

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
DEFAULT_INPUT = PROJECT_ROOT / "phrase_dataset" / "phrase_dataset.json"

MODEL_NAME = "facebook/nllb-200-distilled-600M"
LANG_JA = "jpn_Jpan"
LANG_RU = "rus_Cyrl"
LANG_EN = "eng_Latn"

DTYPE_MAP = {
    "fp16": torch.float16,
    "bf16": torch.bfloat16,
    "fp32": torch.float32,
}


class PhraseDataset(Dataset):
    """Simple dataset wrapper for phrase indices and texts."""

    def __init__(self, indices: list[int], phrases: list[dict]) -> None:
        self.items = [(i, phrases[i]["text"]) for i in indices]

    def __len__(self) -> int:
        return len(self.items)

    def __getitem__(self, idx: int) -> tuple[int, str]:
        return self.items[idx]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Translate Japanese phrases to RU/EN on A100 GPU"
    )
    parser.add_argument("--input", type=Path, default=DEFAULT_INPUT)
    parser.add_argument("--output", type=Path, default=None)
    parser.add_argument("--batch-size", type=int, default=128)
    parser.add_argument("--skip-existing", action="store_true")
    parser.add_argument("--model", type=str, default=MODEL_NAME)
    parser.add_argument(
        "--dtype",
        choices=["fp16", "bf16", "fp32"],
        default="bf16",
    )
    parser.add_argument("--no-compile", action="store_true")
    parser.add_argument("--save-every", type=int, default=50,
        help="Save checkpoint every N batches (default: 50)")
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


def filter_phrases(phrases: list[dict], skip_existing: bool) -> list[int]:
    if not skip_existing:
        return list(range(len(phrases)))
    return [
        i
        for i, p in enumerate(phrases)
        if not (p.get("translation_ru") and p.get("translation_en"))
    ]


def load_model(
    model_name: str, device: torch.device, dtype: torch.dtype, no_compile: bool
) -> tuple[AutoTokenizer, AutoModelForSeq2SeqLM]:
    print(f"Loading model: {model_name} [{dtype}] ...")
    t0 = time.monotonic()

    tokenizer = AutoTokenizer.from_pretrained(model_name)
    model = AutoModelForSeq2SeqLM.from_pretrained(
        model_name,
        torch_dtype=dtype,
        use_safetensors=True,
    ).to(device)
    model.eval()

    if not no_compile:
        print("Compiling model with torch.compile() ...")
        model = torch.compile(model)

    print(f"Model ready in {time.monotonic() - t0:.1f}s")
    return tokenizer, model


def collate_fn(batch: list[tuple[int, str]]) -> tuple[list[int], list[str]]:
    indices, texts = zip(*batch)
    return list(indices), list(texts)


def translate_batch(
    texts: list[str],
    tokenizer: AutoTokenizer,
    model: AutoModelForSeq2SeqLM,
    target_lang: str,
    device: torch.device,
) -> list[str]:
    if not texts:
        return []

    tokenizer.src_lang = LANG_JA
    inputs = tokenizer(
        texts, return_tensors="pt", padding=True, truncation=True, max_length=256
    )
    inputs = {k: v.to(device) for k, v in inputs.items()}
    forced_bos_token_id = tokenizer.convert_tokens_to_ids(target_lang)

    with torch.no_grad():
        generated = model.generate(
            **inputs,
            forced_bos_token_id=forced_bos_token_id,
            max_length=256,
        )

    results: list[str] = []
    for i, output in enumerate(generated):
        decoded = tokenizer.decode(output, skip_special_tokens=True).strip()
        if not decoded or decoded == texts[i].strip():
            decoded = ""
        results.append(decoded)

    return results


def translate_pass(
    dataloader: DataLoader,
    phrases: list[dict],
    tokenizer: AutoTokenizer,
    model: AutoModelForSeq2SeqLM,
    target_lang: str,
    field: str,
    device: torch.device,
    initial_batch_size: int,
    dataset: dict,
    output_path: Path,
    save_every: int,
) -> tuple[int, int]:
    """Run one translation pass (JA→target_lang). Returns (translated, errors)."""
    translated = 0
    errors = 0
    batch_count = 0

    desc = f"JA→{'RU' if target_lang == LANG_RU else 'EN'}"
    pbar = tqdm(dataloader, desc=desc, unit="batch")

    for batch_indices, batch_texts in pbar:
        try:
            translations = _translate_with_oom_fallback(
                batch_texts, tokenizer, model, target_lang, device
            )
        except Exception as exc:
            print(f"\n  Batch error: {exc}", file=sys.stderr)
            errors += len(batch_indices)
            for idx in batch_indices:
                phrases[idx].setdefault(field, "")
            pbar.set_postfix(translated=translated, errors=errors)
            continue

        for j, idx in enumerate(batch_indices):
            value = translations[j] if j < len(translations) else ""
            phrases[idx][field] = value
            if not value:
                errors += 1
            translated += 1

        gpu_mb = torch.cuda.max_memory_allocated() / 1e9
        pbar.set_postfix(translated=translated, errors=errors, gpu=f"{gpu_mb:.1f}GB")

        batch_count += 1
        if save_every > 0 and batch_count % save_every == 0:
            save_dataset(dataset, output_path)
            pbar.write(f"  Checkpoint saved (batch {batch_count})")

    pbar.close()
    return translated, errors


def _translate_with_oom_fallback(
    texts: list[str],
    tokenizer: AutoTokenizer,
    model: AutoModelForSeq2SeqLM,
    target_lang: str,
    device: torch.device,
    max_retries: int = 3,
) -> list[str]:
    """Try full batch first; on OOM, halve batch size and retry."""
    for attempt in range(max_retries):
        try:
            return translate_batch(texts, tokenizer, model, target_lang, device)
        except RuntimeError as exc:
            if "out of memory" not in str(exc).lower():
                raise
            torch.cuda.empty_cache()
            if len(texts) <= 1:
                return [""] * len(texts)
            half = len(texts) // 2
            left = _translate_with_oom_fallback(
                texts[:half], tokenizer, model, target_lang, device, max_retries - attempt
            )
            right = _translate_with_oom_fallback(
                texts[half:], tokenizer, model, target_lang, device, max_retries - attempt
            )
            return left + right
    return [""] * len(texts)


def print_report(
    total: int, translated: int, skipped: int, errors: int,
    elapsed: float, peak_gpu_gb: float,
) -> None:
    rate = translated / elapsed * 60 if elapsed > 0 and translated > 0 else 0
    print()
    print("=" * 60)
    print(f"  Total:      {total:,}")
    print(f"  Translated: {translated:,}")
    print(f"  Skipped:    {skipped:,}")
    print(f"  Errors:     {errors:,}")
    print(f"  Time:       {elapsed:.1f}s")
    print(f"  Rate:       {rate:,.0f} phrases/min")
    print(f"  GPU Memory: {peak_gpu_gb:.1f} GB peak")
    print("=" * 60)


def save_dataset(dataset: dict, path: Path) -> None:
    tmp_path = path.with_suffix(".json.tmp")
    with tmp_path.open("w", encoding="utf-8") as f:
        json.dump(dataset, f, ensure_ascii=False, indent=2)
        f.write("\n")
    tmp_path.replace(path)


def main() -> None:
    args = parse_args()
    output_path: Path = args.output or args.input
    dtype = DTYPE_MAP[args.dtype]
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")

    if device.type == "cuda":
        print(f"Device: {torch.cuda.get_device_name(0)}")
    else:
        print(f"Device: {device} (WARNING: no CUDA)")

    dataset = load_dataset(args.input)
    phrases: list[dict] = dataset["phrases"]
    print(f"Loaded {len(phrases):,} phrases from {args.input}")

    indices = filter_phrases(phrases, args.skip_existing)
    total = len(indices)
    skipped = len(phrases) - total
    print(f"Phrases to translate: {total:,} (skipped: {skipped:,})")

    if not indices:
        print("Nothing to translate. Exiting.")
        return

    tokenizer, model = load_model(args.model, device, dtype, args.no_compile)

    torch.cuda.reset_peak_memory_stats()
    t_start = time.monotonic()

    phrase_dataset = PhraseDataset(indices, phrases)
    dataloader = DataLoader(
        phrase_dataset,
        batch_size=args.batch_size,
        shuffle=False,
        num_workers=0,
        collate_fn=collate_fn,
    )

    total_translated = 0
    total_errors = 0

    for target_lang, field in [(LANG_RU, "translation_ru"), (LANG_EN, "translation_en")]:
        translated, errors = translate_pass(
            dataloader, phrases, tokenizer, model,
            target_lang, field, device, args.batch_size,
            dataset, output_path, args.save_every,
        )
        total_translated += translated
        total_errors += errors

        save_dataset(dataset, output_path)
        lang_name = "RU" if target_lang == LANG_RU else "EN"
        print(f"Pass {lang_name} complete. Saved to {output_path}")

    elapsed = time.monotonic() - t_start
    peak_gpu = torch.cuda.max_memory_allocated() / 1e9

    print_report(total, total_translated // 2, skipped, total_errors, elapsed, peak_gpu)
    save_dataset(dataset, output_path)
    print(f"Saved to: {output_path}")


if __name__ == "__main__":
    main()
