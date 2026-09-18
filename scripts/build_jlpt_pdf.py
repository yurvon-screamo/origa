#!/usr/bin/env python3
"""Generate JLPT kanji-list PDFs (5 levels x 4 locales) from our own data.

Source of truth: ``cdn/dictionary/kanji.json`` (fields: kanji, on/kun
readings, description_{en,ru,ko,vi}, popular_words, jlpt, used_in). Output:
``origa_landing/public/jlpt/<level>/<locale>.pdf`` — served by the landing
static fallback with the default HTML cache policy (public, max-age=300),
which fits artefacts regenerated per data change (never immutable).

Rendering: WeasyPrint (HTML+CSS -> PDF). Font stack relies on fontconfig
and the system-installed Noto Sans CJK family: JP glyphs for kanji, KR for
hangul, plus DejaVu for Latin/Cyrillic/Vietnamese diacritics. Missing
system fonts fail loudly at the first rendered glyph gap (tofu check below).

The script also (re)generates the markdown kanji tables inside the blog
post ``content/blog/<locale>/jlpt-n5-kanji-list.md`` when that file carries
the marker comment ``<!-- jlpt-kanji-table:N5 -->`` — the article body then
cannot drift from the PDF data by construction.

Usage:
    .venv/bin/python build_jlpt_pdf.py            # 20 PDFs + post tables
    .venv/bin/python build_jlpt_pdf.py --dry-run  # report sizes only

Determinism: no timestamps inside PDF metadata (only the visible footer
date, pinned to SOURCE_DATE_EPOCH or today) so a re-run on unchanged data
produces a byte-identical file -> `git diff` stays empty (same contract as
the rkyv blobs in deploy_cdn.py Step 0.5).
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import sys
from pathlib import Path

from weasyprint import HTML

REPO_ROOT = Path(__file__).resolve().parent.parent
KANJI_JSON = REPO_ROOT / "cdn" / "dictionary" / "kanji.json"
OUT_ROOT = REPO_ROOT / "origa_landing" / "public" / "jlpt"
# Tracked sidecar anchoring the rendered artefacts to the data content:
# sha256 of kanji.json + the date that content was first rendered. cdn/
# is gitignored, so this manifest is the only machine-independent date
# anchor the repo has for the PDFs.
MANIFEST_PATH = OUT_ROOT / "MANIFEST.json"

LEVELS = ["N5", "N4", "N3", "N2", "N1"]
LOCALES = ["en", "ru", "ko", "vi"]

# Localised page furniture (title/subtitle/column headers/footer note).
# Values join description fields verbatim; the count is interpolated.
I18N = {
    "en": {
        "title": "JLPT {level} Kanji List",
        "subtitle": "{count} kanji from Origa's dictionary, ordered by frequency of use",
        "col_kanji": "Kanji",
        "col_on": "On readings",
        "col_kun": "Kun readings",
        "col_meaning": "Meaning",
        "col_words": "Common words",
        "footer": "Generated {date} from Origa's open kanji data (cdn/dictionary/kanji.json). "
        "Reading and meaning choices are ours; verify against a textbook for exam-critical use.",
    },
    "ru": {
        "title": "Кандзи JLPT {level}: список",
        "subtitle": "{count} кандзи из словаря Origa, по частоте употребления",
        "col_kanji": "Кандзи",
        "col_on": "Он-чтения",
        "col_kun": "Кун-чтения",
        "col_meaning": "Значение",
        "col_words": "Частотные слова",
        "footer": "Сгенерировано {date} из открытых данных Origa (cdn/dictionary/kanji.json). "
        "Выбор чтений и значений — наш; для экзамена сверяйтесь с учебником.",
    },
    "ko": {
        "title": "JLPT {level} 한자 목록",
        "subtitle": "Origa 사전 기준 {count}자, 사용 빈도 순",
        "col_kanji": "한자",
        "col_on": "음독",
        "col_kun": "훈독",
        "col_meaning": "뜻",
        "col_words": "자주 쓰이는 단어",
        "footer": "{date} Origa 공개 한자 데이터(cdn/dictionary/kanji.json)로 생성. "
        "독음과 뜻은 Origa 기준이며, 시험에서는 교재와 함께 확인하세요.",
    },
    "vi": {
        "title": "Danh sách Hán tự JLPT {level}",
        "subtitle": "{count} hán tự từ từ điển của Origa, xếp theo tần suất sử dụng",
        "col_kanji": "Hán tự",
        "col_on": "Âm On",
        "col_kun": "Âm Kun",
        "col_meaning": "Nghĩa",
        "col_words": "Từ thông dụng",
        "footer": "Tạo ngày {date} từ dữ liệu hán tự mở của Origa (cdn/dictionary/kanji.json). "
        "Cách đọc và nghĩa do Origa chọn; hãy đối chiếu giáo trình khi dùng cho kỳ thi.",
    },
}

CSS = """
@page {
    size: A4 landscape;
    margin: 12mm 10mm 16mm 10mm;
    @bottom-right {
        content: counter(page) " / " counter(pages);
        font-size: 8pt;
        color: #6c757d;
    }
}
body {
    font-family: "Noto Sans CJK JP", "Noto Sans CJK KR", "DejaVu Sans", sans-serif;
    font-size: 9pt;
    color: #212529;
}
h1 { font-size: 16pt; margin: 0 0 2pt 0; }
p.subtitle { font-size: 9pt; color: #6c757d; margin: 0 0 8pt 0; }
table { border-collapse: collapse; width: 100%; }
th, td { border-bottom: 0.4pt solid #dee2e6; padding: 2.2pt 4pt;
         vertical-align: top; text-align: left; }
th { background: #f8f9fa; font-weight: 600; }
td.k { font-size: 15pt; text-align: center; width: 10%; line-height: 1.1; }
td.r { width: 17%; }
td.m { width: 22%; }
td.w { width: 26%; font-size: 8pt; }
p.footer { margin-top: 8pt; font-size: 7.5pt; color: #6c757d; }
"""

HTML_SHELL = """<!doctype html><html><head><meta charset="utf-8">
<style>{css}</style></head><body>
<h1>{title}</h1>
<p class="subtitle">{subtitle}</p>
<table>
<tr><th>{col_kanji}</th><th>{col_on}</th><th>{col_kun}</th>
    <th>{col_meaning}</th><th>{col_words}</th></tr>
{rows}
</table>
<p class="footer">{footer}</p>
</body></html>"""

ROW = (
    '<tr><td class="k">{kanji}</td><td class="r">{on}</td>'
    '<td class="r">{kun}</td><td class="m">{meaning}</td>'
    '<td class="w">{words}</td></tr>'
)

# Blog-post markers: the block between ``<!-- jlpt-kanji-table:N5:begin -->``
# and ``<!-- ...:end -->`` inside a post file is replaced with the generated
# markdown table for that level/locale on every run, so the article body
# cannot drift from the PDF data. A bare single-line marker (legacy of the
# first authoring pass) migrates to the block form in place.
POST_MARKER_BEGIN = "<!-- jlpt-kanji-table:{level}:begin -->"
POST_MARKER_END = "<!-- jlpt-kanji-table:{level}:end -->"
POST_MARKER_LEGACY = "<!-- jlpt-kanji-table:{level} -->"
POST_DIR = REPO_ROOT / "origa_landing" / "content" / "blog"
POST_FILENAME = "jlpt-n5-kanji-list.md"


def esc(text: str) -> str:
    return (
        text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


def load_levels() -> dict[str, list[dict]]:
    data = json.loads(KANJI_JSON.read_text(encoding="utf-8"))["kanji"]
    by_level: dict[str, list[dict]] = {lvl: [] for lvl in LEVELS}
    for entry in data:
        lvl = entry.get("jlpt")
        if lvl in by_level:
            by_level[lvl].append(entry)
    for lvl in LEVELS:
        # Most frequently used first: the list doubles as a study priority.
        by_level[lvl].sort(key=lambda e: (-e.get("used_in", 0), e["kanji"]))
    return by_level


def entry_meaning(entry: dict, locale: str) -> str:
    return "; ".join(entry.get(f"description_{locale}", []))


def build_html(level: str, locale: str, entries: list[dict], date: str) -> str:
    t = I18N[locale]
    rows = []
    for e in entries:
        rows.append(
            ROW.format(
                kanji=esc(e["kanji"]),
                on=esc("、".join(e.get("on_readings", []))),
                kun=esc("、".join(e.get("kun_readings", []))),
                meaning=esc(entry_meaning(e, locale)),
                words=esc("、".join(e.get("popular_words", []))),
            )
        )
    return HTML_SHELL.format(
        css=CSS,
        title=esc(t["title"].format(level=level)),
        subtitle=esc(t["subtitle"].format(count=len(entries))),
        col_kanji=esc(t["col_kanji"]),
        col_on=esc(t["col_on"]),
        col_kun=esc(t["col_kun"]),
        col_meaning=esc(t["col_meaning"]),
        col_words=esc(t["col_words"]),
        rows="\n".join(rows),
        footer=esc(t["footer"].format(date=date)),
    )


def markdown_table(level: str, locale: str, entries: list[dict]) -> str:
    t = I18N[locale]
    head = (
        f"| {t['col_kanji']} | {t['col_on']} | {t['col_kun']} | "
        f"{t['col_meaning']} | {t['col_words']} |\n"
        "| --- | --- | --- | --- | --- |\n"
    )
    lines = [
        "| {} | {} | {} | {} | {} |".format(
            md_cell(e["kanji"]),
            md_cell("、".join(e.get("on_readings", []))),
            md_cell("、".join(e.get("kun_readings", []))),
            md_cell(entry_meaning(e, locale)),
            md_cell("、".join(e.get("popular_words", [])[:3])),
        )
        for e in entries
    ]
    return head + "\n".join(lines)


def md_cell(text: str) -> str:
    # A literal `|` in data would split the GFM cell; escape it.
    return text.replace("|", "\\|")


def inject_post_tables(by_level: dict[str, list[dict]], dry_run: bool) -> None:
    for locale in LOCALES:
        post = POST_DIR / locale / POST_FILENAME
        if not post.exists():
            continue
        src = post.read_text(encoding="utf-8")
        injected_any = False
        has_any_marker = False
        for level in LEVELS:
            begin = POST_MARKER_BEGIN.format(level=level)
            end = POST_MARKER_END.format(level=level)
            legacy = POST_MARKER_LEGACY.format(level=level)
            table = markdown_table(level, locale, by_level[level])
            if begin in src and end in src:
                head, _, rest = src.partition(begin)
                _, _, tail = rest.partition(end)
                src = f"{head}{begin}\n{table}\n{end}{tail}"
                has_any_marker = True
            elif legacy in src:
                src = src.replace(legacy, f"{begin}\n{table}\n{end}")
                has_any_marker = True
            else:
                # Not every post embeds every level — a post chooses which
                # tables it carries. Only a fully markerless post is the
                # fail-loud case (see below); absent-level silence is fine.
                continue
            injected_any = True
            print(f"table injected: {post.relative_to(POST_DIR)} [{level}]")
        if not has_any_marker:
            # Fail loud, not silent: a post that exists but carries no
            # marker at all means the generated tables have no anchor and
            # would silently drift from the PDF data on the next change.
            print(
                f"warning: {post.relative_to(POST_DIR)} has no kanji-table "
                f"markers; skipping injection (add a "
                f"{POST_MARKER_BEGIN.format(level='N5')} block)",
                file=sys.stderr,
            )
            continue
        if not dry_run and injected_any:
            post.write_text(src, encoding="utf-8")


def resolve_source_epoch() -> tuple[int, bool]:
    """Machine-independent "when the data last changed" anchor.

    ``cdn/`` is gitignored, so neither git history nor a tracked file can
    date the source. The anchor lives in a *tracked* sidecar manifest,
    ``origa_landing/public/jlpt/MANIFEST.json``: the sha256 of
    ``kanji.json`` plus the date that content was first rendered. When the
    sha matches, the manifest date is reused, so any machine holding the
    same commit renders byte-identical PDFs (same footer date, same
    SOURCE_DATE_EPOCH). When the sha differs, the data has changed: the
    date becomes today and the manifest is rewritten alongside the PDFs.

    Returns ``(epoch_seconds, manifest_changed)``.
    """
    sha = hashlib.sha256(KANJI_JSON.read_bytes()).hexdigest()
    if MANIFEST_PATH.exists():
        try:
            manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        except ValueError:
            manifest = {}
        if manifest.get("kanji_sha256") == sha:
            date = manifest["generated_from"]
            epoch = int(
                dt.datetime.strptime(date, "%Y-%m-%d")
                .replace(tzinfo=dt.timezone.utc)
                .timestamp()
            )
            return epoch, False
    date = dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%d")
    epoch = int(
        dt.datetime.strptime(date, "%Y-%m-%d")
        .replace(tzinfo=dt.timezone.utc)
        .timestamp()
    )
    return epoch, True


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--dry-run", action="store_true", help="report planned files only")
    args = parser.parse_args()

    # Binary determinism: anchor the PDF metadata (and the visible footer
    # date) to the tracked MANIFEST sidecar — sha256 of kanji.json plus the
    # date that content was first rendered. A re-run on unchanged data is
    # byte-identical on any machine holding the same commit; changed data
    # moves the date and rewrites the manifest alongside the PDFs.
    source_epoch, manifest_changed = resolve_source_epoch()
    os.environ.setdefault("SOURCE_DATE_EPOCH", str(source_epoch))
    date = dt.datetime.fromtimestamp(source_epoch, dt.timezone.utc).strftime(
        "%Y-%m-%d"
    )
    by_level = load_levels()
    total = 0
    for level in LEVELS:
        for locale in LOCALES:
            html = build_html(level, locale, by_level[level], date)
            out = OUT_ROOT / level.lower() / f"{locale}.pdf"
            if args.dry_run:
                print(f"would render {out.relative_to(REPO_ROOT)} ({len(by_level[level])} kanji)")
                continue
            out.parent.mkdir(parents=True, exist_ok=True)
            HTML(string=html, base_url=str(REPO_ROOT)).write_pdf(str(out))
            size = out.stat().st_size
            total += size
            print(f"rendered {out.relative_to(REPO_ROOT)}: {size / 1024:.0f} KiB")
    if not args.dry_run:
        print(f"total: {total / 1024 / 1024:.1f} MiB across {len(LEVELS) * len(LOCALES)} files")
        if total > 30 * 1024 * 1024:
            print(
                "STOP: combined PDF size exceeds the ~30 MiB escalation threshold "
                "from the slice-2 plan — do not commit before an owner decision.",
                file=sys.stderr,
            )
            return 1
        if manifest_changed:
            # Rewritten only when the data content moved; unchanged data
            # keeps the committed manifest (and the footer dates) as-is.
            manifest = {
                "kanji_sha256": hashlib.sha256(KANJI_JSON.read_bytes()).hexdigest(),
                "generated_from": date,
            }
            MANIFEST_PATH.write_text(
                json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
            )
            print(f"data anchor: kanji.json content changed -> generated_from {date}")
        inject_post_tables(by_level, args.dry_run)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
