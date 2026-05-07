"""
Retranslate phrase translations using Hunyuan model.

Processes phrases from CDN phrases dataset, regenerates EN and RU translations
using the Hunyuan translation model served via vLLM.

Features:
- Checkpoint every batch (auto-resume by ID on restart)
- Graceful shutdown on Ctrl+C
- Two-step translation: JP -> EN, then JP -> RU

Usage:
    python scripts/retranslate_phrases.py --api-key <KEY> --input cdn/phrases/data --output cdn/phrases/data

Requirements:
    pip install requests tqdm
"""

import json
import argparse
import requests
import sys
import os
import time
import signal
import copy
from pathlib import Path
from typing import Optional
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor, as_completed
from tqdm import tqdm


PROMPT_JP_TO_EN = "Translate the following segment into English, without additional explanation.\n\n{source_text}"
PROMPT_JP_TO_RU = "Translate the following segment into Russian, without additional explanation.\n\n{source_text}"

VLLM_PARAMS = {
    "max_tokens": 512,
    "temperature": 0.7,
    "top_p": 0.6,
    "top_k": 20,
    "repetition_penalty": 1.05,
}


class Checkpoint:
    def __init__(self, output_path: Path):
        self.output_path = output_path
        self.path = output_path.with_suffix(".retranslate_checkpoint.json")
        self.seen_ids: set[str] = set()
        self._load()

    def _load(self):
        if not self.path.exists():
            return
        with open(self.path, encoding="utf-8") as f:
            data = json.load(f)
        self.seen_ids = set(data.get("processed_ids", []))

    def save(self, processed_ids: list[str]):
        output = {
            "updated_at": datetime.now().isoformat(),
            "processed_count": len(processed_ids),
            "processed_ids": processed_ids,
        }
        tmp = self.path.with_suffix(".tmp")
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump(output, f, ensure_ascii=False)
        tmp.replace(self.path)

    def is_seen(self, phrase_id: str) -> bool:
        return phrase_id in self.seen_ids

    def add_batch(self, ids: list[str]):
        self.seen_ids.update(ids)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Retranslate phrases using Hunyuan model"
    )
    parser.add_argument("--api-key", required=True, help="API key")
    parser.add_argument(
        "--api-base",
        default=os.getenv("LLM_API_BASE", "http://10.2.11.6:8001/v1"),
        help="API base URL",
    )
    parser.add_argument(
        "--model",
        default="hunyuan",
        help="Model name (default: hunyuan)",
    )
    parser.add_argument("--input", required=True, help="Path to phrases data directory")
    parser.add_argument("--output", required=True, help="Path to output directory for updated JSON files")
    parser.add_argument("--workers", type=int, default=50, help="Concurrent workers")
    parser.add_argument("--max-phrases", type=int, default=None, help="Max phrases to process")
    parser.add_argument(
        "--languages",
        default="en,ru",
        help="Comma-separated languages to translate (default: en,ru)",
    )
    parser.add_argument(
        "--checkpoint",
        default="retranslate_checkpoint.json",
        help="Checkpoint file path",
    )
    return parser.parse_args()


def collect_json_files(input_path: Path) -> list[Path]:
    if input_path.is_file():
        return [input_path]
    if input_path.is_dir():
        return sorted(input_path.rglob("p*.json"))
    raise ValueError(f"Input path not found: {input_path}")


def load_phrases_from_file(file_path: Path) -> list[dict]:
    with open(file_path, encoding="utf-8") as f:
        return json.load(f)


def translate(
    api_base: str,
    api_key: str,
    model: str,
    text: str,
    target_lang: str,
) -> str:
    if target_lang == "en":
        prompt = PROMPT_JP_TO_EN.format(source_text=text)
    elif target_lang == "ru":
        prompt = PROMPT_JP_TO_RU.format(source_text=text)
    else:
        prompt = f"Translate the following segment into {target_lang}, without additional explanation.\n\n{text}"

    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json",
    }

    payload = {
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        **VLLM_PARAMS,
    }

    for attempt in range(3):
        try:
            response = requests.post(
                f"{api_base}/chat/completions",
                headers=headers,
                json=payload,
                timeout=30,
            )

            if response.status_code == 429:
                time.sleep((attempt + 1) * 3)
                continue

            if response.status_code != 200:
                return ""

            data = response.json()
            content = (
                data.get("choices", [{}])[0]
                .get("message", {})
                .get("content", "")
                .strip()
            )
            return content

        except requests.exceptions.Timeout:
            if attempt < 2:
                continue
            return ""
        except Exception:
            return ""

    return ""


def process_phrase(
    api_base: str,
    api_key: str,
    model: str,
    phrase: dict,
    languages: list[str],
) -> dict:
    """Translate a single phrase. Returns updated phrase dict."""
    updated = copy.copy(phrase)
    text = phrase.get("x", "")

    if not text:
        return updated

    for lang in languages:
        translation = translate(api_base, api_key, model, text, lang)
        if translation:
            updated[lang] = translation

    return updated


