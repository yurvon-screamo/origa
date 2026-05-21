#!/usr/bin/env python3
"""
Clean vocabulary translations v2 — targeted cleanup.
Only removes > blocks containing CJK characters.
Keeps all clean > blocks (useful notes in target language).
Also fixes Chinese contamination and CJK garbage in specific entries.

Usage:
    python scripts/clean_vocabulary_v2.py                # Process all chunks
    python scripts/clean_vocabulary_v2.py --dry-run       # Preview only
    python scripts/clean_vocabulary_v2.py --chunk 3       # Process chunk_03 only
    python scripts/clean_vocabulary_v2.py --verbose       # Show sample changes
"""

import json
import os
import re
import sys
import tempfile
import argparse
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
CHUNKS_DIR = PROJECT_ROOT / "cdn" / "dictionary"

CJK_REGEX = re.compile(r"[\u3040-\u309F\u30A0-\u30FF\u4E00-\u9FFF\u3400-\u4DBF]")


# fmt: off
# ---------------------------------------------------------------------------
# PATCHES: specific fixes for Chinese contamination in RU (38),
# CJK garbage in EN (16), and manual fixes (3).
# Keys are Japanese words; values are dicts of {field: corrected_text}.
# These patches OVERRIDE whatever the > block removal produced.
# ---------------------------------------------------------------------------

