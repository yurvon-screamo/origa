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
                    translations ru/vi/ko are carried over from the previous
                    corpus (or added later by translate_examples.py)

Deterministic: sorted iteration. Sentence ids are STABLE across regenerations
via carry-over: a sentence whose text already existed in the previous corpus
keeps its previous id (so the immutable, id-addressed audio keys
`examples/audio/{id}.opus` and the ru/vi/ko translations inside the chunks
survive a regeneration); only genuinely new sentences get fresh ids above the
old index's max sid. Requires the previous corpus (index.json + data/*.json)
to be present in cdn/examples/; otherwise falls back to compact ids with a
warning. A PRESENT but unreadable/inconsistent previous corpus fails loud
(OldCorpusError): regenerating over it would shift every id and drop the
translations. Re-running with the same inputs yields identical bytes.

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
from _cdn_corpus import corpus_dir

REPO = Path(__file__).resolve().parent.parent
CDN = corpus_dir()
EXAMPLES_URL = "https://www.edrdg.org/pub/Nihongo/examples.utf.gz"
DEFAULT_WORK = Path("/tmp/opencode")

MAX_SENT_CHARS = 60  # textbook register: short, self-contained
MIN_SENT_CHARS = 6
MAX_PER_WORD = 3
CHUNK_SIZE = 1000  # sentences per data chunk
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
            reading = m.group(1)
            # Tanaka id artifacts (`で(2028980)`): a numeric token id, not
            # a reading — shipped as pairs like で|#2028980 and rendered
            # as garbage furigana.
            if any(ch.isdigit() for ch in reading) or reading.startswith("#"):
                continue
            ruby.append([base, reading])
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
        req = urllib.request.Request(
            EXAMPLES_URL, headers={"User-Agent": "origa-build"}
        )
        with urllib.request.urlopen(req, timeout=60) as resp, gz.open("wb") as dst:
            while chunk := resp.read(1 << 20):
                dst.write(chunk)
    with (
        gzip.open(gz, "rt", encoding="utf-8") as src,
        path.open("w", encoding="utf-8") as dst,
    ):
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
                ok = MIN_SENT_CHARS <= len(ja) <= MAX_SENT_CHARS and ja.endswith(
                    SENT_END
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
    """Pick up to MAX_PER_WORD sentences per word: shortest first (textbook register).

    Kanji words take only sentences where the word SURFACES (`start >= 0`):
    offset-less kana-variant matches (`[-1,-1]`, e.g. the auxiliary てしまう
    usage of 仕舞う) do not demonstrate the card word at all — an example
    without the word is worse than no example.
    """
    sel: dict[str, list[dict]] = {}
    for w in sorted(words):
        cand = matches.get(w, {})
        if KANJI_RE.search(w):
            cand = {sid: off for sid, off in cand.items() if off[0] >= 0}
        if not cand:
            continue
        ranked = sorted(
            cand.items(),
            key=lambda kv: (kv[1][0] == -1, len(sentences[kv[0]]["x"]), kv[0]),
        )[:MAX_PER_WORD]
        sel[w] = [{"sid": s, "start": off[0], "end": off[1]} for s, off in ranked]
    return sel


class OldCorpusError(Exception):
    """The previous corpus is present but unreadable/inconsistent.

    A regeneration with carry-over needs a trustworthy previous corpus:
    silently falling back to compact ids here would shift every sentence id
    (breaking the immutable audio keys) and drop the chunk-stored ru/vi/ko
    translations — the exact failure class this mechanism exists to prevent.
    Fail loud instead; a genuinely absent corpus (fresh machine) is the only
    legitimate compact-fallback path.
    """


def read_old_corpus_index(out_dir: Path) -> tuple[set[int], int]:
    """Parse cdn/examples/index.json; return (referenced_sids, max_ref_sid).

    Raises OldCorpusError when the file exists but does not parse.
    """
    path = out_dir / "index.json"
    try:
        old_index = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as e:
        raise OldCorpusError(
            f"{path} is present but unreadable ({e}); "
            "fix or remove the previous corpus before regenerating"
        ) from e
    referenced: set[int] = set()
    max_sid = 0
    for v in old_index.get("words", {}).values():
        for r in v.get("refs", []):
            if isinstance(r, list) and r:
                try:
                    sid = int(r[0])
                except (TypeError, ValueError):
                    continue
                referenced.add(sid)
                max_sid = max(max_sid, sid)
    return referenced, max_sid


def parse_old_chunk_row(
    row: object, referenced: set[int]
) -> tuple[int, str, dict[str, str]] | None:
    """Extract (sid, x, translations) from one old chunk row.

    Returns None for rows not worth carrying: malformed ids, filler
    (unreferenced — generate_missing_examples.py rows that never got index
    refs and are unreachable by clients), or rows without usable text.
    """
    if not isinstance(row, dict):
        return None
    try:
        sid = int(row["i"])
    except (KeyError, TypeError, ValueError):
        return None
    if sid not in referenced:
        return None
    x = row.get("x", "")
    if not isinstance(x, str) or not x:
        return None
    tr: dict[str, str] = {}
    for L in ("ru", "vi", "ko"):
        v = row.get(L)
        if isinstance(v, str) and v.strip():
            tr[L] = v
    return sid, x, tr


def read_old_corpus_chunk_maps(
    out_dir: Path, referenced: set[int]
) -> tuple[dict[int, str], dict[str, int], dict[int, dict[str, str]]]:
    """Scan data/s*.json for referenced rows; build the carry maps.

    Raises OldCorpusError when a chunk file exists but does not parse.
    Collision policy for duplicate x: smallest old_sid wins.
    """
    old_sid_to_x: dict[int, str] = {}
    old_x_to_sid: dict[str, int] = {}
    old_sid_to_tr: dict[int, dict[str, str]] = {}
    for cf in sorted(out_dir.glob("data/s*.json")):
        try:
            rows = json.loads(cf.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as e:
            raise OldCorpusError(
                f"{cf} is present but unreadable ({e}); "
                "fix or remove the previous corpus before regenerating"
            ) from e
        if not isinstance(rows, list):
            raise OldCorpusError(f"{cf}: expected a JSON array of rows")
        for row in rows:
            parsed = parse_old_chunk_row(row, referenced)
            if parsed is None:
                continue
            sid, x, tr = parsed
            old_sid_to_x[sid] = x
            if x not in old_x_to_sid or sid < old_x_to_sid[x]:
                old_x_to_sid[x] = sid
            if tr:
                old_sid_to_tr[sid] = tr
    return old_sid_to_x, old_x_to_sid, old_sid_to_tr


def read_old_corpus(
    out_dir: Path,
) -> tuple[dict[int, str], dict[str, int], dict[int, dict[str, str]], int]:
    """Read the previous corpus for carry-over of stable sentence ids.

    Returns (old_sid_to_x, old_x_to_sid, old_sid_to_tr, max_old_index_sid).
    Only INDEX-REFERENCED sentences are included: filler sentences
    (generate_missing_examples.py) have no refs in index.json and are dead
    weight, so they are deliberately dropped here. Collision policy for
    duplicate x: smallest old_sid wins.

    Absent corpus (fresh machine) returns empty structures — the caller
    falls back to compact ids. A PRESENT but unreadable or inconsistent
    corpus raises OldCorpusError: regenerating over it would silently
    shift every sentence id (breaking the immutable audio keys) and drop
    the chunk-stored translations.
    """
    old_index_path = out_dir / "index.json"
    old_chunks = sorted(out_dir.glob("data/s*.json"))
    if not (old_index_path.exists() and old_chunks):
        return {}, {}, {}, 0

    referenced, max_old_index_sid = read_old_corpus_index(out_dir)
    old_sid_to_x, old_x_to_sid, old_sid_to_tr = read_old_corpus_chunk_maps(
        out_dir, referenced
    )
    if referenced and not old_x_to_sid:
        raise OldCorpusError(
            f"index.json references {len(referenced)} sentences but no "
            "matching chunk rows were read; the previous corpus is "
            "incomplete — fix or remove it before regenerating"
        )
    return old_sid_to_x, old_x_to_sid, old_sid_to_tr, max_old_index_sid


def build_sid_map(
    sentences: list[dict],
    used_sids: list[int],
    old_x_to_sid: dict[str, int],
    max_old_index_sid: int,
) -> tuple[dict[int, int], dict[str, int]]:
    """Map each newly-parsed sid to a carried (old) or fresh id.

    Returns (sid_map, stats) where stats = {"carried", "new"}.
    """
    sid_map: dict[int, int] = {}
    stats = {"carried": 0, "new": 0}
    next_new_id = max_old_index_sid + 1 if max_old_index_sid > 0 else 1
    # if no old corpus, start compact ids at 0 to mirror legacy behaviour
    if max_old_index_sid == 0 and not old_x_to_sid:
        next_new_id = 0
    used_old_sids: set[int] = set()
    for sid in used_sids:
        x = sentences[sid]["x"]
        old_sid = old_x_to_sid.get(x)
        # an old sid can be claimed by only one new sentence (1:1 carry);
        # a duplicate x on the new side (or an already-claimed old sid)
        # falls through to a fresh id.
        if old_sid is not None and old_sid not in used_old_sids:
            sid_map[sid] = old_sid
            used_old_sids.add(old_sid)
            stats["carried"] += 1
        else:
            sid_map[sid] = next_new_id
            next_new_id += 1
            stats["new"] += 1
    return sid_map, stats


def build_out_rows(
    sentences: list[dict],
    used_sids: list[int],
    sid_map: dict[int, int],
    old_x_to_sid: dict[str, int],
    old_sid_to_tr: dict[int, dict[str, str]],
) -> list[dict]:
    """Build the output chunk rows, riding carried translations along.

    A row carried from the previous corpus keeps its ru/vi/ko translations;
    genuinely new rows ship without them (the client falls back to English).
    """
    rows = []
    for sid in used_sids:
        s = sentences[sid]
        row = {"i": sid_map[sid], "x": s["x"], "en": s["en"], "f": s["f"]}
        old_sid = old_x_to_sid.get(s["x"])
        if old_sid is not None and old_sid in old_sid_to_tr:
            for L in ("ru", "vi", "ko"):
                v = old_sid_to_tr[old_sid].get(L)
                if v is not None:
                    row[L] = v
        rows.append(row)
    return rows


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
    ensure_examples_file(examples_path)

    words = load_popular_words()
    print(f"popular words: {len(words)}", flush=True)

    sentences, matches = parse_and_match(words, examples_path)
    print(
        f"sentences parsed: {len(sentences)}; words matched: {len(matches)}", flush=True
    )

    sel = select_for_words(sentences, matches, words)
    covered = len(sel)
    full = sum(1 for v in sel.values() if len(v) >= 3)
    print(f"coverage: >=1: {covered}/{len(words)} | >=3: {full}", flush=True)

    # --- Carry-over of stable sentence ids (post-#929 regeneration) -------
    # See read_old_corpus / build_sid_map / build_out_rows helpers above.
    out_dir = CDN / "examples"
    (out_dir / "data").mkdir(parents=True, exist_ok=True)

    try:
        old_sid_to_x, old_x_to_sid, old_sid_to_tr, max_old_index_sid = read_old_corpus(
            out_dir
        )
    except OldCorpusError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1
    if old_x_to_sid:
        print(
            f"carry-over: old index max sid {max_old_index_sid}, "
            f"referenced rows {len(old_sid_to_x)}, "
            f"unique x {len(old_x_to_sid)}",
            flush=True,
        )
    else:
        print(
            "carry-over: no previous corpus present; using compact ids "
            "(audio keys and chunk translations will NOT be carried)",
            flush=True,
        )

    used_sids = sorted({ref["sid"] for v in sel.values() for ref in v})

    sid_map, carry_stats = build_sid_map(
        sentences, used_sids, old_x_to_sid, max_old_index_sid
    )
    print(
        f"carry-over: carried {carry_stats['carried']}, "
        f"new {carry_stats['new']}, "
        f"max id now {max(sid_map.values()) if sid_map else 0}",
        flush=True,
    )

    out_sentences = build_out_rows(
        sentences, used_sids, sid_map, old_x_to_sid, old_sid_to_tr
    )

    out_words = {
        w: {
            "refs": [[sid_map[r["sid"]], r["start"], r["end"]] for r in sel[w]],
        }
        for w in sorted(sel)
    }

    # wipe old chunks AFTER reading them for carry-over
    for old in out_dir.glob("data/s*.json"):
        old.unlink()

    # write chunks by id RANGE so gaps do not eat files
    chunks_map: dict[int, list[dict]] = {}
    for row in out_sentences:
        chunks_map.setdefault(row["i"] // CHUNK_SIZE, []).append(row)
    for cn in sorted(chunks_map):
        chunks_map[cn].sort(key=lambda r: r["i"])
        p = out_dir / "data" / f"s{cn:04d}.json"
        p.write_text(
            json.dumps(chunks_map[cn], ensure_ascii=False, separators=(",", ":")),
            encoding="utf-8",
        )

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

    # cache matches for the translation pipeline (ids aligned with chunks)
    cache_sel = {}
    for w in sorted(sel):
        cache_sel[w] = [
            {"sid": sid_map[r["sid"]], "start": r["start"], "end": r["end"]}
            for r in sel[w]
        ]
    (WORK / "examples_matches.json").write_text(
        json.dumps(cache_sel, ensure_ascii=False),
        encoding="utf-8",
    )

    n_chunks = len(chunks_map)
    size_kb = sum(p.stat().st_size for p in (out_dir / "data").glob("*.json")) // 1024
    print(
        f"written: index.json + {n_chunks} chunks ({size_kb} KB), "
        f"{len(out_sentences)} sentences",
        flush=True,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
