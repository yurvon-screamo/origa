"""Unit tests for the tiered Cache-Control policy (``_cdn_cache``).

The policy is the fix for the PR #182 edge-cache poisoning: release-updated
content (grammar, phrases, dictionary) must NOT be cached immutable, while
truly-static assets (ML models, kanji art, system dictionary) stay immutable.

Tests are pure and have no dependency on S3, the network, or the gitignored
``cdn/`` store.
"""

from __future__ import annotations

import pytest

import _cdn_cache
from _cdn_cache import IMMUTABLE, NO_CACHE, RELEASE_UPDATED, cache_control_for
from deploy_cdn import SYNC_DIRS, VERSIONED_FILES


# ---------------------------------------------------------------------------
# Always-fresh
# ---------------------------------------------------------------------------


def test_manifest_is_no_cache():
    assert cache_control_for("manifest.json") == NO_CACHE


# ---------------------------------------------------------------------------
# Release-updated — the PR #182 bug category
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    "path",
    [
        "grammar/grammar.json",
        "dictionary/chunk_01.json",
        "dictionary/chunk_11.json",
        "dictionary/kanji.json",
        "dictionary/radicals.json",
        "phrases/phrase_index.json",
        "phrases/data/chunk_0.json",
        "pitch/index.json",
        "well_known_set/jlpt_n1.json",
        "well_known_set/well_known_sets_meta.json",
        "well_known_set/well_known_types_meta.json",
        "well_known_set/irodori_nyuumon/foo.json",
    ],
)
def test_release_updated_content(path: str):
    assert cache_control_for(path) == RELEASE_UPDATED


def test_grammar_is_not_immutable():
    # PR #182 root cause: grammar.json got immutable, edge cache poisoned.
    assert cache_control_for("grammar/grammar.json") != IMMUTABLE


# ---------------------------------------------------------------------------
# Truly-static / immutable
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    "path",
    [
        "ndlocr/model.onnx",
        "whisper/decoder.bin",
        "kanji_animations/anim_0001.svg",
        "kanji_frames/frame_0001.json",
        "phrases/audio/phrase_0001.mp3",
        "dictionaries/char_def.bin",
        "dictionaries/matrix.mtx",
        "dictionaries/dict.words",
        "dictionaries/JmdictFurigana.txt",
        "dictionaries/metadata.json",
    ],
)
def test_immutable_assets(path: str):
    assert cache_control_for(path) == IMMUTABLE


# ---------------------------------------------------------------------------
# Disambiguation: phrases/ has all three policies under one parent
# ---------------------------------------------------------------------------


def test_phrases_subtree_is_split_by_policy():
    assert cache_control_for("phrases/audio/phrase_0001.mp3") == IMMUTABLE
    assert cache_control_for("phrases/data/chunk_0.json") == RELEASE_UPDATED
    assert cache_control_for("phrases/phrase_index.json") == RELEASE_UPDATED


def test_dictionary_singular_vs_dictionaries_plural_differ():
    # "dictionary/" = app content (release-updated); "dictionaries/" = the
    # compiled lindera system dictionary (immutable). Easy to confuse.
    assert cache_control_for("dictionary/kanji.json") == RELEASE_UPDATED
    assert cache_control_for("dictionaries/char_def.bin") == IMMUTABLE


# ---------------------------------------------------------------------------
# Conservative default
# ---------------------------------------------------------------------------


def test_unknown_path_falls_back_to_release_updated():
    assert cache_control_for("some/unknown/path.json") == RELEASE_UPDATED


def test_unknown_prefix_under_known_dir_uses_dir_policy():
    # A novel file added under grammar/ inherits the directory policy, so a
    # future grammar/rules/ tree is not silently over-cached.
    assert cache_control_for("grammar/rules/some_new_rule.json") == RELEASE_UPDATED


# ---------------------------------------------------------------------------
# Regression: every shipped VERSIONED_FILE and SYNC_DIR is classified
# ---------------------------------------------------------------------------


def test_every_versioned_file_is_classified():
    for path in VERSIONED_FILES:
        assert cache_control_for(path) in (IMMUTABLE, RELEASE_UPDATED, NO_CACHE)


def test_every_sync_dir_is_classified():
    for dir_name in SYNC_DIRS:
        assert cache_control_for(dir_name + "/") in (IMMUTABLE, RELEASE_UPDATED)


def test_only_manifest_uses_no_cache():
    # no-cache is reserved for the change-detection beacon alone; if another
    # path ever needs it, that decision should be made deliberately.
    assert cache_control_for("grammar/grammar.json") != NO_CACHE
    assert cache_control_for("dictionaries/dict.words") != NO_CACHE


# ---------------------------------------------------------------------------
# Constant sanity — guard against accidental policy-string edits
# ---------------------------------------------------------------------------


def test_policy_constants_have_expected_values():
    assert IMMUTABLE == "public, max-age=31536000, immutable"
    assert RELEASE_UPDATED == "public, max-age=300, must-revalidate"
    assert NO_CACHE == "no-cache"
    assert _cdn_cache.cache_control_for("manifest.json") == "no-cache"