PATCHES: dict[str, dict[str, str]] = {
    # === Task 1: Chinese contamination in russian_translation (38 entries) ===

    "いっそ": {
        "russian_translation": "- лучше уж",
    },
    "アウトラインプロセッサ": {
        "russian_translation": (
            "- программное обеспечение для работы с иерархической структурой документа\n"
            "- инструмент для создания и редактирования черновиков с использованием оглавления"
        ),
    },
    "七夕": {
        "russian_translation": (
            "- день семи звёзд (японский праздник)\n"
            "- праздник ткачихи (Веги) и пастуха (Альтаира)"
        ),
    },
    "七夕祭り": {
        "russian_translation": (
            "- фестиваль Танабата (7-е число 7-го лунного месяца)\n"
            "- фестиваль в честь праздника девы"
        ),
    },
    "予告": {
        "russian_translation": (
            "- предупреждение\n"
            "- анонс"
        ),
    },
    "商機": {
        "russian_translation": (
            "- возможность для бизнеса\n"
            "- благоприятный момент (для сделки)"
        ),
    },
    "回りまわって": {
        "russian_translation": (
            "- в конечном итоге\n"
            "- через цепь событий\n"
            "- путём долгих окольных путей"
        ),
    },
    "売り込む": {
        "russian_translation": (
            "- активно продавать (навязывать товары или услуги)\n"
            "- вовлекать в продажу (привлекать клиента к покупке через настойчивые действия)"
        ),
    },
    "多かれ少なかれ": {
        "russian_translation": (
            "- в той или иной степени\n"
            "- так или иначе"
        ),
    },
    "広々とした": {
        "russian_translation": "- просторный",
    },
    "成り果てる": {
        "russian_translation": (
            "- опасть (до такого состояния)\n"
            "- прийти к плачевному финалу\n"
            "- разрушиться до крайности"
        ),
    },
    "採光": {
        "russian_translation": "- освещение (естественное, дневное)",
    },
    "書き言葉": {
        "russian_translation": (
            "- письменная речь\n"
            "- письменный язык (стиль)"
        ),
    },
    "未決定": {
        "russian_translation": (
            "- нерешённый\n"
            "- неопределённый"
        ),
    },
    "潰し値": {
        "russian_translation": (
            "- демпинг (продажа товаров ниже себестоимости для уничтожения конкурентов)\n"
            "- снижение цен (агрессивное занижение рыночных цен)"
        ),
    },
    "無計画": {
        "russian_translation": (
            "- без плана\n"
            "- неорганизованный"
        ),
    },
    "特長": {
        "russian_translation": (
            "- преимущество\n"
            "- особые способности"
        ),
    },
    "独り言": {
        "russian_translation": "- бормотание (вслух)",
    },
    "生疵": {
        "russian_translation": (
            "- дефект\n"
            "- шрам\n"
            "- пятно"
        ),
    },
    "痰壺": {
        "russian_translation": "- сплювотница",
    },
    "白露": {
        "russian_translation": (
            "- белые росы (15-й сезон)\n"
            "- название осеннего месяца в календаре"
        ),
    },
    "石こう": {
        "russian_translation": "- гипс",
    },
    "端緒": {
        "russian_translation": (
            "- начало (дела, разговора)\n"
            "- зацепка, зачаток"
        ),
    },
    "精華": {
        "russian_translation": "- экстракт, концентрат (сущность или суть явления)",
    },
    "精髄": {
        "russian_translation": (
            "- сущность\n"
            "- суть"
        ),
    },
    "織り女": {
        "russian_translation": (
            "- ткачиха (женщина, профессионально занимающаяся ткачеством)\n"
            "- в мифологическом контексте: персонаж, плетущий облака или ткани небес "
            "(например, Химэ в японской легенде о Ткачихе и Пастухе)"
        ),
    },
    "言い残す": {
        "russian_translation": "- оставить невысказанным",
    },
    "賢妻": {
        "russian_translation": "- верная спутница жизни",
    },
    "適度": {
        "russian_translation": (
            "- умеренный\n"
            "- в меру\n"
            "- справедливый (в контексте распределения или оценки)"
        ),
    },
    "適量": {
        "russian_translation": "- необходимое количество",
    },
    "間欠泉": {
        "russian_translation": "- гейзер",
    },
    "間歇泉": {
        "russian_translation": (
            "- гейзер\n"
            "- периодический источник (термальный)"
        ),
    },
    "障り": {
        "russian_translation": (
            "- препятствие\n"
            "- помеха"
        ),
    },
    "雑報欄": {
        "russian_translation": (
            "- раздел кратких новостей, заметок и объявлений\n"
            "- колонка miscellaneous (специальная рубрика в газете или журнале)"
        ),
    },
    "雨水": {
        "russian_translation": (
            "- вода, стекающая с неба (дождевая вода)\n"
            "- 2-й сезон (праздник в календаре), отмечающий начало сезона дождей"
        ),
    },
    "食べごろ": {
        "russian_translation": (
            "- в стадии зрелости (о еде)\n"
            "- в момент наилучшей готовности к употреблению"
        ),
    },
    "食べ頃": {
        "russian_translation": (
            "- пик вкусовых качеств\n"
            "- время лучшего употребления (для еды)"
        ),
    },
    "髄": {
        "russian_translation": (
            "- мозг (в переносном смысле: суть)\n"
            "- суть, самое важное"
        ),
    },

    # === Task 2: CJK garbage in english_translation (16 entries) ===

    "劇": {
        "english_translation": (
            "- drama (theatrical performance)\n"
            "- play (script)"
        ),
    },
    "座右": {
        "english_translation": "- place of honor (e.g., in a study or office)",
    },
    "慰問袋": {
        "english_translation": (
            "- care package (sent to hospitalized, evacuated, or deployed personnel)"
        ),
    },
    "戊": {
        "english_translation": (
            "- the 5th Heavenly Stem (used in the sexagenary cycle)\n"
            "- the 5th day of a 10-day period"
        ),
    },
    "暈": {
        "english_translation": (
            "- halo (atmospheric optical phenomenon)\n"
            "- aura (glow or radiance)"
        ),
    },
    "槻の木": {
        "english_translation": (
            "- a type of Japanese horse chestnut tree (Aesculus turbinata)"
        ),
    },
    "氣儘": {
        "english_translation": (
            "- as one pleases\n"
            "- at one's own discretion\n"
            "- self-indulgent"
        ),
    },
    "牢": {
        "english_translation": (
            "- prison\n"
            "- jail"
        ),
    },
    "目のあたり": {
        "english_translation": (
            "- before one's eyes\n"
            "- direct sight"
        ),
    },
    "目の当り": {
        "english_translation": (
            "- before one's eyes\n"
            "- direct experience"
        ),
    },
    "眼の当たり": {
        "english_translation": "- before one's eyes",
    },
    "眼の当り": {
        "english_translation": "- before one's eyes",
    },
    "精": {
        "english_translation": (
            "- essence; vital energy (e.g., in traditional medicine or philosophy)\n"
            "- refined; pure; select (e.g., refined sugar, elite students)\n"
            "- spirit; mind; consciousness (e.g., mental state, focus)"
        ),
    },
    "蚕室": {
        "english_translation": (
            "- a place where silkworms are raised\n"
            "- a district in Seoul, South Korea, named after the historical silkworm breeding site"
        ),
    },
    "透かし絵": {
        "english_translation": (
            "- watermark (security feature in paper/currency)"
        ),
    },
    "鋒鋩": {
        "english_translation": (
            "- edge of a sword\n"
            "- sharpness or keenness (of wit, talent, or spirit)"
        ),
    },

    # === Manual fixes (3 entries) ===

    "なま": {
        "russian_translation": (
            "- сырой (о еде, особенно рыбе)\n"
            "- незрелый (о фруктах)\n"
            "- непереработанный (о материалах, данных)\n"
            "- прямой (эфир)\n"
            "- неполный (о работе, строительстве)"
        ),
    },
    "ヤバい": {
        "russian_translation": "опасный; ужасный; крутой; офигенный (сленг)",
    },
    "黒んぼ": {
        "russian_translation": (
            "- темнокожий человек (устаревшее)\n"
            "- человек с сильным загаром\n"
            "- закулисный рабочий"
        ),
    },
}
# fmt: on


