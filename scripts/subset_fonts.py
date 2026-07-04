"""Generate self-hosted subset woff2 fonts for origa_ui.

Downloads pinned source fonts, extracts the CJK glyph corpus from the cdn/
content, subsets each font to the glyphs the app actually uses, and writes
content-hash-versioned woff2 files to cdn/fonts/. Hashing the filename gives
automatic CDN cache-bust on regeneration (PR #182 edge-poisoning class).

Run with fonttools available (woff2 flavor needs brotli):

    uv run --with "fonttools[woff]" scripts/subset_fonts.py

Outputs:
  - cdn/fonts/<logical>-<sha8>.woff2   (8 files)
  - end2end/cdn-manifest.txt            (FONTS-START/END block refreshed)

Source fonts are pinned by URL + SHA256; a mismatch aborts the run so an
upstream change forces a deliberate hash update (reproducibility, NFR-4).
"""

from __future__ import annotations

import hashlib
import subprocess
import sys
import zipfile
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
CDN_DIR = PROJECT_ROOT / "cdn"
FONTS_OUT = CDN_DIR / "fonts"
CACHE_DIR = PROJECT_ROOT / "scripts" / ".font_cache"
CDN_MANIFEST = PROJECT_ROOT / "end2end" / "cdn-manifest.txt"

LATIN_UNICODES = "U+0000-007F,U+00A0-00FF,U+2000-206F,U+20A0-20CF,U+2100-214F"
# Ranges included wholesale into the corpus (small, always needed).
WHOLESALE_RANGES: tuple[range, ...] = (
    range(0x3000, 0x3040),   # CJK symbols and punctuation
    range(0x3040, 0x3100),   # Hiragana + katakana
    range(0xFF00, 0xFFEF + 1),  # Halfwidth/Fullwidth forms
    range(0x0020, 0x007F),   # ASCII
)
# Kanji ranges: only codepoints actually present in cdn/ content are kept,
# otherwise the full ~20k ideograph block would defeat subsetting.
KANJI_RANGES: tuple[tuple[int, int], ...] = (
    (0x3400, 0x4DC0),   # CJK Unified Ideographs Extension A
    (0x4E00, 0x9FFF + 1),  # CJK Unified Ideographs
    (0xF900, 0xFAFF + 1),  # CJK Compatibility Ideographs
)


def _is_kanji(codepoint: int) -> bool:
    return any(start <= codepoint < end for start, end in KANJI_RANGES)


@dataclass(frozen=True)
class Source:
    logical: str
    url: str
    sha256: str
    kind: str  # "cjk" | "latin_static" | "latin_variable"
    archive: str | None = None
    extract_glob: str | None = None


