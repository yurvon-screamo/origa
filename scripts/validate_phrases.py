"""
Phrase validation script using LLM API.

Validates phrases from CDN phrases dataset for educational relevance.
Criteria for invalidation (N):
- Sound-only phrases (laughing, gasping, etc.)
- Sexual content

Usage:
    python scripts/validate_phrases.py --api-key <KEY> --input cdn/phrases/data

Requirements:
    pip install requests tqdm
"""

import json
import argparse
import requests
import sys
import os
import time
from pathlib import Path
from dataclasses import dataclass, field
from typing import Optional
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor, as_completed
from tqdm import tqdm


PHRASE_VALIDATION_PROMPT = """You are a Japanese language educational content validator.
Your task is to determine if a phrase is suitable for language learning.

## Valid phrases (Y):
- Complete sentences with meaningful content
- Phrases that convey information, emotions, or actions
- Useful expressions for communication

## Invalid phrases (N) - should be removed:
1. **Sound-only phrases**: Just sounds like laughing (あはは, うふふ, ケラケラ), gasping (あっ, んっ), or other non-verbal expressions that don't carry educational meaning
2. **Sexual content**: Phrases with sexual undertones or adult content

Return ONLY a single character: Y or N.
- Y = phrase is valid for educational use
- N = phrase should be removed from dataset

Phrase to evaluate: {phrase}

Translation (if available): {translation}

Your answer (Y/N):"""


@dataclass
class PhraseValidationResult:
    id: str
    text: str
    translation_ru: str
    translation_en: str
    llm_response: str
    is_valid: bool


@dataclass
class ValidationReport:
    total_phrases: int = 0
    valid_phrases: int = 0
    invalid_phrases: int = 0
    results: list = field(default_factory=list)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate phrases for educational relevance using LLM"
    )
    parser.add_argument(
        "--api-key",
        required=True,
        help="API key for LLM provider",
    )
    parser.add_argument(
        "--api-base",
        default=os.getenv("LLM_API_BASE", "https://openrouter.ai/api/v1"),
        help="API base URL (default: https://openrouter.ai/api/v1)",
    )
    parser.add_argument(
        "--model",
        default=os.getenv("LLM_MODEL", "google/gemini-2.5-flash-lite"),
        help="Model to use (default: google/gemini-2.5-flash-lite)",
    )
    parser.add_argument(
        "--input",
        required=True,
        help="Path to phrases data directory or index file",
    )
    parser.add_argument(
        "--output",
        default="validation_report.json",
        help="Output report file path (default: validation_report.json)",
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=10,
        help="Number of concurrent workers (default: 10)",
    )
    parser.add_argument(
        "--max-phrases",
        type=int,
        default=None,
        help="Maximum number of phrases to process (default: all)",
    )
    parser.add_argument(
        "--resume-from",
        type=int,
        default=0,
        help="Resume from phrase index (for continuing interrupted validation)",
    )
    return parser.parse_args()


def collect_json_files(input_path: Path) -> list[Path]:
    """Collect all JSON files from input path."""
    if input_path.is_file():
        return [input_path]
    elif input_path.is_dir():
        json_files = []
        for p in sorted(input_path.rglob("p*.json")):
            if p.is_file():
                json_files.append(p)
        return json_files
    else:
        raise ValueError(f"Input path not found: {input_path}")


def load_phrases_from_file(file_path: Path) -> list[dict]:
    """Load phrases from a single JSON file."""
    with open(file_path, encoding="utf-8") as f:
        data = json.load(f)

    phrases = []
    for item in data:
        phrases.append({
            "id": item.get("i", ""),
            "text": item.get("x", ""),
            "translation_ru": item.get("ru", ""),
            "translation_en": item.get("en", ""),
        })
    return phrases


