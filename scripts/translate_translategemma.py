"""
Batch translation of Japanese phrases using TranslateGemma 4B via vLLM.

Processes JSON chunk files (p0000.json – p0197.json) containing Japanese phrases
and regenerates "ru" and/or "en" translations using vLLM's OpenAI-compatible API.
"""

import argparse
import json
import os
import shutil
import signal
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

import openai
from tqdm import tqdm

shutdown_requested = False


def handle_signal(signum, _frame):
    global shutdown_requested
    shutdown_requested = True
    print(f"\nReceived signal {signum}, shutting down gracefully...")


signal.signal(signal.SIGINT, handle_signal)
signal.signal(signal.SIGTERM, handle_signal)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Batch translate Japanese phrases using TranslateGemma 4B via vLLM"
    )
    parser.add_argument("--input", required=True, help="Directory with p*.json chunk files")
    parser.add_argument("--output", required=True, help="Directory for translated chunk files")
    parser.add_argument("--workers", type=int, default=50, help="Concurrent requests (default: 50)")
    parser.add_argument(
        "--checkpoint",
        default="checkpoint_translate.json",
        help="Checkpoint file path (default: checkpoint_translate.json)",
    )
    parser.add_argument(
        "--languages",
        default="en,ru",
        help="Comma-separated target languages (default: en,ru)",
    )
    parser.add_argument("--max-files", type=int, default=None, help="Max chunk files to process")
    parser.add_argument(
        "--api-url",
        default="http://localhost:8000/v1",
        help="vLLM OpenAI-compatible API URL (default: http://localhost:8000/v1)",
    )
    parser.add_argument(
        "--model",
        default="Infomaniak-AI/vllm-translategemma-4b-it",
        help="Model name (default: Infomaniak-AI/vllm-translategemma-4b-it)",
    )
    return parser


def load_checkpoint(path: str) -> set[str]:
    if not os.path.exists(path):
        return set()
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    return set(data.get("completed_files", []))


def save_checkpoint(path: str, completed: set[str]) -> None:
    tmp_fd, tmp_path = tempfile.mkstemp(dir=os.path.dirname(path) or ".", suffix=".json")
    try:
        with os.fdopen(tmp_fd, "w", encoding="utf-8") as f:
            json.dump({"completed_files": sorted(completed)}, f, ensure_ascii=False, indent=2)
        shutil.move(tmp_path, path)
    except BaseException:
        if os.path.exists(tmp_path):
            os.remove(tmp_path)
        raise


def read_json(path: Path) -> list[dict]:
    raw = path.read_text(encoding="utf-8-sig")
    return json.loads(raw)


def write_json_atomic(path: Path, data: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_fd, tmp_path = tempfile.mkstemp(dir=str(path.parent), suffix=".json")
    try:
        with os.fdopen(tmp_fd, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, indent=2)
        shutil.move(tmp_path, str(path))
    except BaseException:
        if os.path.exists(tmp_path):
            os.remove(tmp_path)
        raise


def translate_text(client: openai.OpenAI, model: str, text: str, target_lang: str) -> str:
    prompt = f"<<<source>>>ja<<<target>>>{target_lang}<<<text>>>{text}"
    response = client.chat.completions.create(
        model=model,
        messages=[
            {
                "role": "user",
                "content": prompt,
            }
        ],
        max_tokens=200,
        temperature=0.0,
    )
    return response.choices[0].message.content.strip()


def translate_phrase(
    client: openai.OpenAI,
    model: str,
    phrase: dict,
    target_langs: list[str],
) -> tuple[dict, bool]:
    """Translate a single phrase into all target languages. Returns (phrase, had_error)."""
    japanese = phrase.get("x", "")
    if not japanese:
        return phrase, False

    had_error = False
    for lang in target_langs:
        try:
            translated = translate_text(client, model, japanese, lang)
            if translated:
                phrase[lang] = translated
            else:
                had_error = True
        except Exception as exc:
            had_error = True
            print(f"  Error translating [{lang}] {japanese[:40]}...: {exc}", file=sys.stderr)

    return phrase, had_error


def process_file(
    client: openai.OpenAI,
    model: str,
    input_path: Path,
    output_path: Path,
    target_langs: list[str],
    workers: int,
) -> tuple[int, int]:
    """Process one chunk file. Returns (success_count, error_count)."""
    phrases = read_json(input_path)
    success = 0
    errors = 0

    with ThreadPoolExecutor(max_workers=workers) as pool:
        futures = {
            pool.submit(translate_phrase, client, model, dict(p), target_langs): idx
            for idx, p in enumerate(phrases)
        }

        with tqdm(total=len(phrases), desc=input_path.name, unit="phrase") as pbar:
            for future in as_completed(futures):
                if shutdown_requested:
                    pool.shutdown(wait=False, cancel_futures=True)
                    break
                idx = futures[future]
                try:
                    translated_phrase, had_error = future.result()
                    phrases[idx] = translated_phrase
                    if had_error:
                        errors += 1
                    else:
                        success += 1
                except Exception as exc:
                    errors += 1
                    print(f"  Fatal error on phrase {idx}: {exc}", file=sys.stderr)
                pbar.update(1)

    write_json_atomic(output_path, phrases)
    return success, errors


def collect_chunk_files(input_dir: str, max_files: int | None) -> list[Path]:
    files = sorted(Path(input_dir).glob("p*.json"))
    if max_files is not None:
        files = files[:max_files]
    return files


def main() -> None:
    args = build_parser().parse_args()
    target_langs = [lang.strip() for lang in args.languages.split(",")]
    chunk_files = collect_chunk_files(args.input, args.max_files)
    completed = load_checkpoint(args.checkpoint)

    client = openai.OpenAI(base_url=args.api_url, api_key="dummy")

    total_phrases = sum(len(read_json(f)) for f in chunk_files)

    print("=" * 60)
    print("TranslateGemma 4B — vLLM Batch Translation")
    print("=" * 60)
    print(f"  vLLM API URL:   {args.api_url}")
    print(f"  Model:          {args.model}")
    print(f"  Workers:        {args.workers}")
    print(f"  Languages:      {target_langs}")
    print(f"  Input dir:      {args.input}")
    print(f"  Output dir:     {args.output}")
    print(f"  Chunk files:    {len(chunk_files)}")
    print(f"  Total phrases:  {total_phrases}")
    print(f"  Already done:   {len(completed)}")
    print(f"  Checkpoint:     {args.checkpoint}")
    print("=" * 60)

    os.makedirs(args.output, exist_ok=True)

    total_success = 0
    total_errors = 0
    start_time = time.time()

    for chunk_file in chunk_files:
        if shutdown_requested:
            print("\nShutdown requested, stopping.")
            break

        if chunk_file.name in completed:
            print(f"Skip {chunk_file.name} (already done)")
            continue

        output_path = Path(args.output) / chunk_file.name
        print(f"\nProcessing {chunk_file.name}...")
        file_success, file_errors = process_file(
            client, args.model, chunk_file, output_path, target_langs, args.workers
        )
        total_success += file_success
        total_errors += file_errors

        completed.add(chunk_file.name)
        save_checkpoint(args.checkpoint, completed)

    elapsed = time.time() - start_time
    print(f"\n{'=' * 60}")
    print(f"Done in {elapsed:.1f}s")
    print(f"  Success: {total_success}")
    print(f"  Errors:  {total_errors}")
    print(f"  Files:   {len(completed)}/{len(chunk_files)}")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