SOURCES: tuple[Source, ...] = (
    Source(
        logical="noto-sans-jp-400",
        url="https://github.com/notofonts/noto-cjk/releases/download/Sans2.004/16_NotoSansJP.zip",
        sha256="2bbdd2c20f30670b39ca735c96d75f1fdabdb348103e43b820cf17701fd22b18",
        kind="cjk",
        archive="NotoSansJP.zip",
        extract_glob="NotoSansJP-Regular.otf",
    ),
    Source(
        logical="noto-serif-jp-400",
        url="https://github.com/notofonts/noto-cjk/releases/download/Serif2.003/12_NotoSerifJP.zip",
        sha256="53bdd2a6e4eb63bf24f7890e018dddb94366e3555d0814c72b74fbb128f328f0",
        kind="cjk",
        archive="NotoSerifJP.zip",
        extract_glob="**/NotoSerifJP-Regular.otf",
    ),
    Source(
        logical="cormorant-garamond",
        url="https://raw.githubusercontent.com/google/fonts/main/ofl/cormorantgaramond/CormorantGaramond%5Bwght%5D.ttf",
        sha256="b20b7d9626dd956b2c5e558692ad328b1f19e3275e2782db4fa07670d83f35e0",
        kind="latin_variable",
    ),
    Source(
        logical="cormorant-garamond-italic",
        url="https://raw.githubusercontent.com/google/fonts/main/ofl/cormorantgaramond/CormorantGaramond-Italic%5Bwght%5D.ttf",
        sha256="0f48ea6abb2084537854f7174c470991a463b13036309e3b50a81511611c530d",
        kind="latin_variable",
    ),
    Source(
        logical="dm-mono-300",
        url="https://raw.githubusercontent.com/google/fonts/main/ofl/dmmono/DMMono-Light.ttf",
        sha256="c7b3645dc8d28237317b4d017bc47b9ff09a7660758122dacb694a5a82552c24",
        kind="latin_static",
    ),
    Source(
        logical="dm-mono-400",
        url="https://raw.githubusercontent.com/google/fonts/main/ofl/dmmono/DMMono-Regular.ttf",
        sha256="55b4c98f123daebb3ed27947ba47b2af00554fc6284d639a540bcef5e6258ad2",
        kind="latin_static",
    ),
    Source(
        logical="dm-mono-500",
        url="https://raw.githubusercontent.com/google/fonts/main/ofl/dmmono/DMMono-Medium.ttf",
        sha256="fd327daf461db87b44a87def475d251bf03b997f7c07d9680592d75dbbfaad0b",
        kind="latin_static",
    ),
    Source(
        logical="dm-mono-400-italic",
        url="https://raw.githubusercontent.com/google/fonts/main/ofl/dmmono/DMMono-Italic.ttf",
        sha256="32b5bad9cbce64eac6d05c8abbeb619317f7e4cb354e1c33db761adbfaae1b16",
        kind="latin_static",
    ),
)


