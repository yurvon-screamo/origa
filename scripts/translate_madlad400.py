"""
Batch translation of Japanese phrases using google/madlad400-7b-mt-bt.

Processes JSON chunk files (p0000.json — p0197.json), regenerates EN and RU translations
using local T5 model inference on GPU. No external API calls.

Features:
- BF16 model with greedy decoding (official madlad400 usage)
- Batched inference sorted by source length
- Checkpoint after each chunk file (auto-resume)
- Graceful shutdown on SIGINT/SIGTERM
- Atomic file writes (tmp + rename)

Usage:
    python scripts/translate_madlad400.py --input cdn/phrases/data --output cdn/phrases_translated/data

Requirements:
    pip install torch transformers tqdm
"""

import argparse
import json
import re
import signal
import sys
from datetime import datetime
from pathlib import Path

import torch
from tqdm import tqdm
from transformers import T5ForConditionalGeneration, T5Tokenizer


LANG_PREFIX = {"en": "<2en>", "ru": "<2ru>"}


class Checkpoint:
    def __init__(self, path: Path):
        self.path = path
        self.processed_files: list[str] = []
        self._load()

    def _load(self):
        if not self.path.exists():
            return
        try:
            with open(self.path, encoding="utf-8") as f:
                data = json.load(f)
            self.processed_files = data.get("processed_files", [])
        except (json.JSONDecodeError, OSError) as e:
            print(f"Warning: checkpoint file corrupted ({e}), starting from scratch")

    def save(self):
        tmp = self.path.with_suffix(".tmp")
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump({
                "processed_files": self.processed_files,
                "updated_at": datetime.now().isoformat(),
            }, f, ensure_ascii=False, separators=(",", ":"))
        tmp.replace(self.path)

    def is_processed(self, filename: str) -> bool:
        return filename in self.processed_files

    def mark_processed(self, filename: str):
        self.processed_files.append(filename)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Translate phrases with madlad400-7b-mt-bt")
    parser.add_argument("--input", required=True, help="Path to phrases data directory")
    parser.add_argument("--output", required=True, help="Path to output directory")
    parser.add_argument("--batch-size", type=int, default=64, help="Batch size for translation")
    parser.add_argument("--checkpoint", default="translate_checkpoint.json", help="Checkpoint file path")
    parser.add_argument("--languages", default="en,ru", help="Comma-separated target languages")
    parser.add_argument("--max-files", type=int, default=None, help="Limit number of files (for testing)")
    parser.add_argument("--model", default="google/madlad400-7b-mt-bt", help="Model name or path")
    return parser.parse_args()


def load_model(model_name: str) -> tuple[T5ForConditionalGeneration, T5Tokenizer]:
    print(f"Loading tokenizer: {model_name}")
    tokenizer = T5Tokenizer.from_pretrained(model_name)

    print(f"Loading model: {model_name}")
    model = T5ForConditionalGeneration.from_pretrained(
        model_name,
        torch_dtype=torch.bfloat16,
        device_map="auto",
    )
    model.eval()

    print_gpu_memory()
    return model, tokenizer


def print_gpu_memory():
    if not torch.cuda.is_available():
        return
    for i in range(torch.cuda.device_count()):
        allocated = torch.cuda.memory_allocated(i) / 1024**3
        reserved = torch.cuda.memory_reserved(i) / 1024**3
        total = torch.cuda.get_device_properties(i).total_memory / 1024**3
        print(f"  GPU {i}: {allocated:.1f} GB allocated, {reserved:.1f} GB reserved, {total:.1f} GB total")


def is_garbage(text: str) -> bool:
    """Detect repetition garbage from model output."""
    if not text:
        return True
    if len(text) > 300:
        return True
    unique_chars = len(set(text.replace(" ", "")))
    if unique_chars < 4 and len(text) > 20:
        return True
    if re.search(r"(.{2,}?)\1{4,}", text):
        return True
    return False


def translate_batch(
    model: T5ForConditionalGeneration,
    tokenizer: T5Tokenizer,
    texts: list[str],
    target_lang: str,
    batch_size: int,
) -> list[str]:
    prefix = LANG_PREFIX.get(target_lang, f"<2{target_lang}>")
    prefixed = [f"{prefix} {t}" for t in texts]

    all_translations: list[str] = ["" for _ in texts]

    # Sort indices by text length for efficient padding
    indices = sorted(range(len(prefixed)), key=lambda i: len(prefixed[i]))

    for batch_start in range(0, len(indices), batch_size):
        batch_indices = indices[batch_start : batch_start + batch_size]
        batch_texts = [prefixed[i] for i in batch_indices]

        inputs = tokenizer(
            batch_texts,
            padding=True,
            truncation=True,
            max_length=256,
            return_tensors="pt",
        ).to(model.device)

        with torch.no_grad():
            outputs = model.generate(
                input_ids=inputs["input_ids"],
                attention_mask=inputs["attention_mask"],
                max_new_tokens=128,
                repetition_penalty=1.2,
                no_repeat_ngram_size=3,
            )

        decoded = tokenizer.batch_decode(outputs, skip_special_tokens=True)

        for idx_in_batch, orig_idx in enumerate(batch_indices):
            all_translations[orig_idx] = decoded[idx_in_batch]

    return all_translations