def process_phrase_wrapper(args: tuple) -> tuple[str, dict]:
    phrase, api_base, api_key, model, languages = args
    result = process_phrase(api_base, api_key, model, phrase, languages)
    return phrase["i"], result


def retranslate_phrases(
    api_base: str,
    api_key: str,
    model: str,
    input_path: Path,
    output_path: Path,
    workers: int,
    max_phrases: Optional[int],
    languages: list[str],
    checkpoint_path: Path,
) -> None:
    checkpoint = Checkpoint(checkpoint_path)
    if checkpoint.seen_ids:
        print(f"Resuming from checkpoint: {len(checkpoint.seen_ids)} phrases already processed")

    json_files = collect_json_files(input_path)
    print(f"Found {len(json_files)} JSON files")

    # Ensure output directory exists
    output_path.mkdir(parents=True, exist_ok=True)

    # Track total progress across files
    all_phrase_count = 0
    already_done = 0

    for json_file in json_files:
        phrases = load_phrases_from_file(json_file)
        for p in phrases:
            pid = p.get("i", "")
            if checkpoint.is_seen(pid):
                already_done += 1
            all_phrase_count += 1

    remaining = all_phrase_count - already_done
    print(f"Total phrases: {all_phrase_count}, already done: {already_done}, remaining: {remaining}")

    if max_phrases and max_phrases < remaining:
        remaining = max_phrases

    shutdown_requested = False
    processed_in_session = 0

    def on_signal(signum, frame):
        nonlocal shutdown_requested
        if not shutdown_requested:
            shutdown_requested = True
            print(f"\nSignal received, saving checkpoint and stopping...")

    signal.signal(signal.SIGINT, on_signal)
    signal.signal(signal.SIGTERM, on_signal)

    batch_size = workers * 2
    all_processed_ids: list[str] = list(checkpoint.seen_ids)

    with tqdm(total=remaining, desc="Translating") as pbar:
        with ThreadPoolExecutor(max_workers=workers) as executor:
            for json_file in json_files:
                if shutdown_requested:
                    break

                phrases = load_phrases_from_file(json_file)
                # Filter out already processed
                to_process = [p for p in phrases if not checkpoint.is_seen(p.get("i", ""))]
                if not to_process:
                    continue

                # Limit if max_phrases is set
                if max_phrases:
                    left = max_phrases - processed_in_session
                    if left <= 0:
                        break
                    to_process = to_process[:left]

                # Process in batches
                offset = 0
                while offset < len(to_process) and not shutdown_requested:
                    batch = to_process[offset : offset + batch_size]

                    futures = {
                        executor.submit(
                            process_phrase_wrapper,
                            (p, api_base, api_key, model, languages),
                        ): p
                        for p in batch
                    }

                    batch_results: dict[str, dict] = {}
                    for future in as_completed(futures):
                        if shutdown_requested:
                            break
                        phrase_id, updated = future.result()
                        batch_results[phrase_id] = updated
                        pbar.update(1)

                    # Apply translations and save file
                    if batch_results:
                        # Re-read file, update translations, write back
                        current_phrases = load_phrases_from_file(json_file)
                        updated_any = False
                        for i, p in enumerate(current_phrases):
                            pid = p.get("i", "")
                            if pid in batch_results:
                                current_phrases[i] = batch_results[pid]
                                updated_any = True

                        if updated_any:
                            out_file = output_path / json_file.name
                            tmp = out_file.with_suffix(".tmp")
                            with open(tmp, "w", encoding="utf-8") as f:
                                json.dump(current_phrases, f, ensure_ascii=False, separators=(",", ":"))
                            tmp.replace(out_file)

                        # Update checkpoint
                        batch_ids = list(batch_results.keys())
                        checkpoint.add_batch(batch_ids)
                        all_processed_ids.extend(batch_ids)
                        checkpoint.save(all_processed_ids)
                        processed_in_session += len(batch_results)

                    offset += batch_size

    print(f"\nDone! Processed {processed_in_session} phrases this session.")
    print(f"Total translated: {len(all_processed_ids)}")


def main():
    args = parse_args()

    input_path = Path(args.input)
    output_path = Path(args.output)
    checkpoint_path = Path(args.checkpoint)

    if not input_path.exists():
        print(f"Error: Input path not found: {input_path}")
        sys.exit(1)

    languages = [l.strip() for l in args.languages.split(",")]

    print(f"Retranslate phrases")
    print(f"API Base: {args.api_base}")
    print(f"Model: {args.model}")
    print(f"Workers: {args.workers}")
    print(f"Languages: {languages}")
    print(f"Input: {input_path}")
    print(f"Output: {output_path}")

    retranslate_phrases(
        api_base=args.api_base,
        api_key=args.api_key,
        model=args.model,
        input_path=input_path,
        output_path=output_path,
        workers=args.workers,
        max_phrases=args.max_phrases,
        languages=languages,
        checkpoint_path=checkpoint_path,
    )


if __name__ == "__main__":
    main()
