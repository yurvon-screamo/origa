"""Translate phrases from Japanese to Russian and English using NLLB-200.

Uses facebook/nllb-200-distilled-600M for batch translation.
Adds translation_ru and translation_en fields to phrase_dataset.json.

Usage:
    python scripts/translate_phrases.py
    python scripts/translate_phrases.py --skip-existing --batch-size 16
    python scripts/translate_phrases.py --input data.json --output translated.json
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path

import torch
from transformers import AutoModelForSeq2SeqLM, AutoTokenizer

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
DEFAULT_INPUT = PROJECT_ROOT / "origa_ui" / "public" / "phrase" / "phrase_dataset.json"

MODEL_NAME = "facebook/nllb-200-distilled-600M"
LANG_JA = "jpn_Jpan"
LANG_RU = "rus_Cyrl"
LANG_EN = "eng_Latn"


def _is_directml_available() -> bool:
    try:
        import torch_directml  # noqa: F401

        return True
    except ImportError:
        return False


def detect_device() -> torch.device:
    """Detect best available compute device: CUDA > DML > XPU > MPS > CPU."""
    if torch.cuda.is_available():
        device = torch.device("cuda")
        print(f"Device: {device} ({torch.cuda.get_device_name(0)})")
    elif _is_directml_available():
        import torch_directml

        device = torch_directml.device()
        print(f"Device: DML ({torch_directml.device_name(device)})")
    elif hasattr(torch, "xpu") and torch.xpu.is_available():
        device = torch.device("xpu")
        print(f"Device: {device} ({torch.xpu.get_device_name(0)})")
    elif hasattr(torch.backends, "mps") and torch.backends.mps.is_available():
        device = torch.device("mps")
        print(f"Device: {device} (Apple Metal)")
    else:
        device = torch.device("cpu")
        print(f"Device: {device}")
    return device


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Translate phrases from Japanese to Russian and English using NLLB-200"
    )
    parser.add_argument(
        "--input",
        type=Path,
        default=DEFAULT_INPUT,
        help="Path to phrase_dataset.json",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Output path (default: overwrite input)",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=8,
        help="Batch size for inference (default: 8)",
    )
    parser.add_argument(
        "--skip-existing",
        action="store_true",
        help="Skip phrases that already have both translations",
    )
    parser.add_argument(
        "--device",
        type=str,
        default=None,
        help="Force device: cuda, dml, xpu, mps, cpu (default: auto-detect)",
    )
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


def needs_translation(phrase: dict, skip_existing: bool) -> bool:
    if not skip_existing:
        return True
    return not (phrase.get("translation_ru") and phrase.get("translation_en"))


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
    inputs = tokenizer(texts, return_tensors="pt", padding=True, truncation=True, max_length=512)
    inputs = {k: v.to(device) for k, v in inputs.items()}

    forced_bos_token_id = tokenizer.convert_tokens_to_ids(target_lang)

    with torch.no_grad():
        generated = model.generate(
            **inputs,
            forced_bos_token_id=forced_bos_token_id,
            max_length=512,
        )

    results: list[str] = []
    for i, output in enumerate(generated):
        decoded = tokenizer.decode(output, skip_special_tokens=True).strip()
        if not decoded or decoded == texts[i].strip():
            decoded = ""
        results.append(decoded)

    return results


def main() -> None:
    args = parse_args()
    output_path: Path = args.output or args.input

    dataset = load_dataset(args.input)
    phrases: list[dict] = dataset["phrases"]
    print(f"Loaded {len(phrases)} phrases from {args.input}")

    indices_to_translate = [
        i for i, p in enumerate(phrases) if needs_translation(p, args.skip_existing)
    ]
    print(f"Phrases to translate: {len(indices_to_translate)}")

    if not indices_to_translate:
        print("Nothing to translate. Exiting.")
        return

    if args.device:
        if args.device == "dml":
            import torch_directml

            device = torch_directml.device()
            print("Device: DML (forced)")
        else:
            device = torch.device(args.device)
            print(f"Device: {device} (forced)")
    else:
        device = detect_device()

    print(f"Loading model: {MODEL_NAME} ...")
    t0 = time.monotonic()
    tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME, use_safetensors=True)
    try:
        model = AutoModelForSeq2SeqLM.from_pretrained(MODEL_NAME, use_safetensors=True).to(device)
    except Exception:
        model = AutoModelForSeq2SeqLM.from_pretrained(MODEL_NAME).to(device)
    model.eval()
    print(f"Model loaded in {time.monotonic() - t0:.1f}s")

    batch_size: int = args.batch_size
    total = len(indices_to_translate)
    translated = 0
    errors = 0
    t_start = time.monotonic()

    try:
        from tqdm import tqdm

        pbar = tqdm(total=total, unit="phrase", desc="Translating")
    except ImportError:
        pbar = None

    for batch_start in range(0, total, batch_size):
        batch_indices = indices_to_translate[batch_start : batch_start + batch_size]
        batch_texts = [phrases[i]["text"] for i in batch_indices]

        try:
            ru_translations = translate_batch(batch_texts, tokenizer, model, LANG_RU, device)
            en_translations = translate_batch(batch_texts, tokenizer, model, LANG_EN, device)
        except Exception as exc:
            print(f"\n  Batch error at {batch_start}: {exc}", file=sys.stderr)
            errors += len(batch_indices)
            for idx in batch_indices:
                phrases[idx].setdefault("translation_ru", "")
                phrases[idx].setdefault("translation_en", "")
            if pbar is not None:
                pbar.update(len(batch_indices))
            continue

        for j, idx in enumerate(batch_indices):
            ru = ru_translations[j] if j < len(ru_translations) else ""
            en = en_translations[j] if j < len(en_translations) else ""
            phrases[idx]["translation_ru"] = ru
            phrases[idx]["translation_en"] = en
            if not ru and not en:
                errors += 1
            translated += 1

        if pbar is not None:
            pbar.update(len(batch_indices))
        else:
            elapsed = time.monotonic() - t_start
            done = min(batch_start + batch_size, total)
            rate = done / elapsed * 60 if elapsed > 0 else 0
            eta = (total - done) / rate * 60 if rate > 0 else 0
            print(
                f"  [{done}/{total}] "
                f"rate={rate:.1f} phrases/min "
                f"ETA={eta:.0f}s"
            )

    if pbar is not None:
        pbar.close()

    elapsed = time.monotonic() - t_start
    rate = translated / elapsed * 60 if elapsed > 0 and translated > 0 else 0

    print()
    print("=" * 60)
    print(f"  Total:      {total}")
    print(f"  Translated: {translated}")
    print(f"  Errors:     {errors}")
    print(f"  Time:       {elapsed:.1f}s")
    print(f"  Rate:       {rate:.1f} phrases/min")
    print("=" * 60)

    with output_path.open("w", encoding="utf-8") as f:
        json.dump(dataset, f, ensure_ascii=False, indent=2)
        f.write("\n")

    print(f"Saved to: {output_path}")


if __name__ == "__main__":
    main()
