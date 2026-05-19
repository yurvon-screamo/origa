"""
Phrase validation script using LLM API.

Validates phrases from CDN phrases dataset for educational relevance.
Criteria for invalidation (N):
- Sound-only phrases (laughing, gasping, moaning without words)
- Sexual content (explicit or euphemistic)

Features:
- Checkpoint every batch (auto-resume by ID on restart)
- Graceful shutdown on Ctrl+C

Usage:
    python scripts/validate_phrases.py --api-key <KEY> --input cdn/phrases/data
    python scripts/validate_phrases.py --api-key <KEY> --input cdn/phrases/data --output v3_report.json

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
from pathlib import Path
from dataclasses import dataclass
from typing import Optional
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor, as_completed, Future
from tqdm import tqdm


PHRASE_VALIDATION_PROMPT = """You validate Japanese phrases for a language learning app.

Y = keep, N = remove.

## Y (keep)
Normal sentences, exclamations, onomatopoeia, animal sounds.
Sounds + speech mixed = Y (e.g. "あはは、すごいね").
Normal body descriptions = Y (e.g. "お腹を揉む", "汗が出た").

## N (remove)
1. Moaning/gasping with no real words (あっ, んっ, はぁっ only).
2. Sexual content — explicit OR euphemistic. A phrase is sexual when it clearly describes a sex act, even if using indirect language. Examples of euphemistic sexual content:
   - "こんなに奥まで入って" (so deep inside me)
   - "もうイっちゃう" (I'm coming)
   - "中に出さないで" (don't come inside)
   - "気持ちいい…もっと" (feels good... more) — when combined with moaning/breathing
   - "熱いの…まだ出して" (it's hot... keep going)
   - "お兄ちゃんと一つになりたい" (I want to become one with brother) — sexual euphemism

Key: if the phrase reads as a sex scene line, it's N regardless of whether explicit words are used.

Return only Y or N.

Phrase: {phrase}
Translation: {translation}

Y/N:"""


@dataclass
class PhraseValidationResult:
    id: str
    text: str
    translation_ru: str
    translation_en: str
    llm_response: str
    is_valid: bool


class Checkpoint:
    def __init__(self, output_path: Path):
        self.output_path = output_path
        self.path = output_path.with_suffix(".checkpoint.json")
        self.seen_ids: set[str] = set()
        self.results: list[PhraseValidationResult] = []
        self._load()

    def _load(self):
        # Try checkpoint first, then fall back to final report
        load_path = None
        if self.path.exists():
            load_path = self.path
        elif self.output_path.exists():
            load_path = self.output_path
            print(f"No checkpoint found, resuming from final report: {load_path}")

        if not load_path:
            return

        with open(load_path, encoding="utf-8") as f:
            data = json.load(f)

        # Report format: invalid_phrases_details + processed_phrase_ids
        # Checkpoint format: results[]
        if "results" in data:
            results = data["results"]
            for r in results:
                phrase_id = r.get("id", "")
                if not phrase_id or phrase_id in self.seen_ids:
                    continue
                self.seen_ids.add(phrase_id)
                self.results.append(PhraseValidationResult(
                    id=phrase_id,
                    text=r.get("text", ""),
                    translation_ru=r.get("translation_ru", ""),
                    translation_en=r.get("translation_en", ""),
                    llm_response=r.get("llm_response", ""),
                    is_valid=r.get("is_valid", True),
                ))
        elif "processed_phrase_ids" in data:
            # Resume from final report — load all processed IDs
            invalid_ids = set(data.get("invalid_phrase_ids", []))
            invalid_details = {d["id"]: d for d in data.get("invalid_phrases_details", [])}
            for phrase_id in data["processed_phrase_ids"]:
                if phrase_id in self.seen_ids:
                    continue
                self.seen_ids.add(phrase_id)
                if phrase_id in invalid_details:
                    d = invalid_details[phrase_id]
                    self.results.append(PhraseValidationResult(
                        id=phrase_id,
                        text=d.get("text", ""),
                        translation_ru=d.get("translation_ru", ""),
                        translation_en=d.get("translation_en", ""),
                        llm_response=d.get("llm_response", ""),
                        is_valid=False,
                    ))
                else:
                    self.results.append(PhraseValidationResult(
                        id=phrase_id,
                        text="",
                        translation_ru="",
                        translation_en="",
                        llm_response="",
                        is_valid=True,
                    ))

    def save(self):
        output = {
            "updated_at": datetime.now().isoformat(),
            "seen_count": len(self.seen_ids),
            "results": [
                {
                    "id": r.id,
                    "text": r.text,
                    "translation_ru": r.translation_ru,
                    "translation_en": r.translation_en,
                    "llm_response": r.llm_response,
                    "is_valid": r.is_valid,
                }
                for r in self.results
            ],
        }
        tmp = self.path.with_suffix(".tmp")
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump(output, f, ensure_ascii=False)
        tmp.replace(self.path)

    def add_batch(self, batch: list[PhraseValidationResult]):
        for r in batch:
            self.seen_ids.add(r.id)
            self.results.append(r)
        self.save()

    def is_seen(self, phrase_id: str) -> bool:
        return phrase_id in self.seen_ids

    def cleanup(self):
        if self.path.exists():
            self.path.unlink()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate phrases for educational relevance using LLM"
    )
    parser.add_argument("--api-key", required=True, help="API key for LLM provider")
    parser.add_argument(
        "--api-base",
        default=os.getenv("LLM_API_BASE", "https://openrouter.ai/api/v1"),
        help="API base URL",
    )
    parser.add_argument(
        "--model",
        default=os.getenv("LLM_MODEL", "google/gemini-2.5-flash-lite"),
        help="Model to use",
    )
    parser.add_argument("--input", required=True, help="Path to phrases data directory")
    parser.add_argument("--output", default="validation_report.json", help="Output report file")
    parser.add_argument("--workers", type=int, default=10, help="Concurrent workers")
    parser.add_argument("--max-phrases", type=int, default=None, help="Max phrases to process")
    return parser.parse_args()


def collect_json_files(input_path: Path) -> list[Path]:
    if input_path.is_file():
        return [input_path]
    if input_path.is_dir():
        return sorted(input_path.rglob("p*.json"))
    raise ValueError(f"Input path not found: {input_path}")


def load_phrases_from_file(file_path: Path) -> list[dict]:
    with open(file_path, encoding="utf-8") as f:
        data = json.load(f)
    return [
        {
            "id": item.get("i", ""),
            "text": item.get("x", ""),
            "translation_ru": item.get("ru", ""),
            "translation_en": item.get("en", ""),
        }
        for item in data
    ]


def validate_single_phrase(
    api_base: str,
    api_key: str,
    model: str,
    phrase: dict,
) -> PhraseValidationResult:
    prompt = PHRASE_VALIDATION_PROMPT.format(
        phrase=phrase["text"],
        translation=phrase.get("translation_en", "") or phrase.get("translation_ru", ""),
    )

    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json",
    }

    payload = {
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 5,
        "temperature": 0.0,
    }

    for attempt in range(3):
        try:
            response = requests.post(
                f"{api_base}/chat/completions",
                headers=headers,
                json=payload,
                timeout=60,
            )

            if response.status_code == 429:
                time.sleep((attempt + 1) * 5)
                continue

            if response.status_code != 200:
                return PhraseValidationResult(
                    id=phrase["id"],
                    text=phrase["text"],
                    translation_ru=phrase["translation_ru"],
                    translation_en=phrase["translation_en"],
                    llm_response=f"HTTP {response.status_code}",
                    is_valid=True,
                )

            data = response.json()
            content = (
                data.get("choices", [{}])[0]
                .get("message", {})
                .get("content", "")
                .strip()
            )
            is_valid = content.upper().startswith("Y")

            return PhraseValidationResult(
                id=phrase["id"],
                text=phrase["text"],
                translation_ru=phrase["translation_ru"],
                translation_en=phrase["translation_en"],
                llm_response=content,
                is_valid=is_valid,
            )

        except requests.exceptions.Timeout:
            if attempt < 2:
                continue
            return PhraseValidationResult(
                id=phrase["id"],
                text=phrase["text"],
                translation_ru=phrase["translation_ru"],
                translation_en=phrase["translation_en"],
                llm_response="Timeout",
                is_valid=True,
            )
        except Exception as e:
            return PhraseValidationResult(
                id=phrase["id"],
                text=phrase["text"],
                translation_ru=phrase["translation_ru"],
                translation_en=phrase["translation_en"],
                llm_response=f"Error: {e}",
                is_valid=True,
            )

    return PhraseValidationResult(
        id=phrase["id"],
        text=phrase["text"],
        translation_ru=phrase["translation_ru"],
        translation_en=phrase["translation_en"],
        llm_response="Retries exhausted",
        is_valid=True,
    )


def process_phrase(args: tuple) -> PhraseValidationResult:
    api_base, api_key, model, phrase = args
    return validate_single_phrase(api_base, api_key, model, phrase)


def validate_phrases(
    api_base: str,
    api_key: str,
    model: str,
    input_path: Path,
    workers: int,
    max_phrases: Optional[int],
    output_path: Path,
) -> None:
    checkpoint = Checkpoint(output_path)
    if checkpoint.seen_ids:
        print(f"Resuming from checkpoint: {len(checkpoint.seen_ids)} phrases already processed")

    json_files = collect_json_files(input_path)
    print(f"Found {len(json_files)} JSON files")

    all_phrases: list[dict] = []
    for json_file in json_files:
        phrases = load_phrases_from_file(json_file)
        all_phrases.extend(phrases)

    # Skip already processed
    all_phrases = [p for p in all_phrases if not checkpoint.is_seen(p["id"])]
    print(f"Phrases to process: {len(all_phrases)} (skipped {len(checkpoint.seen_ids)} already done)")

    if max_phrases:
        all_phrases = all_phrases[:max_phrases]
        print(f"Limited to {max_phrases} phrases")

    valid_count = sum(1 for r in checkpoint.results if r.is_valid)
    invalid_count = sum(1 for r in checkpoint.results if not r.is_valid)

    shutdown_requested = False

    def on_signal(signum, frame):
        nonlocal shutdown_requested
        if not shutdown_requested:
            shutdown_requested = True
            print(f"\nSignal received, saving checkpoint and stopping...")

    signal.signal(signal.SIGINT, on_signal)
    signal.signal(signal.SIGTERM, on_signal)

    # Process in batches of workers * 2 to avoid memory/queue bloat
    batch_size = workers * 2
    total = len(all_phrases)

    with tqdm(total=total, desc="Validating") as pbar:
        with ThreadPoolExecutor(max_workers=workers) as executor:
            offset = 0
            while offset < total and not shutdown_requested:
                batch_phrases = all_phrases[offset : offset + batch_size]
                futures = {
                    executor.submit(
                        process_phrase,
                        (api_base, api_key, model, phrase),
                    ): phrase
                    for phrase in batch_phrases
                }

                batch_results: list[PhraseValidationResult] = []
                for future in as_completed(futures):
                    if shutdown_requested:
                        break
                    result = future.result()
                    batch_results.append(result)
                    if result.is_valid:
                        valid_count += 1
                    else:
                        invalid_count += 1
                    pbar.update(1)

                # Save checkpoint after each batch
                if batch_results:
                    checkpoint.add_batch(batch_results)

                offset += batch_size

    # Save final report
    invalid_phrases = [r for r in checkpoint.results if not r.is_valid]

    report = {
        "generated_at": datetime.now().isoformat(),
        "prompt_version": "v3",
        "summary": {
            "total_phrases": len(checkpoint.results),
            "valid_phrases": valid_count,
            "invalid_phrases": invalid_count,
            "invalid_percentage": (
                round(invalid_count / len(checkpoint.results) * 100, 2)
                if checkpoint.results
                else 0
            ),
        },
        "total_phrases": len(checkpoint.results),
        "invalid_phrase_ids": [p.id for p in invalid_phrases],
        "invalid_phrases_details": [
            {
                "id": p.id,
                "text": p.text,
                "translation_ru": p.translation_ru,
                "translation_en": p.translation_en,
                "llm_response": p.llm_response,
            }
            for p in invalid_phrases
        ],
        "processed_phrase_ids": [r.id for r in checkpoint.results],
    }

    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(report, f, ensure_ascii=False, indent=2)

    # Keep checkpoint for potential resume, don't delete

    print(f"\nReport saved to: {output_path}")
    print(f"Total: {len(checkpoint.results)}")
    print(f"Valid: {valid_count}")
    print(f"Invalid: {invalid_count} ({report['summary']['invalid_percentage']}%)")


def main():
    args = parse_args()

    input_path = Path(args.input)
    if not input_path.exists():
        print(f"Error: Input path not found: {input_path}")
        sys.exit(1)

    output_path = Path(args.output)

    print(f"Phrase validation v3")
    print(f"API Base: {args.api_base}")
    print(f"Model: {args.model}")
    print(f"Workers: {args.workers}")
    print(f"Output: {output_path}")

    validate_phrases(
        api_base=args.api_base,
        api_key=args.api_key,
        model=args.model,
        input_path=input_path,
        workers=args.workers,
        max_phrases=args.max_phrases,
        output_path=output_path,
    )


if __name__ == "__main__":
    main()
