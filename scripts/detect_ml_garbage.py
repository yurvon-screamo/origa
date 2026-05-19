#!/usr/bin/env python3
"""
Detect ML garbage (collapsed repetitions, wrong language, gibberish)
in phrase translation chunks.

Output format is compatible with scripts/remove_invalid_phrases.py.

Usage:
    python scripts/detect_ml_garbage.py --input cdn/phrases/data --output garbage_report.json
    python scripts/detect_ml_garbage.py --input cdn/phrases/data --chunk 0 --verbose
"""

import json
import argparse
import re
import os
from pathlib import Path
from datetime import datetime, timezone


# ── Character classes ──────────────────────────────────────────────

_CJK_RE = re.compile(r'[\u4e00-\u9fff]')
_HIRAGANA_RE = re.compile(r'[\u3040-\u309f]')
_CYRILLIC_RE = re.compile(r'[\u0400-\u04ff]')

_LAT_VOWELS_RE = re.compile(r'[aeiou]', re.IGNORECASE)
_LAT_CONSONANTS_RE = re.compile(r'[bcdfghjklmnpqrstvwxyz]', re.IGNORECASE)
_CYR_VOWELS_RE = re.compile(r'[аеёиоуыэюя]', re.IGNORECASE)
_CYR_CONSONANTS_RE = re.compile(r'[бвгджзйклмнпрстфхцчшщ]', re.IGNORECASE)
_NON_TEXT_RE = re.compile(r'[^a-zа-яё\s\.,!?;:()\[\]«»"\'\-]', re.IGNORECASE)

_TOKEN_SPLIT_RE = re.compile(r'[ ,.!?;:()\[\]\n\t]+')
_JP_TOKEN_SPLIT_RE = re.compile(r'[ ,.!?;:()\[\]、「」。、！？\n]+')

_EXEMPT_NEAR_EMPTY = frozenset({'no', 'ok', 'да', 'нет'})

# ── Detectors ──────────────────────────────────────────────────────

def detect_repetition_collapse(text: str) -> bool:
    tokens = [t for t in _TOKEN_SPLIT_RE.split(text.lower()) if t]
    if len(tokens) < 10:
        return False

    # Unigram: ≥10 identical tokens in a row
    run_length = 1
    for i in range(1, len(tokens)):
        if tokens[i] == tokens[i - 1]:
            run_length += 1
            if run_length >= 10:
                return True
        else:
            run_length = 1

    return False


def detect_length_anomaly(jp_text: str, translation: str) -> bool:
    """Translation is much longer than original (indicates collapse / looping)."""
    jp_len = len(jp_text)
    tr_len = len(translation)

    if jp_len <= 5:
        return tr_len > 300

    return tr_len > 8 * jp_len and tr_len > 150


def detect_empty_or_near_empty(text: str) -> bool:
    """Translation is empty or nearly empty (but not a valid short response)."""
    stripped = text.strip()

    if len(stripped) == 0:
        return True

    if len(stripped) <= 2 and stripped.lower() not in _EXEMPT_NEAR_EMPTY:
        return True

    return False


def detect_identity(text: str, jp_text: str) -> bool:
    """Translation contains Japanese characters instead of actual translation."""
    cjk_count = len(_CJK_RE.findall(text))
    hiragana_count = len(_HIRAGANA_RE.findall(text))

    return (cjk_count + hiragana_count) >= 3


def detect_charset_mismatch(text: str, expected_lang: str) -> bool:
    """Translation contains characters from a different script family."""
    if expected_lang == 'ru':
        return len(_CJK_RE.findall(text)) >= 5

    if expected_lang == 'en':
        return len(_CYRILLIC_RE.findall(text)) >= 3

    return False


