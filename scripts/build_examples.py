#!/usr/bin/env python3
"""Build cdn/examples/ — textbook example sentences for popular words.

Sources (issue #528):
  1. Tanaka Corpus / WWWJDIC examples.utf (~148k ja-en pairs, CC-BY) —
     B-lines carry a headword index used for word matching.
  2. Substring pass over A-line text for compound words the index misses.

Outputs (all under cdn/examples/):
  index.json        {"v","h","words": {word: {"refs": [[sid, start, end]..]}},
                     "s": sentence_count}
  data/sNNNN.json   [{"i","x","en","f"}] — deduped sentences, ~1000 per chunk,
                    translations ru/vi/ko merged later by merge_translations.py

Deterministic: sorted iteration, stable ids (sid = 0-based insertion order after
sorting sentences by id). Re-running with the same inputs yields identical bytes.

The word→sentence matching result is cached to /tmp/opencode/examples_matches.json
for the translation pipeline.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import urllib.request
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CDN = REPO / "cdn"
EXAMPLES_URL = "https://www.edrdg.org/pub/Nihongo/examples.utf.gz"
DEFAULT_WORK = Path("/tmp/opencode")

MAX_SENT_CHARS = 60          # textbook register: short, self-contained
MIN_SENT_CHARS = 6
MAX_PER_WORD = 3
CHUNK_SIZE = 1000            # sentences per data chunk
SENT_END = ("。", "？", "！", "！?", "？？")

RE_TOKEN_GROUPS = re.compile(r"[\(\{\[]")
# [M]/[F] gender tags and #ID= suffixes in Tanaka English translations
RE_GENDER_TAG = re.compile(r"\s*\[[MF]\]")
RE_ID_TAG = re.compile(r"#ID=\S+$")
RE_TRAIL_WS = re.compile(r"\s+")

KANA_RE = re.compile(r"[\u3040-\u30ff]")
KANJI_RE = re.compile(r"[\u4e00-\u9fff]")


def is_kana(s: str) -> bool:
    return bool(KANA_RE.search(s)) and not KANJI_RE.search(s)


def load_popular_words() -> dict[str, str]:
    """word -> JLPT level ('' when unknown). Union of all well_known_set files."""
    words: dict[str, str] = {}
    for f in sorted((CDN / "well_known_set").rglob("*.json")):
        try:
            d = json.loads(f.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError) as e:
            print(f"WARN: skipping unreadable set {f}: {e}", file=sys.stderr)
            continue
        if isinstance(d, dict) and isinstance(d.get("words"), list):
            level = d.get("level", "")
            for w in d["words"]:
                if isinstance(w, str) and w:
                    words.setdefault(w, level if isinstance(level, str) else "")
    return words


def load_furigana() -> tuple[dict[str, set[str]], dict[str, set[str]]]:
    """JmdictFurigana: (form->readings, reading->forms)."""
    fwd: dict[str, set[str]] = defaultdict(set)
    rev: dict[str, set[str]] = defaultdict(set)
    f = CDN / "dictionaries" / "JmdictFurigana.txt"
    with f.open(encoding="utf-8-sig") as fh:
        for line in fh:
            p = line.rstrip("\n").split("|")
            if len(p) >= 2 and p[0] and p[1]:
                fwd[p[0]].add(p[1])
                rev[p[1]].add(p[0])
    return fwd, rev


def bline_forms(line: str) -> set[str]:
    """Forms from one B-line: headwords + parenthesised readings + braced surfaces."""
    forms: set[str] = set()
    for tok in line.split():
        base = RE_TOKEN_GROUPS.split(tok)[0]
        if base:
            forms.add(base)
        for m in re.finditer(r"\(([^)]*)\)|\{([^}]*)\}", tok):
            v = m.group(1) or m.group(2)
            if v:
                forms.add(v)
    return forms


def bline_ruby(line: str) -> list[list[str]]:
    """[kanji, reading] pairs from one B-line (UI ruby seed)."""
    ruby: list[list[str]] = []
    for tok in line.split():
        base = RE_TOKEN_GROUPS.split(tok)[0]
        if not base or not (KANA_RE.search(base) or KANJI_RE.search(base)):
            continue
        m = re.search(r"\(([^)]*)\)", tok)
        if m:
            ruby.append([base, m.group(1)])
    return ruby


def clean_en(en: str) -> str:
    en = RE_ID_TAG.sub("", en)
    en = RE_GENDER_TAG.sub("", en)
    return RE_TRAIL_WS.sub(" ", en).strip()


def ensure_examples_file(path: Path) -> None:
    """Fetch the corpus into `path.parent` unless a fresh copy exists."""
    if path.exists() and path.stat().st_size > 1_000_000:
        return
    import gzip

    gz = WORK / "examples.utf.gz"
    if not gz.exists():
        print(f"downloading {EXAMPLES_URL} ...", flush=True)
        req = urllib.request.Request(EXAMPLES_URL, headers={"User-Agent": "origa-build"})
        with urllib.request.urlopen(req, timeout=60) as resp, gz.open("wb") as dst:
            while chunk := resp.read(1 << 20):
                dst.write(chunk)
    with gzip.open(gz, "rt", encoding="utf-8") as src, path.open("w", encoding="utf-8") as dst:
        while chunk := src.read(1 << 20):
            dst.write(chunk)


def parse_and_match(
    words: dict[str, str], examples_path: Path
) -> tuple[list[dict], dict[str, dict[int, list[int]]]]:
    """Parse examples.utf; return (sentences, matches).

    File layout: an A-line (sentence + English) is followed by its B-line
    (tokenised headword index). Matching therefore happens when the B-line
    of the *previous* A-line arrives.

    sentences: list of {"id": int, "x": str, "en": str, "f": [[surface, reading]..]}
    matches:   {word: {sid: [start, end]}} — sentence ids with word offsets.
    """
    fwd, rev = load_furigana()

    # form -> word candidates (index words may cover several popular words)
    form_to_word: dict[str, set[str]] = defaultdict(set)
    for w in words:
        form_to_word[w].add(w)
        for r in fwd.get(w, ()):  # kanji word -> readings
            form_to_word[r].add(w)
        if is_kana(w):
            for kanji_form in rev.get(w, ()):  # kana word -> kanji forms
                form_to_word[kanji_form].add(w)

    sentences: list[dict] = []
    matches: dict[str, dict[int, list[int]]] = defaultdict(dict)
    holes: set[str] = set(words)  # words without a match yet

    def match_sentence(sid: int, ja: str, forms: set[str]) -> set[str]:
        """Register matches for one sentence; returns the matched words."""
        hit_words: set[str] = set()

        # exact-form matches (index)
        for form in forms:
            for w in form_to_word.get(form, ()):
                hit_words.add(w)

        # substring matches for words still missing any match
        chars = set(ja)
        by_first: dict[str, list[str]] = defaultdict(list)
        for w in holes:
            if w not in hit_words and w[0] in chars:
                by_first[w[0]].append(w)
        for ws in by_first.values():
            for w in ws:
                idx = ja.find(w)
                if idx != -1:
                    matches[w][sid] = [idx, idx + len(w)]
                    hit_words.add(w)

        # offsets for index hits
        for w in hit_words:
            if sid in matches[w]:
                continue
            idx = ja.find(w)
            if idx != -1:
                matches[w][sid] = [idx, idx + len(w)]
            else:
                matches[w][sid] = [-1, -1]  # form differs from surface (kana variant)
        return hit_words

    pending: int | None = None  # sid awaiting its B-line
    with examples_path.open(encoding="utf-8") as fh:
        for raw in fh:
            if raw.startswith("A: "):
                a_part, _, en_part = raw[3:].partition("\t")
                ja = a_part.strip()
                en = clean_en(en_part.strip())
                ok = (
                    MIN_SENT_CHARS <= len(ja) <= MAX_SENT_CHARS
                    and ja.endswith(SENT_END)
                )
                if ok:
                    sid = len(sentences)
                    sentences.append({"id": sid, "x": ja, "en": en, "f": []})
                    pending = sid
                else:
                    pending = None
                continue
            if raw.startswith("B: ") and pending is not None:
                ja = sentences[pending]["x"]
                sentences[pending]["f"] = bline_ruby(raw[3:])
                holes -= match_sentence(pending, ja, bline_forms(raw[3:]))
                pending = None

    return sentences, matches


def select_for_words(
    sentences: list[dict],
    matches: dict[str, dict[int, list[int]]],
    words: dict[str, str],
) -> dict[str, list[dict]]:
    """Pick up to MAX_PER_WORD sentences per word: shortest first (textbook register)."""
    sel: dict[str, list[dict]] = {}
    for w in sorted(words):
        cand = matches.get(w, {})
        if not cand:
            continue
        ranked = sorted(
            cand.items(),
            key=lambda kv: (kv[1][0] == -1, len(sentences[kv[0]]["x"]), kv[0]),
        )[:MAX_PER_WORD]
        sel[w] = [
            {"sid": s, "start": off[0], "end": off[1]} for s, off in ranked
        ]
    return sel


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--examples",
        type=Path,
        default=None,
        help="path to the unpacked examples.utf (default: <work>/examples.utf)",
    )
    ap.add_argument(
        "--work",
        type=Path,
        default=DEFAULT_WORK,
        help="scratch dir for the downloaded corpus and the match cache",
    )
    args = ap.parse_args()
    global WORK
    WORK = args.work
    examples_path = args.examples or (WORK / "examples.utf")

    WORK.mkdir(parents=True, exist_ok=True)
    ensure_examples_file(args.examples)

    words = load_popular_words()
    print(f"popular words: {len(words)}", flush=True)

    sentences, matches = parse_and_match(words, examples_path)
    print(f"sentences parsed: {len(sentences)}; words matched: {len(matches)}", flush=True)

    sel = select_for_words(sentences, matches, words)
    covered = len(sel)
    full = sum(1 for v in sel.values() if len(v) >= 3)
    print(f"coverage: >=1: {covered}/{len(words)} | >=3: {full}", flush=True)

    used_sids = sorted({ref["sid"] for v in sel.values() for ref in v})
    sid_to_new = {sid: i for i, sid in enumerate(used_sids)}
    out_sentences = []
    for sid in used_sids:
        s = sentences[sid]
        out_sentences.append({"i": sid_to_new[sid], "x": s["x"], "en": s["en"], "f": s["f"]})

    out_words = {
        w: {
            "refs": [[sid_to_new[r["sid"]], r["start"], r["end"]] for r in sel[w]],
        }
        for w in sorted(sel)
    }

    out_dir = CDN / "examples"
    (out_dir / "data").mkdir(parents=True, exist_ok=True)
    for old in out_dir.glob("data/s*.json"):
        old.unlink()

    chunks = 0
    for start in range(0, len(out_sentences), CHUNK_SIZE):
        part = out_sentences[start : start + CHUNK_SIZE]
        p = out_dir / "data" / f"s{chunks:04d}.json"
        p.write_text(
            json.dumps(part, ensure_ascii=False, separators=(",", ":")), encoding="utf-8"
        )
        chunks += 1

    body = json.dumps(
        {"v": 1, "h": "", "words": out_words, "s": len(out_sentences)},
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    )
    h = hashlib.sha256(body.encode("utf-8")).hexdigest()
    index = json.loads(body)
    index["h"] = h
    (out_dir / "index.json").write_text(
        json.dumps(index, ensure_ascii=False, sort_keys=True, separators=(",", ":")),
        encoding="utf-8",
    )

    # cache matches for the translation pipeline
    (WORK / "examples_matches.json").write_text(
        json.dumps(
            {w: sel[w] for w in sorted(sel)}, ensure_ascii=False
        ),
        encoding="utf-8",
    )

    n_chunks = chunks
    size_kb = sum(p.stat().st_size for p in (out_dir / "data").glob("*.json")) // 1024
    print(f"written: index.json + {n_chunks} chunks ({size_kb} KB), "
          f"{len(out_sentences)} sentences", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
