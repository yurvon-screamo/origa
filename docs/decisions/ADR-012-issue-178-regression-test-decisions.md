# ADR-012: Issue #178 Regression Test — Key Decisions

## Status

Accepted

## Date

2026-06-23

## Context

Issue #178 "Regression test" is a meta-issue collecting 37 defects surfaced by
regression testing of Origa across 8 categories (`/words`, `/sets`, `/phrases`,
`/profile`, lesson, additional, landing, release). It was delivered in 5 phases
through 22 vertical slices (35+ commits, 70 files, +3762/-369 LOC).

Each decision below was taken in the context of a specific bug. The rationale is
documented here to avoid re-litigation and to make the trade-offs explicit for
future contributors.

## Decisions

### ADR-012.1: Phrase index hash algorithm — `sort_keys=True`

**Context:** Field `h` in `cdn/phrases/phrase_index.json` could not be
reproduced by the Python script `remove_invalid_phrases.py`, which blocked safe
phrase removal (P-3 profanity cleanup). The mismatch was in key ordering: the
Rust producer `compute_hash`
(`utils/src/commands/enrich_phrases_with_grammar.rs:94-122`) uses `serde_json`
without the `preserve_order` feature, so `Map = BTreeMap` and keys are emitted in
alphabetical order (`c,g,i,t`). Python used `sort_keys=False`, producing
insertion order (`i,t,c,g`).

**Decision:** Switch Python from `sort_keys=False` to `sort_keys=True`. It was
also discovered that v14 hashes were computed WITHOUT sorting `grammar_rules`
(a defense-in-depth sort was added later in `e5422218`); the Python algorithm
must match v14 for compatibility with the published index.

**Alternatives considered:**

- Recompute ALL hashes via Rust with a grammar sort → changes every `h`, too
  invasive for the dataset.
- Add the `preserve_order` feature to workspace `serde_json` → breaks other
  dependencies that rely on the default `Map` behaviour.

**Consequences:** A `--verify-hash` mode was added for verification; 518
profanity phrases were removed (156749 → 156231), hash v15 `f5ae77ef...`.

### ADR-012.2: `build.rs` as the canonical source for JLPT levels (title parsing)

**Context:** S-3 (Duolingo/Spy Family sets all tagged N5) was initially fixed by
the Python script `remap_duolingo_spy_levels.py` (commit `94e1d3bc`). But the
tests kept failing — `origa_ui/build.rs` regenerated `meta.json` with a
hardcoded `"N5"` on every build, overwriting the Python band-aid. Before the
root-cause fix (commit `3100bea7`), the `extract_meta` calls inside
`generate_well_known_meta` (`origa_ui/build.rs`) passed a literal `"N5"` for
every Duolingo and Spy x Family entry.

**Decision:** Fix the generator, not the data. A `section_number_from_title()`
helper was added to `build.rs` (parsing "Section X" / "Модуль X" from the title
in EN+RU) plus a mapping Section 1-2 → N5, 3-4 → N4, 5-6 → N3. SpyFamily is
hardcoded to N3.

**Empirical insight:** The parent directory name (`n3/n4/n5`) is a legacy
grouping, NOT the JLPT level — the `n5/` catalogue contains 60 files with
"Section 3" titles (which must be N4).

**Alternatives considered:**

- Keep the Python band-aid and run it as a build step → couples build
  correctness to a Python dependency and to the script staying in sync with the
  generator.
- Derive the level from the parent directory name (`n3/n4/n5`) → empirically
  wrong (the `n5/` directory contains 60 "Section 3" sets that must be N4).

**Consequences:** The Python script is DEPRECATED (kept only as a diagnostic).
The single source of truth is `build.rs`. The parsing logic is duplicated in
`well_known_sets_audit.rs` because the build script cannot depend on the test
crate.

### ADR-012.3: Feature flag `grammar_practice_lesson_mode` (W-5)

**Context:** W-5 — the "Тренировка" (Training) button for grammar must launch a
full lesson instead of a modal. High risk: lesson balancing and FSRS scheduling
could break.

**Decision:** Implement under the feature flag `grammar_practice_lesson_mode`
(default OFF in `origa_ui/Cargo.toml`). Skeleton: `LessonMode::GrammarPractice`
enum variant + `parse_grammar_practice_query`. Card generation for now reuses
`SelectCardsToLessonUseCase` (full grammar-aware generation is deferred and
coupled with L-6).

**Alternatives considered:**

- Ship default ON immediately → too risky for production.
- Defer entirely → blocks progress on the issue.

**Consequences:** The feature flag allows gradual rollout and fast rollback.
E2E tests in `end2end/pages/grammar.page.ts:94` must be updated when the flag is
enabled in CI.

### ADR-012.4: L-6 deferred — domain change required

**Context:** L-6 — when a grammar card enters a lesson, three mutant quiz cards
are pulled in. A mini-investigation found the mechanism does not exist (0 matches
in code).

**Decision:** DEFER. Implementation requires:

- A new module `origa/src/domain/knowledge/grammar_companions.rs`.
- A domain change: `RateMode::GrammarCompanion` variant.

Per AGENTS.md ("Спроси сначала" for `origa/src/domain/`), this needs explicit
approval.

**Alternatives considered:**

- Implement without a domain change by reusing an existing `RateMode` → breaks
  FSRS scheduling contracts.
