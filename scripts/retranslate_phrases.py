"""
Retranslate phrase translations using Hunyuan model.

Processes phrases from CDN phrases dataset, regenerates EN and RU translations.
Processes each JSON file atomically: read -> translate -> write.

Features:
- Checkpoint after each file (auto-resume by ID on restart)
- Graceful shutdown on Ctrl+C
- Fixed batch size (200) independent of worker count

Usage:
    uv run scripts/retranslate_phrases.py --api-key blame --api-base http://10.2.11.6:8002/v1 --model tencent/HY-MT1.5-7B-FP8 --workers 200 --input cdn/phrases/data --output cdn/phrases/data

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

BATCH_SIZE = 200


class Checkpoint:
    def __init__(self, path: Path):
        self.path = path
        self.seen_ids: set[str] = set()
        self._load()

    def _load(self):
        if not self.path.exists():
            return
        try:
            with open(self.path, encoding="utf-8") as f:
                data = json.load(f)
            self.seen_ids = set(data.get("processed_ids", []))
        except (json.JSONDecodeError, OSError):
            pass

    def save(self):
        tmp = self.path.with_suffix(".tmp")
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump({
                "updated_at": datetime.now().isoformat(),
                "processed_count": len(self.seen_ids),
                "processed_ids": sorted(self.seen_ids),
            }, f, ensure_ascii=False, separators=(",", ":"))
        tmp.replace(self.path)

    def is_seen(self, phrase_id: str) -> bool:
        return phrase_id in self.seen_ids

    def mark_seen(self, ids: list[str]):
        self.seen_ids.update(ids)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Retranslate phrases using Hunyuan model")
    parser.add_argument("--api-key", required=True, help="API key")
    parser.add_argument("--api-base", default=os.getenv("LLM_API_BASE", "http://10.2.11.6:8001/v1"), help="API base URL")
    parser.add_argument("--model", default="hunyuan", help="Model name")
    parser.add_argument("--input", required=True, help="Path to phrases data directory")
    parser.add_argument("--output", required=True, help="Path to output directory")
    parser.add_argument("--workers", type=int, default=100, help="Concurrent workers")
    parser.add_argument("--max-phrases", type=int, default=None, help="Max phrases to process")
    parser.add_argument("--checkpoint", default="retranslate_checkpoint.json", help="Checkpoint file")
    parser.add_argument("--languages", default="en,ru", help="Languages (default: en,ru)")
    return parser.parse_args()


def collect_json_files(input_path: Path) -> list[Path]:
    if input_path.is_file():
        return [input_path]
    if input_path.is_dir():
        return sorted(input_path.rglob("p*.json"))
    raise ValueError(f"Input path not found: {input_path}")


def translate_text(api_base: str, api_key: str, model: str, text: str, target_lang: str) -> str:
    if target_lang == "en":
        prompt = PROMPT_JP_TO_EN.format(source_text=text)
    elif target_lang == "ru":
        prompt = PROMPT_JP_TO_RU.format(source_text=text)
    else:
        prompt = f"Translate the following segment into {target_lang}, without additional explanation.\n\n{text}"

    headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}
    payload = {"model": model, "messages": [{"role": "user", "content": prompt}], **VLLM_PARAMS}

    for attempt in range(3):
        try:
            resp = requests.post(f"{api_base}/chat/completions", headers=headers, json=payload, timeout=30)
            if resp.status_code == 429:
                time.sleep((attempt + 1) * 3)
                continue
            if resp.status_code != 200:
                return ""
            content = resp.json().get("choices", [{}])[0].get("message", {}).get("content", "").strip()
            return content
        except requests.exceptions.Timeout:
            if attempt < 2:
                continue
            return ""
        except Exception:
            return ""
    return ""


def process_phrase(args: tuple) -> tuple[str, str, str]:
    phrase, api_base, api_key, model, languages = args
    text = phrase.get("x", "")
    en_text = ""
    ru_text = ""
    for lang in languages:
        translation = translate_text(api_base, api_key, model, text, lang)
        if lang == "en" and translation:
            en_text = translation
        elif lang == "ru" and translation:
            ru_text = translation
    return phrase["i"], en_text, ru_text


def retranslate_phrases(
    api_base: str, api_key: str, model: str,
    input_path: Path, output_path: Path,
    workers: int, max_phrases: Optional[int],
    languages: list[str], checkpoint_path: Path,
) -> None:
    checkpoint = Checkpoint(checkpoint_path)
    if checkpoint.seen_ids:
        print(f"Resuming from checkpoint: {len(checkpoint.seen_ids)} phrases already processed")

    json_files = collect_json_files(input_path)
    output_path.mkdir(parents=True, exist_ok=True)

    total_phrases = sum(len(json.load(open(jf, encoding="utf-8"))) for jf in json_files)
    already_done = sum(1 for jf in json_files for p in json.load(open(jf, encoding="utf-8")) if checkpoint.is_seen(p.get("i", "")))
    remaining = total_phrases - already_done
    if max_phrases and max_phrases < remaining:
        remaining = max_phrases

    print(f"Total: {total_phrases}, done: {already_done}, remaining: {remaining}")
    print(f"Files: {len(json_files)}, Workers: {workers}, Batch: {BATCH_SIZE}")

    shutdown_requested = False
    processed_total = 0

    def on_signal(signum, frame):
        nonlocal shutdown_requested
        if not shutdown_requested:
            shutdown_requested = True
            print("\nSignal received, finishing current file and saving...")

    signal.signal(signal.SIGINT, on_signal)
    signal.signal(signal.SIGTERM, on_signal)

    with tqdm(total=remaining, desc="Translating") as pbar:
        with ThreadPoolExecutor(max_workers=workers) as executor:
            for json_file in json_files:
                if shutdown_requested:
                    break

                with open(json_file, encoding="utf-8") as f:
                    file_phrases = json.load(f)

                to_process = [p for p in file_phrases if not checkpoint.is_seen(p.get("i", ""))]
                if not to_process:
                    continue

                if max_phrases:
                    left = max_phrases - processed_total
                    if left <= 0:
                        break
                    to_process = to_process[:left]

                file_results: dict[str, tuple[str, str]] = {}
                offset = 0

                while offset < len(to_process) and not shutdown_requested:
                    batch = to_process[offset : offset + BATCH_SIZE]
                    task_args = [(p, api_base, api_key, model, languages) for p in batch]

                    futures = {executor.submit(process_phrase, arg): arg[0] for arg in task_args}

                    for future in as_completed(futures):
                        if shutdown_requested:
                            break
                        phrase_id, en_text, ru_text = future.result()
                        file_results[phrase_id] = (en_text, ru_text)
                        pbar.update(1)

                    checkpoint.mark_seen([p["i"] for p in batch])
                    checkpoint.save()
                    offset += BATCH_SIZE

                if file_results:
                    for i, p in enumerate(file_phrases):
                        pid = p.get("i", "")
                        if pid in file_results:
                            en_text, ru_text = file_results[pid]
                            if en_text:
                                file_phrases[i]["en"] = en_text
                            if ru_text:
                                file_phrases[i]["ru"] = ru_text

                    out_file = output_path / json_file.name
                    tmp = out_file.with_suffix(".tmp")
                    with open(tmp, "w", encoding="utf-8") as f:
                        json.dump(file_phrases, f, ensure_ascii=False, separators=(",", ":"))
                    tmp.replace(out_file)

                processed_total += len(file_results)

    checkpoint.save()
    print(f"\nDone! Processed {processed_total} this session. Total: {len(checkpoint.seen_ids)}")


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
    print(f"API: {args.api_base}, Model: {args.model}, Workers: {args.workers}")
    print(f"Languages: {languages}")
    print(f"Input: {input_path}, Output: {output_path}")

    retranslate_phrases(
        api_base=args.api_base, api_key=args.api_key, model=args.model,
        input_path=input_path, output_path=output_path,
        workers=args.workers, max_phrases=args.max_phrases,
        languages=languages, checkpoint_path=checkpoint_path,
    )


if __name__ == "__main__":
    main()