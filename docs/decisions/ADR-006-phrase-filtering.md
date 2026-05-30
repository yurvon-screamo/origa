# ADR-006: Phrase Dataset Filtering — Removing Non-Educational Fragments

## Status

Accepted

## Date

2026-05-28

## Context

The CDN phrase dataset contains 158,126 Japanese phrases (sourced from anime/manga dialogue). Many are short conversational fragments with no grammatical structure — bare noun compounds, single-word reactions, katakana loanwords. These fragments provide minimal educational value in a language learning app.

Additionally, some phrases contained tokens not present in the vocabulary dictionary due to kana-instead-of-kanji writing confusing the tokenizer, producing lemmatized forms that users would never encounter in study.

### Analysis Performed

Four filtering strategies were evaluated on the full dataset using sudachipy for POS tagging:

| Strategy | Removed | False Positive Estimate |
| :--- | :--- | :--- |
| Token count ≥ 3 | 20,027 (12.7%) | Too aggressive — removes all 2-token phrases |
| POS combinations (verb OR noun+adj OR noun+adv) | 12,584 (8.0%) | ~50% false positives — removes adj-only phrases, pron+adj, copula-dropped sentences |
| Combined (≤2 tokens AND no verb AND no adj) | 6,345 (4.0%) | 82.6% false positives — ignores copula (だ/です) and particles in full text |
| **Safe variant** (combined + exclude if ANY grammar marker in full text) | **1,109 (0.7%)** | **<5%** — only removes truly bare fragments |

### Key Finding

In casual Japanese, copula (だ/です) is frequently dropped, and many grammatically complete sentences rely on particles alone (は, が, の, etc.). The phrase index stores only content tokens (nouns, verbs, adjectives) — particles and copula are NOT included. Therefore, any filter based solely on content tokens cannot distinguish between "裸の名詞" (bare noun) and "copulaが省略された文" (copula-dropped sentence) without examining the full phrase text.

## Decision

Apply the **safe variant** filter in two steps:

### Step 1: Remove phrases with unknown vocabulary tokens (82 phrases)

Phrases where any content token is absent from the vocabulary dictionary (`cdn/dictionary/chunk_*.json`). These are tokenizer artifacts caused by kana-instead-of-kanji writing.

Unknown tokens: 縫い包み (ぬいぐるみ), 為出かす (しでかす), 将又 (はたまた), 真っ平 (まっぴら), 買い言葉, 彼式 (あれしき), 雲脂 (ふけ).

### Step 2: Remove phrases with no grammar markers (1,109 phrases)

A phrase is removed if ALL conditions are true:

1. ≤ 2 content tokens (from index `t` field)
2. No verb (動詞) among tokens
3. No adjective (形容詞 or 形状詞) among tokens
4. No copula (だ, です, である) in full text
5. No question markers (か？, かい, かな, かしら, の？) in full text
6. No particles (は, が, を, に, で, の, も, と, から, まで, へ) in full text

### Total removed: 1,191 phrases (0.75% of 158,126)

Result: **156,935 phrases remain** (index version 7).

## Alternatives Considered

### Aggressive POS-based filtering (8%)

Would remove 12,584 phrases but with ~50% false positive rate. Adjective-only phrases (嫌だ, 良い), copula-dropped sentences (これは恋), and question patterns (特徴は？) would be incorrectly removed.

### Combined filter without grammar marker check (4%)

Would remove 6,345 phrases but 82.6% (5,241) contain grammar markers (particles, copula, questions) in their full text. These are legitimate Japanese sentences with dropped copula — a standard feature of casual speech.

### No filtering

Keep all 158,126 phrases. Rejected because bare fragments like "メイド喫茶", "冗談冗談", "ハートショット" add noise to the learning experience without teaching grammar or vocabulary patterns.

## Implementation

- Python scripts: `scripts/remove_unknown_phrases.py` (Step 1), `scripts/remove_no_marker_phrases.py` (Step 2)
- Removal via existing `scripts/remove_invalid_phrases.py` pipeline (atomic write to chunks, index, audio)
- POS tagging: `sudachipy` with `sudachidict_core`
- Analysis scripts: `scripts/smoke_phrase_analysis.py` (6 analyses)

## Orphaned User Cards

No migration needed. `remove_orphaned_phrase_cards()` in `origa/src/use_cases/seed_ready_phrases.rs` automatically detects and deletes phrase cards whose `phrase_id` is no longer in the index. This runs on every `SeedReadyPhrasesUseCase::execute()` call.

## Consequences

- **Dataset is cleaner**: bare noun-noun fragments and tokenizer artifacts removed
- **Conservative approach**: <1% removal minimizes risk of losing educational content
- **False positives near zero**: only phrases with zero grammar markers are removed
- **Automatic cleanup**: existing orphan removal handles user-side card deletion
- **Reproducible**: all filtering decisions are in version-controlled scripts with deterministic output (random seed 42 for sampling)