def sha256_file(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def download(url: str, dest: Path) -> None:
    print(f"  downloading {url}")
    subprocess.run(
        ["curl", "-sSL", "--fail", "-o", str(dest), url],
        check=True,
    )


def ensure_source(src: Source) -> Path:
    """Return the local path to the extracted/ready-to-subset source font."""
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    if src.archive:
        archive_path = CACHE_DIR / src.archive
        if not archive_path.is_file() or sha256_file(archive_path) != src.sha256:
            download(src.url, archive_path)
        actual = sha256_file(archive_path)
        if actual != src.sha256:
            sys.exit(f"SHA256 mismatch for {src.archive}: {actual} != {src.sha256}")
        return extract_from_zip(archive_path, src.extract_glob or "*.otf")
    else:
        filename = src.url.rsplit("/", 1)[-1]
        font_path = CACHE_DIR / filename
        if not font_path.is_file() or sha256_file(font_path) != src.sha256:
            download(src.url, font_path)
        actual = sha256_file(font_path)
        if actual != src.sha256:
            sys.exit(f"SHA256 mismatch for {filename}: {actual} != {src.sha256}")
        return font_path


def extract_from_zip(archive_path: Path, pattern: str) -> Path:
    with zipfile.ZipFile(archive_path) as zf:
        matches = [n for n in zf.namelist() if Path(n).match(pattern)]
        if not matches:
            sys.exit(f"No file matching {pattern} in {archive_path.name}")
        target = CACHE_DIR / Path(matches[0]).name
        target.write_bytes(zf.read(matches[0]))
        return target


# Files whose kanji are reference data (radical -> every kanji sharing it),
# not text the UI renders; including them would bloat the subset to ~12k glyphs.
CORPUS_EXCLUDE: frozenset[str] = frozenset({"dictionary/radicals.json"})


def extract_cjk_corpus() -> set[int]:
    """Build the glyph set for Noto subsetting.

    Wholesale ranges (kana, punctuation, ASCII) are included in full since they
    are small and always relevant. Kanji are added only when actually present in
    the cdn/ content, so the subset stays bounded to the app's real vocabulary.
    Reference-only files (see CORPUS_EXCLUDE) are skipped.
    """
    corpus: set[int] = set()
    for r in WHOLESALE_RANGES:
        corpus.update(r)

    for json_path in CDN_DIR.rglob("*.json"):
        rel = json_path.relative_to(CDN_DIR).as_posix()
        if rel in CORPUS_EXCLUDE:
            continue
        try:
            text = json_path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        for ch in text:
            if _is_kanji(ord(ch)):
                corpus.add(ord(ch))
    return corpus


def write_corpus_file(codepoints: Iterable[int]) -> Path:
    corpus_path = CACHE_DIR / "cjk_corpus.txt"
    chars = "".join(chr(c) for c in sorted(codepoints))
    corpus_path.write_text(chars, encoding="utf-8")
    return corpus_path


def subset_one(src_path: Path, src: Source, corpus_path: Path) -> Path:
    """Run pyftsubset and return the path to the content-hashed output."""
    raw_out = FONTS_OUT / f"{src.logical}.raw.woff2"
    cmd = [sys.executable, "-m", "fontTools.subset", str(src_path)]
    if src.kind == "cjk":
        cmd += ["--text-file=" + str(corpus_path)]
    else:
        cmd += ["--unicodes=" + LATIN_UNICODES]
    cmd += [
        "--flavor=woff2",
        "--layout-features=*",
        "--output-file=" + str(raw_out),
    ]
    subprocess.run(cmd, check=True)
    return raw_out


def hash_rename(raw_path: Path) -> Path:
    digest = sha256_file(raw_path)[:8]
    final = raw_path.with_name(f"{raw_path.name.removesuffix('.raw.woff2')}-{digest}.woff2")
    raw_path.replace(final)
    return final


def update_cdn_manifest(font_relative: Iterable[str]) -> None:
    start_marker = "# FONTS-START"
    end_marker = "# FONTS-END"
    original = (
        CDN_MANIFEST.read_text(encoding="utf-8").splitlines()
        if CDN_MANIFEST.is_file()
        else []
    )

    marker_count = sum(
        1 for line in original if line.strip() in (start_marker, end_marker)
    )
    if marker_count == 1:
        sys.exit(
            f"{CDN_MANIFEST} has a lone FONTS marker; remove the stray marker or "
            "pair START/END before regenerating."
        )

    outside: list[str] = []
    in_block = False
    for line in original:
        if line.strip() == start_marker:
            in_block = True
            continue
        if line.strip() == end_marker:
            in_block = False
            continue
        if not in_block:
            outside.append(line)

    block = [start_marker, *sorted(font_relative), end_marker]
    body = "\n".join(outside).rstrip()
    content = body + "\n\n" + "\n".join(block) + "\n" if body else "\n".join(block) + "\n"
    CDN_MANIFEST.write_text(content, encoding="utf-8")


def main() -> None:
    FONTS_OUT.mkdir(parents=True, exist_ok=True)
    for stale in FONTS_OUT.glob("*.woff2"):
        stale.unlink()

    corpus_path = write_corpus_file(extract_cjk_corpus())

    print("Subsetting fonts:")
    produced: list[str] = []
    for src in SOURCES:
        source_path = ensure_source(src)
        raw = subset_one(source_path, src, corpus_path)
        final = hash_rename(raw)
        size_kb = final.stat().st_size // 1024
        print(f"  {final.name}  ({size_kb} KB)")
        produced.append(f"fonts/{final.name}")

    update_cdn_manifest(produced)
    verify_glyph_coverage()
    print(f"\nDone: {len(produced)} fonts in {FONTS_OUT}")
    print(f"Corpus: {len(corpus_path.read_text(encoding='utf-8'))} glyphs")


# Glyphs the font set MUST render: the bug-report kanji (must use Japanese
# forms), representative kana, and CJK punctuation. A failed assertion means
# the corpus scan or subset dropped coverage the app depends on.
MUST_HAVE = "食海語難挨拶あア。、「」ー"


def verify_glyph_coverage() -> None:
    from fontTools.ttLib import TTFont

    noto = next(FONTS_OUT.glob("noto-serif-jp-400-*.woff2"))
    cmap = TTFont(noto).getBestCmap()
    missing = [c for c in MUST_HAVE if ord(c) not in cmap]
    if missing:
        sys.exit(f"Coverage regression: {missing} missing from {noto.name}")
    print(f"Coverage OK: {len(MUST_HAVE)} must-have glyphs present in {noto.name}")


if __name__ == "__main__":
    main()