def detect_gibberish(text: str) -> bool:
    """Detect random garbage / unreadable text."""
    if len(text) <= 20:
        return False

    text_lower = text.lower()
    total = len(text)

    # Non-text character ratio
    non_text_count = len(_NON_TEXT_RE.findall(text_lower))
    if non_text_count > total * 0.3:
        return True

    # Consonant-to-vowel ratio (Latin)
    lat_v = len(_LAT_VOWELS_RE.findall(text_lower))
    lat_c = len(_LAT_CONSONANTS_RE.findall(text_lower))
    lat_total = lat_v + lat_c

    if lat_total > 10:
        if lat_v == 0 or lat_c == 0:
            return True
        ratio = lat_c / lat_v
        if ratio < 0.3 or ratio > 5.0:
            return True

    # Consonant-to-vowel ratio (Cyrillic)
    cyr_v = len(_CYR_VOWELS_RE.findall(text_lower))
    cyr_c = len(_CYR_CONSONANTS_RE.findall(text_lower))
    cyr_total = cyr_v + cyr_c

    if cyr_total > 10:
        if cyr_v == 0 or cyr_c == 0:
            return True
        ratio = cyr_c / cyr_v
        if ratio < 0.3 or ratio > 5.0:
            return True

    return False


# ── Classification ─────────────────────────────────────────────────

def classify_phrase(phrase: dict) -> tuple[str, list[str], list[str]]:
    """
    Classify a phrase and return (critical|warning|ok, ru_issues, en_issues).

    CRITICAL: repetition_collapse detected in BOTH ru and en translations
              (only repetition_collapse — not length_anomaly).
    WARNING:  any other issue in either translation.
    """
    jp_text = phrase['x']
    ru_text = phrase['ru']
    en_text = phrase['en']

    ru_issues: list[str] = []
    en_issues: list[str] = []

    # repetition_collapse — самый важный детектор
    if detect_repetition_collapse(ru_text):
        ru_issues.append('repetition_collapse')
    if detect_repetition_collapse(en_text):
        en_issues.append('repetition_collapse')

    # length_anomaly
    if detect_length_anomaly(jp_text, ru_text):
        ru_issues.append('length_anomaly')
    if detect_length_anomaly(jp_text, en_text):
        en_issues.append('length_anomaly')

    # empty_or_near_empty
    if detect_empty_or_near_empty(ru_text):
        ru_issues.append('empty_or_near_empty')
    if detect_empty_or_near_empty(en_text):
        en_issues.append('empty_or_near_empty')

    # gibberish
    if detect_gibberish(ru_text):
        ru_issues.append('gibberish')
    if detect_gibberish(en_text):
        en_issues.append('gibberish')

    # charset_mismatch
    if detect_charset_mismatch(ru_text, 'ru'):
        ru_issues.append('charset_mismatch')
    if detect_charset_mismatch(en_text, 'en'):
        en_issues.append('charset_mismatch')

    # identity
    if detect_identity(ru_text, jp_text):
        ru_issues.append('identity')
    if detect_identity(en_text, jp_text):
        en_issues.append('identity')

    # CRITICAL: repetition_collapse in BOTH translations
    ru_has_rep = 'repetition_collapse' in ru_issues
    en_has_rep = 'repetition_collapse' in en_issues

    if ru_has_rep and en_has_rep:
        return 'critical', ru_issues, en_issues

    # WARNING: any other issues
    if ru_issues or en_issues:
        return 'warning', ru_issues, en_issues

    return 'ok', [], []


# ── Chunk processing ───────────────────────────────────────────────

def process_chunk(path: Path, chunk_num: int) -> tuple[dict, int]:
    """
    Process a single chunk file. Returns a tuple of:
      - result dict with 'critical' and 'warning' lists
      - phrase_count (int)
    """
    result = {
        'critical': [],
        'warning': [],
    }

    try:
        with open(path, encoding='utf-8') as f:
            phrases = json.load(f)
    except (json.JSONDecodeError, OSError) as exc:
        print(f"Warning: skipping {path} — {exc}")
        return result, 0

    for phrase in phrases:
        classification, ru_issues, en_issues = classify_phrase(phrase)

        if classification == 'critical':
            all_reasons = sorted(set(ru_issues + en_issues))
            result['critical'].append((phrase['i'], all_reasons))
        elif classification == 'warning':
            if ru_issues:
                result['warning'].append((phrase['i'], ru_issues, 'ru'))
            if en_issues:
                result['warning'].append((phrase['i'], en_issues, 'en'))

    return result, len(phrases)


# ── Report building ────────────────────────────────────────────────