def has_cjk(text: str) -> bool:
    return bool(CJK_REGEX.search(text))


def clean_translation_selective(
    text: str,
) -> tuple[str, bool, str, str]:
    """Remove > blocks ONLY if they contain CJK characters.

    Keep > blocks that are purely in the target language.

    Returns:
        (cleaned_text, was_modified, removed_preview, kept_preview)
    """
    if ">" not in text:
        return text, False, "", ""

    lines = text.split("\n")

    # Find the first > line (allowing leading whitespace)
    blockquote_start = None
    for i, line in enumerate(lines):
        if line.lstrip().startswith(">"):
            blockquote_start = i
            break

    if blockquote_start is None:
        return text, False, "", ""

    # Collect the entire > block (all consecutive > lines and blank lines after)
    block_lines = lines[blockquote_start:]
    block_text = "\n".join(block_lines)

    if not has_cjk(block_text):
        # Block is CLEAN — no CJK, keep it
        return text, False, "", block_text[:100]

    # Block HAS CJK — remove it
    kept_lines = lines[:blockquote_start]

    # Remove trailing blank lines before the block
    while kept_lines and not kept_lines[-1].strip():
        kept_lines.pop()

    cleaned = "\n".join(kept_lines)
    return cleaned, True, block_text[:200], ""


def write_atomic(data: dict, target_path: Path) -> None:
    """Write JSON data atomically using temp file + os.replace."""
    json_str = json.dumps(data, ensure_ascii=False, indent=2)

    try:
        json.loads(json_str)
    except json.JSONDecodeError as e:
        print(f"  ERROR: generated JSON is invalid: {e}", file=sys.stderr)
        sys.exit(1)

    tmp_path = target_path.with_suffix(".json.tmp")
    try:
        with open(tmp_path, "w", encoding="utf-8") as f:
            f.write(json_str)
            f.write("\n")

        with open(tmp_path, "r", encoding="utf-8") as f:
            json.load(f)

        os.replace(tmp_path, target_path)
        size = os.path.getsize(target_path)
        print(f"  Written {target_path.name} ({size:,} bytes, valid JSON)")
    except Exception:
        if tmp_path.exists():
            tmp_path.unlink()
        raise


