#!/usr/bin/env python3
"""
Detect repetitive Japanese phrases (stuttering, laughter, interjections)
in phrase chunks.

Output format is compatible with scripts/remove_invalid_phrases.py.

Usage:
    python scripts/detect_jp_repetition.py --input cdn/phrases/data --output jp_repetition_report.json
    python scripts/detect_jp_repetition.py --input cdn/phrases/data --chunk 0 --verbose
"""

import json
import argparse
import os
import re
import tempfile
from pathlib import Path
from datetime import datetime, timezone


# ── Constants ──────────────────────────────────────────────────────

_JP_PUNCT_RE = re.compile(r'[。、！？…―・～♪「」『』（）\(\)\s]+')

_HIRAGANA_RANGE = (0x3040, 0x309F)
_KATAKANA_RANGE = (0x30A0, 0x30FF)
_CHOONPU = '\u30fc'



def _is_kana(char: str) -> bool:
    cp = ord(char)
    return (_HIRAGANA_RANGE[0] <= cp <= _HIRAGANA_RANGE[1]
            or _KATAKANA_RANGE[0] <= cp <= _KATAKANA_RANGE[1]) and char != _CHOONPU


# ── Detectors ──────────────────────────────────────────────────────

def detect_word_repeat(text: str) -> dict | None:
    """
    Detect consecutive word/token repeats (>=3 identical tokens in a row).

    Tokens are produced by splitting on Japanese punctuation and whitespace.
    Only tokens >=2 characters are considered.
    """
    tokens = [t for t in _JP_PUNCT_RE.split(text) if t]
    if not tokens:
        return None

    text_len = len(text)
    best_run_len = 0
    best_token = ""

    run_len = 1
    for i in range(1, len(tokens)):
        if tokens[i] == tokens[i - 1] and len(tokens[i]) >= 2:
            run_len += 1
        else:
            if run_len >= 3 and len(tokens[i - 1]) >= 2 and run_len > best_run_len:
                best_run_len = run_len
                best_token = tokens[i - 1]
            run_len = 1

    # Check the last run
    if run_len >= 3 and len(tokens[-1]) >= 2 and run_len > best_run_len:
        best_run_len = run_len
        best_token = tokens[-1]

    if best_run_len < 3:
        return None

    dominance = (len(best_token) * best_run_len) / text_len if text_len > 0 else 0.0

    return {
        "type": "word_repeat",
        "token": best_token,
        "count": best_run_len,
        "dominance": round(dominance, 4),
    }


def detect_kana_repeat(text: str) -> dict | None:
    """
    Detect repeated identical kana characters (>=5 in a row).

    Checks hiragana (U+3040-U+309F) and katakana (U+30A0-U+30FF).
    Excludes chōonpu (ー, U+30FC) which is used for vowel lengthening.
    """
    text_len = len(text)
    if text_len == 0:
        return None

    best_count = 0
    best_char = ""

    run_char = ""
    run_count = 0

    for char in text:
        if _is_kana(char) and char == run_char:
            run_count += 1
        else:
            if run_count >= 5 and run_count > best_count:
                best_count = run_count
                best_char = run_char
            if _is_kana(char):
                run_char = char
                run_count = 1
            else:
                run_char = ""
                run_count = 0

    # Check the last run
    if run_count >= 5 and run_count > best_count:
        best_count = run_count
        best_char = run_char

    if best_count < 5:
        return None

    dominance = best_count / text_len

    return {
        "type": "kana_repeat",
        "char": best_char,
        "count": best_count,
        "dominance": round(dominance, 4),
    }


def detect_char_repeat(text: str) -> dict | None:
    """
    Detect repeated identical characters (>=6 in a row).

    Checks ANY character including punctuation, dashes, dots, etc.
    """
    text_len = len(text)
    if text_len == 0:
        return None

    best_count = 0
    best_char = ""

    run_char = ""
    run_count = 0

    for char in text:
        if char == run_char:
            run_count += 1
        else:
            if run_count >= 6 and run_count > best_count:
                best_count = run_count
                best_char = run_char
            run_char = char
            run_count = 1

    # Check the last run
    if run_count >= 6 and run_count > best_count:
        best_count = run_count
        best_char = run_char

    if best_count < 6:
        return None

    dominance = best_count / text_len

    return {
        "type": "char_repeat",
        "char": best_char,
        "count": best_count,
        "dominance": round(dominance, 4),
    }


# ── Classification ─────────────────────────────────────────────────

def classify_phrase(phrase: dict) -> tuple[str, list[dict]]:
    """
    Classify a phrase based on repetition detection.

    Returns ("remove" | "keep", list_of_detection_results).
    """
    jp_text = phrase['x']

    results: list[dict] = []

    r = detect_word_repeat(jp_text)
    if r is not None:
        results.append(r)

    r = detect_kana_repeat(jp_text)
    if r is not None:
        results.append(r)

    r = detect_char_repeat(jp_text)
    if r is not None:
        results.append(r)

    if not results:
        return "keep", []

    # Rule 1: word repeat >=3 AND dominance >= 50%
    for r in results:
        if r["type"] == "word_repeat" and r["count"] >= 3 and r["dominance"] >= 0.5:
            return "remove", results

    # Rule 2: kana repeat >=5 AND dominance >= 60%
    for r in results:
        if r["type"] == "kana_repeat" and r["count"] >= 5 and r["dominance"] >= 0.6:
            return "remove", results

    # Rule 3: char repeat >=6 AND dominance >= 30%
    for r in results:
        if r["type"] == "char_repeat" and r["count"] >= 6 and r["dominance"] >= 0.3:
            return "remove", results

    return "keep", results