def build_report(
    chunk_results: dict[int, dict],
    total_phrases: int,
) -> dict:
    """Aggregate chunk results into the final JSON report."""
    critical_count = 0
    warning_count = 0

    invalid_ids: list[str] = []
    invalid_details: list[dict] = []
    warning_ids: list[str] = []
    warning_details: list[dict] = []

    for chunk_num in sorted(chunk_results.keys()):
        for phrase_id, reasons in chunk_results[chunk_num]['critical']:
            critical_count += 1
            invalid_ids.append(phrase_id)
            invalid_details.append({
                'i': phrase_id,
                'chunk': chunk_num,
                'reasons': reasons,
            })

        for phrase_id, reasons, lang in chunk_results[chunk_num]['warning']:
            warning_count += 1
            warning_ids.append(phrase_id)
            warning_details.append({
                'i': phrase_id,
                'chunk': chunk_num,
                'reasons': reasons,
                'lang': lang,
            })

    ok_count = total_phrases - critical_count - warning_count

    return {
        'generated_at': datetime.now(timezone.utc).isoformat(),
        'script_version': '1.4',
        'summary': {
            'total_phrases': total_phrases,
            'critical': critical_count,
            'warning': warning_count,
            'ok': ok_count,
        },
        'invalid_phrase_ids': invalid_ids,
        'invalid_phrases_details': invalid_details,
        'warning_phrase_ids': warning_ids,
        'warning_phrases_details': warning_details,
    }


# ── CLI ────────────────────────────────────────────────────────────

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description='Detect ML garbage in phrase translation chunks',
    )
    parser.add_argument(
        '--input',
        required=True,
        help='Path to directory with chunk files (p*.json)',
    )
    parser.add_argument(
        '--output',
        default='garbage_report.json',
        help='Path to output JSON report (default: garbage_report.json)',
    )
    parser.add_argument(
        '--chunk',
        type=int,
        default=None,
        help='Process only a specific chunk number',
    )
    parser.add_argument(
        '--verbose',
        action='store_true',
        help='Print details for each CRITICAL/WARNING phrase',
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    input_dir = Path(args.input)
    if not input_dir.is_dir():
        print(f"Error: input directory not found: {input_dir}")
        return

    chunk_files = sorted(input_dir.glob('p*.json'))
    if not chunk_files:
        print(f"Error: no chunk files (p*.json) found in {input_dir}")
        return

    # Filter to a single chunk if requested
    if args.chunk is not None:
        target_name = f'p{args.chunk:04d}.json'
        chunk_files = [p for p in chunk_files if p.name == target_name]
        if not chunk_files:
            print(f"Error: chunk {args.chunk} not found (expected: {target_name})")
            return

    print(f"Processing {len(chunk_files)} chunk file(s)...")

    chunk_results: dict[int, dict] = {}
    total_phrases = 0

    for chunk_path in chunk_files:
        try:
            chunk_num = int(chunk_path.stem[1:])
        except ValueError:
            print(f"Warning: skipping file with unexpected name: {chunk_path.name}")
            continue

        result, phrase_count = process_chunk(chunk_path, chunk_num)
        chunk_results[chunk_num] = result
        total_phrases += phrase_count

        if args.verbose:
            crit_count = len(result['critical'])
            warn_count = len(result['warning'])
            if crit_count > 0 or warn_count > 0:
                print(f"\n[{chunk_path.name}] CRITICAL: {crit_count}, WARNING: {warn_count}")

                for phrase_id, reasons in result['critical']:
                    print(f"  CRITICAL {phrase_id}: {', '.join(reasons)}")

                for phrase_id, reasons, lang in result['warning']:
                    print(f"  WARNING  {phrase_id} [{lang}]: {', '.join(reasons)}")

        if phrase_count > 0:
            crit = len(result['critical'])
            warn = len(result['warning'])
            pct = (crit / phrase_count) * 100 if crit > 0 else 0
            print(f"  {chunk_path.name}: {phrase_count} phrases, "
                  f"{crit} critical ({pct:.1f}%), {warn} warning")

    report = build_report(chunk_results, total_phrases)

    output_path = Path(args.output)
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(report, f, ensure_ascii=False, indent=2)

    s = report['summary']
    print(f"\nReport saved to: {output_path}")
    print(f"Total: {s['total_phrases']} phrases")
    print(f"  Critical: {s['critical']} (both translations bad — in invalid_phrase_ids)")
    print(f"  Warning:  {s['warning']} (one translation bad — in warning_phrase_ids)")
    print(f"  OK:       {s['ok']}")


if __name__ == '__main__':
    main()
