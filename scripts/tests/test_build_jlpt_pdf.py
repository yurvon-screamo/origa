"""Unit tests for ``build_jlpt_pdf.py`` pure logic.

Covers the markdown-table generation (escaping, locale furniture, row
count) and the post-injection state machine (begin/end block replacement
is idempotent, legacy single-line marker migrates, a markerless post is
skipped with a loud warning). Rendering itself (WeasyPrint, system fonts)
is exercised by the CLI run contract: byte-identical re-run, verified
manually/via the script's own totals.
"""

from __future__ import annotations

import hashlib
import json
from datetime import datetime as _dt
import datetime as dt

import build_jlpt_pdf as gen

ENTRIES = [
    {
        "kanji": "人",
        "on_readings": ["ジン", "ニン"],
        "kun_readings": ["ひと", "と"],
        "description_en": ["person"],
        "popular_words": ["人間", "一人"],
        "jlpt": "N5",
        "used_in": 2357,
    },
    {
        "kanji": "一",
        "on_readings": ["イチ", "イツ"],
        "kun_readings": ["ひと"],
        "description_en": ["one | first"],
        "popular_words": ["一番"],
        "jlpt": "N5",
        "used_in": 9999,
    },
]


class TestMarkdownTable:
    def test_row_count_matches_entries(self):
        table = gen.markdown_table("N5", "en", ENTRIES)
        # header separator + one line per entry
        assert table.count("\n") == len(ENTRIES) + 1

    def test_pipe_in_data_is_escaped(self):
        table = gen.markdown_table("N5", "en", ENTRIES)
        for line in table.splitlines():
            # escaped pipes survive as `\|`; every row must have exactly
            # 6 unescaped separators (5 columns -> 6 pipes)
            assert line.count("|") - line.count("\\|") == 6, line

    def test_vi_uses_han_tu_not_kanji(self):
        # VI SEO invariant: the locale furniture must say "Hán tự", never
        # the Latin word "kanji" (mirrors the blog-side test).
        table = gen.markdown_table("N5", "vi", ENTRIES)
        assert table.startswith("| Hán tự |")
        assert "kanji" not in table.splitlines()[0]


class TestResolveSourceEpoch:
    def test_matching_sha_reuses_manifest_date(self, tmp_path, monkeypatch):
        monkeypatch.setattr(gen, "MANIFEST_PATH", tmp_path / "MANIFEST.json")
        sha = hashlib.sha256(gen.KANJI_JSON.read_bytes()).hexdigest()
        (tmp_path / "MANIFEST.json").write_text(
            json.dumps({"kanji_sha256": sha, "generated_from": "2020-01-01"}),
            encoding="utf-8",
        )
        epoch, changed = gen.resolve_source_epoch()
        assert not changed
        assert dt.datetime.fromtimestamp(epoch, dt.timezone.utc).strftime(
            "%Y-%m-%d"
        ) == "2020-01-01"

    def test_changed_sha_moves_date_to_today(self, tmp_path, monkeypatch):
        monkeypatch.setattr(gen, "MANIFEST_PATH", tmp_path / "MANIFEST.json")
        (tmp_path / "MANIFEST.json").write_text(
            json.dumps({"kanji_sha256": "stale", "generated_from": "2020-01-01"}),
            encoding="utf-8",
        )
        epoch, changed = gen.resolve_source_epoch()
        today = dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%d")
        assert changed
        assert dt.datetime.fromtimestamp(epoch, dt.timezone.utc).strftime(
            "%Y-%m-%d"
        ) == today

    def test_missing_manifest_is_first_render(self, tmp_path, monkeypatch):
        monkeypatch.setattr(gen, "MANIFEST_PATH", tmp_path / "MANIFEST.json")
        _, changed = gen.resolve_source_epoch()
        assert changed


class TestInjectPostTables:
    def _levels(self):
        return {lvl: [] for lvl in gen.LEVELS}

    def test_block_replacement_is_idempotent(self, tmp_path, capsys, monkeypatch):
        post = tmp_path / "en" / gen.POST_FILENAME
        post.parent.mkdir()
        begin = gen.POST_MARKER_BEGIN.format(level="N5")
        end = gen.POST_MARKER_END.format(level="N5")
        post.write_text(
            f"intro\n\n{begin}\n| old |\n{end}\n\ncoda\n", encoding="utf-8"
        )
        monkeypatch.setattr(gen, "POST_DIR", tmp_path)
        levels = self._levels()
        levels["N5"] = [
            {
                "kanji": "人",
                "on_readings": [],
                "kun_readings": [],
                "description_en": [],
                "popular_words": [],
            }
        ]
        gen.inject_post_tables(levels, dry_run=False)
        first = post.read_text(encoding="utf-8")
        assert "| 人 |" in first and "| old |" not in first
        gen.inject_post_tables(levels, dry_run=False)
        assert post.read_text(encoding="utf-8") == first
        assert first.endswith("coda\n")

    def test_legacy_marker_migrates_to_block(self, tmp_path, monkeypatch):
        post = tmp_path / "en" / gen.POST_FILENAME
        post.parent.mkdir()
        legacy = gen.POST_MARKER_LEGACY.format(level="N5")
        post.write_text(f"intro\n\n{legacy}\n\ncoda\n", encoding="utf-8")
        monkeypatch.setattr(gen, "POST_DIR", tmp_path)
        gen.inject_post_tables(self._levels(), dry_run=False)
        out = post.read_text(encoding="utf-8")
        assert gen.POST_MARKER_BEGIN.format(level="N5") in out
        assert gen.POST_MARKER_END.format(level="N5") in out

    def test_markerless_post_warns_loudly(self, tmp_path, capsys, monkeypatch):
        post = tmp_path / "en" / gen.POST_FILENAME
        post.parent.mkdir()
        post.write_text("just prose, no markers\n", encoding="utf-8")
        monkeypatch.setattr(gen, "POST_DIR", tmp_path)
        gen.inject_post_tables(self._levels(), dry_run=False)
        # File must be untouched (no anchor) and the skip must be visible
        # on stderr — fail-loud, not fail-silent.
        assert post.read_text(encoding="utf-8") == "just prose, no markers\n"
        err = capsys.readouterr().err
        assert "no kanji-table markers" in err and "warning" in err