def translate_single(
    model: T5ForConditionalGeneration,
    tokenizer: T5Tokenizer,
    text: str,
    target_lang: str,
) -> str:
    prefix = LANG_PREFIX.get(target_lang, f"<2{target_lang}>")
    prefixed = f"{prefix} {text}"
    inputs = tokenizer(prefixed, return_tensors="pt", truncation=True, max_length=256).to(model.device)
    with torch.no_grad():
        outputs = model.generate(
            input_ids=inputs["input_ids"],
            attention_mask=inputs["attention_mask"],
            max_new_tokens=128,
            repetition_penalty=1.2,
            no_repeat_ngram_size=3,
        )
    return tokenizer.decode(outputs[0], skip_special_tokens=True)


def collect_files(input_path: Path) -> list[Path]:
    if input_path.is_file():
        return [input_path]
    if input_path.is_dir():
        return sorted(f for f in input_path.iterdir() if re.match(r"p\d{4}\.json$", f.name))
    raise ValueError(f"Input path not found: {input_path}")


def read_json(path: Path) -> list[dict]:
    raw = path.read_bytes()
    text = raw.lstrip(b"\xef\xbb\xbf").decode("utf-8")
    return json.loads(text)


def write_json_atomic(path: Path, data: list[dict]):
    tmp = path.with_suffix(".tmp")
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, separators=(",", ":"))
    tmp.replace(path)


def process_file(
    model: T5ForConditionalGeneration,
    tokenizer: T5Tokenizer,
    input_file: Path,
    output_file: Path,
    languages: list[str],
    batch_size: int,
    pbar: tqdm,
) -> int:
    phrases = read_json(input_file)
    texts = [p.get("x", "") for p in phrases]

    translations: dict[str, list[str]] = {}

    for lang in languages:
        translations[lang] = translate_batch(model, tokenizer, texts, lang, batch_size)
        pbar.set_postfix(lang=lang, refresh=False)

    retried = 0
    failed = 0

    for i, phrase in enumerate(phrases):
        for lang in languages:
            result = translations[lang][i]
            if is_garbage(result):
                for attempt in range(3):
                    retry = translate_single(model, tokenizer, texts[i], lang)
                    if not is_garbage(retry):
                        result = retry
                        retried += 1
                        break
                else:
                    failed += 1
                    continue
            phrase[lang] = result

    write_json_atomic(output_file, phrases)
    pbar.update(len(phrases))
    if retried or failed:
        print(f"  {input_file.name}: retried={retried}, failed={failed}")
    return failed


def main():
    args = parse_args()
    input_path = Path(args.input)
    output_path = Path(args.output)
    checkpoint_path = Path(args.checkpoint)
    languages = [l.strip() for l in args.languages.split(",")]

    if not input_path.exists():
        print(f"Error: Input path not found: {input_path}")
        sys.exit(1)

    all_files = collect_files(input_path)
    if args.max_files is not None:
        all_files = all_files[: args.max_files]

    file_phrase_counts: dict[Path, int] = {}
    total_phrases = 0
    for f in all_files:
        data = read_json(f)
        file_phrase_counts[f] = len(data)
        total_phrases += len(data)

    print(f"Model:       {args.model}")
    print(f"Device:      {'cuda' if torch.cuda.is_available() else 'cpu'}")
    if torch.cuda.is_available():
        print(f"GPU:         {torch.cuda.get_device_name(0)}")
    print(f"Batch size:  {args.batch_size}")
    print(f"Languages:   {languages}")
    print(f"Input:       {input_path}")
    print(f"Output:      {output_path}")
    print(f"Files:       {len(all_files)}")
    print(f"Total phrases: {total_phrases}")
    print()

    model, tokenizer = load_model(args.model)

    checkpoint = Checkpoint(checkpoint_path)
    if checkpoint.processed_files:
        print(f"Resuming: {len(checkpoint.processed_files)} files already processed")

    output_path.mkdir(parents=True, exist_ok=True)

    shutdown_requested = False

    def on_signal(signum, frame):
        nonlocal shutdown_requested
        if not shutdown_requested:
            shutdown_requested = True
            print("\nSignal received, finishing current file and saving...")

    signal.signal(signal.SIGINT, on_signal)
    signal.signal(signal.SIGTERM, on_signal)

    files_to_process = [
        f for f in all_files if not checkpoint.is_processed(f.name)
    ]

    total_failed = 0

    with tqdm(total=total_phrases, desc="Translating", unit="phrase") as pbar:
        already_done = total_phrases - sum(
            file_phrase_counts[f]
            for f in files_to_process
        )
        pbar.update(already_done)

        for input_file in files_to_process:
            if shutdown_requested:
                break

            output_file = output_path / input_file.name
            total_failed += process_file(model, tokenizer, input_file, output_file, languages, args.batch_size, pbar)

            checkpoint.mark_processed(input_file.name)
            checkpoint.save()

    print(f"\nDone! Processed {len(checkpoint.processed_files)} files total.")
    if total_failed > 0:
        print(f"Warning: {total_failed} translations failed after 3 retries (original values kept).")


if __name__ == "__main__":
    main()
