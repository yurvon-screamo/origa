# ADR-048: Precomputed tokenization — lindera off the render path

## Status

Accepted

## Date

2026-09-12

## Context

The tokenizer dictionary (SudachiDict, ~344 MB raw, ~688 MB of JS→WASM
memcpy per warm start) sat on the startup overlay as step 0 of 9 AND on
every render of Japanese text: `furiganize_segments` → `annotate_text` →
`tokenize_text` (lindera), `TranslatorText` → `tokenize_text`, TTS
readings, and card creation (#521).

The user-facing requirement: **tokenization is only allowed when creating
new content; rendering existing content (lessons, user pages, phrases,
grammar) must not touch it at all.**

Backward compatibility is a hard constraint on three axes: old clients +
new CDN files, new clients + old CDN (missing blobs), old persisted user
data (cards without the new caches).

## Decision

Split by content lifecycle instead of by component:

1. **Static CDN content → offline precompute.** Builders (`utils
   build-phrase-precompute`, `utils build-grammar-precompute`) derive
   furigana spans + tokens for the exact strings components render:
   phrase full text + every `split_japanese_sentences` sentence (the
   splitter lives in the `origa` domain so builder and runtime cannot
   drift), grammar markdown text nodes (same pulldown+ammonia+scraper
   pipeline as `MarkdownText`) and whole strings (titles, nuances).
   Blobs are deflated rkyv (~23% of raw), lazy per phrase chunk,
   freshness binds source bytes PLUS tokenizer inputs (dictionary
   version dir + furigana source hash) so a dictionary bump regenerates.
2. **User cards → cache at creation + one-time startup migration.**
   `VocabularyCard.tokens` (serde-default, legacy-safe) persists the
   token the constructor already computes. For cards created before the
   field existed, `BackfillCardTokensUseCase` runs once at startup after
   the background warmup completes: it tokenizes every legacy card and
   persists the result through the regular `save` (the same
   write-on-startup pattern as the phrase seeding). The pass is idempotent
   and becomes a no-op; renders answer from the persistent cache, and
   until the pass completes legacy cards simply use the live-path
   fallback.
3. **Everything else → fast paths + on-demand gate.**
   `furiganize_segments` resolves kanji-free text as one plain segment
   (no dictionary at all), consults the precompute store, then tries a
   single-word furigana-dictionary lookup (card words are lemmas), and
   only then falls back to live tokenization. The dictionary left the
   startup overlay (9→8 flags), warms up in the background after the
   light phases (iOS jetsam budget: no overlap with the heavy stage),
   and content-creation flows gate on `ensure_tokenizer_loaded` (CAS
   lifecycle with call coalescing and retry-after-failure).

**Contract invariants** (enforced by tests):

- A store entry with **empty `furigana_spans` is a miss** for the
  furiganize path — it serves token translations only, so a card entry
  can never shadow a better reading from the live paths.
- `is_known` is computed at render time from the user's `known_kanji`,
  so precomputed spans never go stale as the user learns kanji.

## Alternatives Considered

### Runtime memoization of tokenization (client-side cache)

Rejected: first render of every phrase still needs lindera; cold lesson
starts stay on the heavy path; no answer for offline first view.

### Full `TokenTranslation` precompute per native language

Rejected: 4× blob size for data the render already has in memory
(vocabulary/grammar dictionaries load before the overlay lifts);
token-level precompute is language-independent.

### Runtime synthesis of legacy tokens (POS cache + furigana reading)

Rejected after an initial implementation: it avoided the one-time write
but kept a permanent second code path (real vs synthesized tokens) with
extra contract cases. A dumb one-time migration on startup is simpler
and converges: after the pass every card carries a real persisted cache
and the synthesis branch disappears.

## Consequences

- Warm start: the overlay no longer waits for the dictionary; renders
  answer from precompute/furigana-dict/card caches.
- CDN grows by ~63 MB deflated phrase blobs (lazy per chunk) + one 4 MB
  grammar blob; deploy order (CDN before app release) unchanged, and the
  e2e mirror lists both so CI exercises the fast path.
- A corrupted blob payload is NOT deep-validated (CheckBytes is shallow,
  raw deflate has no checksum) — accepted trade-off, documented on
  `access_precompute_blob`; the manifest guard binds blobs to sources.
- The tokenizer still loads (background) for creation flows and rare
  arbitrary-text paths — it is not eliminated, just off the render path.
- Offline bundle (precache) does not include phrase precompute blobs —
  offline phrase renders use the fallback (the dictionary IS precached);
  tracked as a future follow-up if offline first-paint regresses.

## References

- Issue #521; PR #523
- `origa/src/domain/tokenizer/precomputed.rs` (store + contract)
- `origa/src/domain/furigana.rs` (fast-path chain)
- `utils/src/commands/build_phrase_precompute.rs`,
  `utils/src/commands/build_grammar_precompute.rs`
- ADR-036/037-era CDN blob pattern (`origa/src/dictionary/cdn_blob.rs`)