def format_reason(result: dict) -> str:
    """Format a detection result into a human-readable reason string."""
    rtype = result["type"]
    if rtype == "word_repeat":
        return f"word_repeat: {result['token']}\u00d7{result['count']} (dominance: {result['dominance']:.0%})"
    if rtype == "kana_repeat":
        return f"kana_repeat: {result['char']}\u00d7{result['count']} (dominance: {result['dominance']:.0%})"
    if rtype == "char_repeat":
        return f"char_repeat: {result['char']}\u00d7{result['count']} (dominance: {result['dominance']:.0%})"
    return f"{rtype}: {result}"


# ── Chunk processing ───────────────────────────────────────────────

def process_chunk(path: Path, chunk_num: int) -> tuple[list[dict], int]:
    """
    Process a single chunk file.

    Returns (list_of_removed_phrase_details, total_phrase_count).
    """
    removed: list[dict] = []

    try:
        with open(path, encoding="utf-8") as f:
            phrases = json.load(f)
    except (json.JSONDecodeError, OSError) as exc:
        print(f"Warning: skipping {path} — {exc}")
        return removed, 0

    for phrase in phrases:
        label, results = classify_phrase(phrase)

        if label == "remove":
            reasons = [format_reason(r) for r in results]
            removed.append({
                "i": phrase["i"],
                "chunk": chunk_num,
                "x": phrase.get("x", ""),
                "reasons": reasons,
            })

    return removed, len(phrases)


# ── Report building ────────────────────────────────────────────────

def build_report(
    chunk_removed: dict[int, list[dict]],
    total_phrases: int,
) -> dict:
    """Aggregate chunk results into the final JSON report."""
    all_removed: list[dict] = []
    for chunk_num in sorted(chunk_removed.keys()):
        all_removed.extend(chunk_removed[chunk_num])

    remove_count = len(all_removed)
    keep_count = total_phrases - remove_count

    return {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "script_version": "1.2",
        "summary": {
            "total_phrases": total_phrases,
            "remove": remove_count,
            "keep": keep_count,
        },
        "invalid_phrase_ids": [entry["i"] for entry in all_removed],
        "invalid_phrases_details": all_removed,
    }


def atomic_write_json(path: Path, data: dict) -> None:
    """Write JSON to a file atomically via tempfile + os.replace."""
    tmp_fd, tmp_path = tempfile.mkstemp(
        dir=path.parent,
        suffix=".tmp",
        prefix=path.stem + "_",
    )
    try:
        with os.fdopen(tmp_fd, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, indent=2)
        os.replace(tmp_path, path)
    except BaseException:
        # Clean up temp file on error
        try:
            os.unlink(tmp_path)
        except OSError:
            pass
        raise


# ── CLI ────────────────────────────────────────────────────────────

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Detect repetitive Japanese phrases (stuttering, laughter, interjections)",
    )
    parser.add_argument(
        "--input",
        required=True,
        help="Path to directory with chunk files (p*.json)",
    )
    parser.add_argument(
        "--output",
        default="jp_repetition_report.json",
        help="Path to output JSON report (default: jp_repetition_report.json)",
    )
    parser.add_argument(
        "--chunk",
        type=int,
        default=None,
        help="Process only a specific chunk number",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Print details for each removed phrase",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    input_dir = Path(args.input)
    if not input_dir.is_dir():
        print(f"Error: input directory not found: {input_dir}")
        return

    chunk_files = sorted(input_dir.glob("p*.json"))
    if not chunk_files:
        print(f"Error: no chunk files (p*.json) found in {input_dir}")
        return

    if args.chunk is not None:
        target_name = f"p{args.chunk:04d}.json"
        chunk_files = [p for p in chunk_files if p.name == target_name]
        if not chunk_files:
            print(f"Error: chunk {args.chunk} not found (expected: {target_name})")
            return

    print(f"Processing {len(chunk_files)} chunk file(s)...")

    chunk_removed: dict[int, list[dict]] = {}
    total_phrases = 0

    for chunk_path in chunk_files:
        try:
            chunk_num = int(chunk_path.stem[1:])
        except ValueError:
            print(f"Warning: skipping file with unexpected name: {chunk_path.name}")
            continue

        removed, phrase_count = process_chunk(chunk_path, chunk_num)
        chunk_removed[chunk_num] = removed
        total_phrases += phrase_count

        if phrase_count > 0:
            pct = (len(removed) / phrase_count) * 100
            print(f"  {chunk_path.name}: {phrase_count} phrases, "
                  f"{len(removed)} remove ({pct:.1f}%)")

        if args.verbose and removed:
            for entry in removed:
                text_preview = entry["x"][:60]
                reasons_str = "; ".join(entry["reasons"])
                print(f"    REMOVE {entry['i']}: {text_preview}")
                print(f"           reasons: {reasons_str}")

    report = build_report(chunk_removed, total_phrases)

    output_path = Path(args.output)
    atomic_write_json(output_path, report)

    s = report["summary"]
    print(f"\nReport saved to: {output_path}")
    print(f"Total: {s['total_phrases']} phrases")
    print(f"  Remove: {s['remove']}")
    print(f"  Keep:   {s['keep']}")


if __name__ == "__main__":
    main()