def process_chunk(
    chunk_path: Path, dry_run: bool, verbose: bool
) -> dict:
    """Process a single chunk file. Returns statistics."""
    stats = {
        "total": 0,
        "gt_removed_has_cjk": 0,
        "gt_kept_clean": 0,
        "patches_applied": 0,
        "not_modified": 0,
    }
    sample_changes: list[dict] = []

    with open(chunk_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    stats["total"] = len(data)
    modified_count = 0

    for word, entry in data.items():
        entry_modified = False

        # Step A: Selective > block removal
        for field in ("russian_translation", "english_translation"):
            original = entry.get(field, "")
            if not original:
                continue

            cleaned, was_modified, removed, kept = clean_translation_selective(
                original
            )

            if was_modified:
                entry_modified = True
                stats["gt_removed_has_cjk"] += 1
                if verbose and len(sample_changes) < 20:
                    sample_changes.append(
                        {
                            "word": word,
                            "field": field,
                            "action": "REMOVED > block with CJK",
                            "before": original[:200],
                            "after": cleaned[:200],
                            "removed_preview": removed[:150],
                        }
                    )
                entry[field] = cleaned
            elif kept:
                stats["gt_kept_clean"] += 1

        # Step B: Apply specific patches
        if word in PATCHES:
            patch = PATCHES[word]
            for field, new_value in patch.items():
                current = entry.get(field, "")
                if current != new_value:
                    entry_modified = True
                    stats["patches_applied"] += 1
                    if verbose and len(sample_changes) < 20:
                        sample_changes.append(
                            {
                                "word": word,
                                "field": field,
                                "action": "PATCH applied",
                                "before": current[:150],
                                "after": new_value[:150],
                            }
                        )
                    entry[field] = new_value

        if entry_modified:
            modified_count += 1
        else:
            stats["not_modified"] += 1

    # Print chunk stats
    print(f"\n{'=' * 60}")
    print(f"Processing: {chunk_path.name}")
    print(f"{'=' * 60}")
    print(f"  Entries: {stats['total']}")
    print(f"  Modified: {modified_count}")
    print(f"  > blocks removed (had CJK): {stats['gt_removed_has_cjk']}")
    print(f"  > blocks kept (clean): {stats['gt_kept_clean']}")
    print(f"  Specific patches applied: {stats['patches_applied']}")

    if verbose and sample_changes:
        print(f"\n  SAMPLE CHANGES:")
        for c in sample_changes[:15]:
            print(f"    [{c['action']}] {c['word']} ({c['field']})")
            print(f"      BEFORE: {c['before'][:120]}")
            print(f"      AFTER:  {c.get('after', '')[:120]}")
            if c.get("removed_preview"):
                print(f"      REMOVED: {c['removed_preview'][:120]}")
            print()

    # Write
    if not dry_run and modified_count > 0:
        write_atomic(data, chunk_path)
    elif dry_run and modified_count > 0:
        print(f"  [DRY RUN] Would modify {modified_count} entries")
    else:
        print(f"  No changes needed")

    stats["modified_count"] = modified_count
    return stats


def get_chunk_paths(args: argparse.Namespace) -> list[Path]:
    """Resolve which chunk files to process based on arguments."""
    if args.chunk is not None:
        chunk_num = args.chunk
        if not 1 <= chunk_num <= 11:
            print(
                f"ERROR: chunk number must be 1-11, got {chunk_num}",
                file=sys.stderr,
            )
            sys.exit(1)
        path = CHUNKS_DIR / f"chunk_{chunk_num:02d}.json"
        if not path.exists():
            print(f"ERROR: {path} not found", file=sys.stderr)
            sys.exit(1)
        return [path]

    paths = sorted(CHUNKS_DIR.glob("chunk_*.json"))
    if not paths:
        print(
            f"ERROR: no chunk_*.json files in {CHUNKS_DIR}",
            file=sys.stderr,
        )
        sys.exit(1)
    return paths


def print_summary(all_stats: list[dict], dry_run: bool) -> None:
    """Print overall summary across all processed chunks."""
    print(f"\n{'=' * 60}")
    mode = "DRY RUN — " if dry_run else ""
    print(f"{mode}SUMMARY")
    print(f"{'=' * 60}")

    total_entries = sum(s["total"] for s in all_stats)
    total_modified = sum(s["modified_count"] for s in all_stats)
    total_gt_removed = sum(s["gt_removed_has_cjk"] for s in all_stats)
    total_gt_kept = sum(s["gt_kept_clean"] for s in all_stats)
    total_patches = sum(s["patches_applied"] for s in all_stats)

    print(f"  Total entries:                  {total_entries}")
    print(f"  Total modified entries:         {total_modified}")
    print(f"  > blocks removed (had CJK):     {total_gt_removed}")
    print(f"  > blocks kept (clean):          {total_gt_kept}")
    print(f"  Specific patches applied:       {total_patches}")

    pct = total_modified / total_entries * 100 if total_entries else 0
    print(f"  Modification rate:              {pct:.1f}%")

    print(f"\n  Per-chunk breakdown:")
    header = f"  {'Chunk':<14s} {'Entries':>8s} {'Modified':>10s} {'>Removed':>10s} {'>Kept':>10s} {'Patches':>10s}"
    print(header)
    print(f"  {'-' * 66}")
    for s in all_stats:
        chunk_name = s.get("chunk_name", "?")
        print(
            f"  {chunk_name:<14s} {s['total']:>8d} "
            f"{s['modified_count']:>10d} {s['gt_removed_has_cjk']:>10d} "
            f"{s['gt_kept_clean']:>10d} {s['patches_applied']:>10d}"
        )


def main() -> None:
    parser = argparse.ArgumentParser(
        description=(
            "Clean vocabulary translations v2 — "
            "selective > block removal (only CJK ones) + specific patches"
        )
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview changes without modifying files",
    )
    parser.add_argument(
        "--chunk",
        type=int,
        help="Process only specific chunk number (1-11)",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Show sample before/after changes",
    )
    args = parser.parse_args()

    if not CHUNKS_DIR.exists():
        print(
            f"ERROR: chunks directory not found: {CHUNKS_DIR}",
            file=sys.stderr,
        )
        sys.exit(1)

    chunk_paths = get_chunk_paths(args)

    print("clean_vocabulary_v2.py")
    print(f"Chunks directory: {CHUNKS_DIR}")
    print(f"Files to process: {len(chunk_paths)}")
    print(f"Patches defined: {len(PATCHES)}")
    if args.dry_run:
        print("Mode: DRY RUN (no files will be modified)")

    all_stats: list[dict] = []

    for chunk_path in chunk_paths:
        stats = process_chunk(chunk_path, args.dry_run, args.verbose)
        stats["chunk_name"] = chunk_path.name
        all_stats.append(stats)

    print_summary(all_stats, args.dry_run)


if __name__ == "__main__":
    main()