- Spot-fix the 3 misclassified records → the mini-investigation showed this is a
  systemic class (B), not 3 records (A).

**Consequences:** L-6 remains open. It is documented in the handoff doc and is
coupled with the full W-5 implementation (which cannot land without the L-6
decision).

### ADR-012.5: D-1 manual kanji restore (auto-recovery rejected)

**Context:** D-1 — some phrases lost their kanji (`しかたねーですね` instead of
`仕方ねーですね`). The user asked about automatic recovery.

**Decision:** REJECT automatic recovery. Only manual fixes to specific phrases (9
phrases fixed: `しかたねー` → `仕方ねー` ×8, `しかたねぇ` → `仕方ねぇ` ×1).

**Rationale:** Automatic recovery has XL complexity and critical risks:

- Homophones (multiple kanji for one reading): `かた` = `方/型/肩/硬/片...` —
  impossible to disambiguate.
- Colloquial forms (`-ねー` long-vowel suffix) are often intentionally written
  without kanji.
- There is no `text_kanji` field in `PhraseDetail` to verify against.

**Alternatives considered:**

- Auto-recover from a homophone dictionary → ambiguity makes any single choice
  unreliable (`かた` alone has 5+ valid kanji).
- Re-tokenize the kana form and pick the most frequent kanji → colloquial
  `-ねー` suffixes are often intentionally kana-only, so "most frequent" is the
  wrong signal.

**Consequences:** D-1 is closed for this issue. If >20 phrases appear later, a
follow-up with a curated list is warranted. The phrase index hash is UNCHANGED
(only the display text `x` was edited; tokens `t` already had kanji).

### ADR-012.6: `FuriganaText` unified API (L-3, L-7)

**Context:** The hover tooltip showing the kanji name was not available on every
card; `木綿` showed the tooltip only for `木` (the first kanji), not for `綿`.

**Decision:** Merge `FuriganaText` and `FuriganaTextWithHover` into a single
`FuriganaText` with optional props `native_language` and
`with_kanji_tooltip`. Multi-kanji segments are split into individual characters
via `split_segment_to_chars` so each kanji gets its own popup.

**Alternatives considered:**

- Keep two components → DRY violation, duplicated logic.
- Tooltip at the segment level (not the character level) → does not solve
  `木綿` (you still need to know which kanji is hovered).

**Consequences:** 5 call-sites in `grammar_practice_modal.rs:366,379,392,405,418`
were updated. `FuriganaTextWithHover` is deprecated/removed. This is a breaking
API change, but backward-compatible through the optional props.

### ADR-012.7: `<rt>` excluded from copy selection (copy artifact)

**Context:** A user reported the artifact `答えコタエて` when copying Japanese
text. Root cause: the browser includes `<rt>` (furigana annotation) in the
clipboard alongside the `<ruby>` body.

**Decision:** A global CSS rule `rt { -webkit-user-select: none;
user-select: none; }` in `origa_ui/input.css`.

**Rationale:** Per the HTML spec, `<rt>` is a ruby text annotation (the reading),
not the base text. This is a universal property of the element, not of a style
class, so it is robust for any future `<rt>` (including hand-authored ruby).
The `-webkit-` prefix covers the Tauri WebView on macOS/WebKit.

**Alternatives considered:**

- A `.furigana-rt` class only → covers the current case but is fragile for the
  future.
- A `data-text` attribute + a JS copy handler → over-engineering for a
  copy-paste problem.

**Consequences:** Copy-paste from the UI now yields clean kanji without the
reading. Applies to every `<ruby>` element in the app.

### ADR-012.8: W-2 `mark_card_as_known` fallback (no domain change)

**Context:** W-2 — `mark_card_as_known` did nothing for non-New cards (silent
`Ok`).

**Decision:** Fallback variant without a domain change: remove the
`if !is_new { silent Ok }` branch; for already-Learned cards emit
`tracing::warn!` + `Ok(())` (idempotent); for InProgress/Hard cards perform the
action unconditionally.

**Alternatives considered:**

- Add `OrigaError::CardAlreadyLearned` (a domain change) → requires "Спроси
  сначала" per AGENTS.md; not done without explicit user approval.

**Consequences:** No domain change. The silent failure is eliminated (now a warn
log). Idempotency for Learned cards is preserved.

## Follow-ups

1. **L-6** — requires a domain design decision (see ADR-012.4).
2. **W-5 full** — grammar-aware card generation, coupled with L-6.
3. **PR-2 (profile/offline cache re-trigger on language switch)** — the
   defensive `CardCacheState::Idle` guard and `tracing::debug` at the
   `set_locale` call sites are in place; the failure path could not be
   reproduced statically, so it "needs verification in production" traces.
4. **CDN deploy ordering** — `docs/handoff_issue_178_cdn.md` records that S3 must
   be deployed BEFORE pushing tests (audit tests compare against CDN state).

## References

- Plan: architect's 22-slice plan (5 phases).
- Implementation: 35+ commits `ce5b1d15` → `b348a4ca`.
- Tests: 1583 passed (baseline 1577 → +6 from the feature flag + audit fixes).
- Handoff: `docs/handoff_issue_178_cdn.md`.
- Related: ADR-006 (phrase-dataset filtering by grammar markers; P-3 reused the
  same `remove_invalid_phrases.py` pipeline for profanity removal, but the
  filtering logic itself differs — ADR-006 removed non-educational fragments,
  P-3 removed profanity).