def validate_single_phrase(
    api_base: str,
    api_key: str,
    model: str,
    phrase: dict,
) -> PhraseValidationResult:
    """Validate a single phrase using LLM API."""
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
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "max_tokens": 5,
        "temperature": 0.0,
    }

    max_retries = 3
    for attempt in range(max_retries):
        try:
            response = requests.post(
                f"{api_base}/chat/completions",
                headers=headers,
                json=payload,
                timeout=60,
            )

            if response.status_code == 429:
                # Rate limited - wait and retry
                wait_time = (attempt + 1) * 5
                print(f"  Rate limited, waiting {wait_time}s...")
                time.sleep(wait_time)
                continue

            if response.status_code != 200:
                return PhraseValidationResult(
                    id=phrase["id"],
                    text=phrase["text"],
                    translation_ru=phrase["translation_ru"],
                    translation_en=phrase["translation_en"],
                    llm_response=f"HTTP {response.status_code}: {response.text[:200]}",
                    is_valid=True,  # Assume valid on error
                )

            data = response.json()
            content = data.get("choices", [{}])[0].get("message", {}).get("content", "").strip()
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
            if attempt < max_retries - 1:
                print(f"  Timeout, retry {attempt + 1}/{max_retries}")
                continue
            return PhraseValidationResult(
                id=phrase["id"],
                text=phrase["text"],
                translation_ru=phrase["translation_ru"],
                translation_en=phrase["translation_en"],
                llm_response="Timeout after retries",
                is_valid=True,
            )
        except Exception as e:
            return PhraseValidationResult(
                id=phrase["id"],
                text=phrase["text"],
                translation_ru=phrase["translation_ru"],
                translation_en=phrase["translation_en"],
                llm_response=f"Error: {str(e)}",
                is_valid=True,
            )

    # All retries exhausted
    return PhraseValidationResult(
        id=phrase["id"],
        text=phrase["text"],
        translation_ru=phrase["translation_ru"],
        translation_en=phrase["translation_en"],
        llm_response="All retries exhausted",
        is_valid=True,
    )


def process_phrase(args: tuple) -> PhraseValidationResult:
    """Wrapper for processing a single phrase (for ThreadPoolExecutor)."""
    api_base, api_key, model, phrase = args
    return validate_single_phrase(api_base, api_key, model, phrase)


def validate_phrases(
    api_base: str,
    api_key: str,
    model: str,
    input_path: Path,
    workers: int,
    max_phrases: Optional[int],
    resume_from: int,
) -> ValidationReport:
    """Main validation loop using thread pool."""
    report = ValidationReport()

    json_files = collect_json_files(input_path)
    print(f"Found {len(json_files)} JSON files to process")

    all_phrases: list[dict] = []

    for json_file in json_files:
        phrases = load_phrases_from_file(json_file)
        all_phrases.extend(phrases)

    # Apply limits
    if resume_from > 0:
        all_phrases = all_phrases[resume_from:]
        print(f"Resuming from index {resume_from}, {len(all_phrases)} phrases remaining")

    if max_phrases:
        all_phrases = all_phrases[:max_phrases]
        print(f"Limited to {max_phrases} phrases")

    report.total_phrases = len(all_phrases)

    # Prepare arguments for parallel processing
    task_args = [
        (api_base, api_key, model, phrase)
        for phrase in all_phrases
    ]

    # Process with thread pool
    with ThreadPoolExecutor(max_workers=workers) as executor:
        futures = {executor.submit(process_phrase, arg): i for i, arg in enumerate(task_args)}

        with tqdm(total=len(all_phrases), desc="Validating phrases") as pbar:
            for future in as_completed(futures):
                result = future.result()
                report.results.append(result)
                if result.is_valid:
                    report.valid_phrases += 1
                else:
                    report.invalid_phrases += 1
                pbar.update(1)

    return report


def save_report(report: ValidationReport, output_path: Path):
    """Save validation report to JSON file."""
    invalid_phrases = [r for r in report.results if not r.is_valid]

    output = {
        "generated_at": datetime.now().isoformat(),
        "summary": {
            "total_phrases": report.total_phrases,
            "valid_phrases": report.valid_phrases,
            "invalid_phrases": report.invalid_phrases,
            "invalid_percentage": (
                round(report.invalid_phrases / report.total_phrases * 100, 2)
                if report.total_phrases > 0 else 0
            ),
        },
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
    }

    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(output, f, ensure_ascii=False, indent=2)

    print(f"\nReport saved to: {output_path}")
    print(f"Total phrases: {report.total_phrases}")
    print(f"Valid: {report.valid_phrases}")
    print(f"Invalid (to delete): {report.invalid_phrases}")
    print(f"Invalid percentage: {output['summary']['invalid_percentage']}%")


def main():
    args = parse_args()

    input_path = Path(args.input)
    if not input_path.exists():
        print(f"Error: Input path not found: {input_path}")
        sys.exit(1)

    print(f"Starting phrase validation...")
    print(f"API Base: {args.api_base}")
    print(f"Model: {args.model}")
    print(f"Workers: {args.workers}")

    report = validate_phrases(
        api_base=args.api_base,
        api_key=args.api_key,
        model=args.model,
        input_path=input_path,
        workers=args.workers,
        max_phrases=args.max_phrases,
        resume_from=args.resume_from,
    )

    output_path = Path(args.output)
    save_report(report, output_path)


if __name__ == "__main__":
    main()