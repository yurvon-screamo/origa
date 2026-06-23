# CDN Handoff — Issue #178 (Phase 2 + Phase 4)

This document lists every CDN file changed during issues #178 Phase 2 and Phase 4
and the exact deployment steps the operator must run. The `cdn/` directory is
gitignored (see `.gitignore:83`), so changes are not visible in git — only
the local working copy on the engineer's machine differs from production.

Run all commands from the repo root (`D:\origa_worktree\origa`).

## Pre-flight: verify local state

```powershell
# Sanity-check that the regenerated phrase_index hash matches the algorithm
python scripts/remove_invalid_phrases.py --verify-hash --phrases cdn/phrases
# Expected: "RESULT: hash matches compute_hash(phrases) (OK)" and v=15 total=156231
```

## ⚠️ Deployment ordering — read before pushing test commits

The three new audit tests under `origa/tests/` (`kanji_descriptions_audit.rs`,
`grammar_content_audit.rs`, `well_known_sets_audit.rs`) read the corrected CDN
files locally and assert the post-fix state. CI (`ci.yml`) seeds `cdn/` from
production S3 before running `cargo test --workspace`. If you push the test
commits before deploying, CI will fetch the OLD CDN content and the audits
will fail — turning master RED and blocking every PR.

**Required order:**

1. Deploy CDN first (recipe below) — S3 now carries the corrected data.
2. Push the test commits — CI fetches the updated S3 content, audits pass.

The audits do **gracefully skip** on environments where `cdn/` is entirely
absent (fresh clones without S3 seeding), matching the canonical pattern in
`origa/tests/grammar_regression_checks.rs`. They only fail when `cdn/` is
present but carries the pre-fix content.

## Changed CDN files

| File                                                          | Slice | Change summary                                                  |
|---------------------------------------------------------------|-------|-----------------------------------------------------------------|
| `cdn/dictionary/kanji.json`                                   | W-10  | 21 polysemic kanji description_ru/description_en corrected      |
| `cdn/grammar/grammar.json`                                    | W-11  | 10 rules cleaned (19 Hangul fragments removed)                  |
| `cdn/grammar/rules/n4_legacy_fixes/rule_32_toki_joshi.json`   | W-11  | 1 rule cleaned (Hangul `공부` → `勉強`)                          |
| `cdn/grammar/rules/n5_legacy_fixes/rule_07_tsumori_desu.json` | W-11  | 1 rule cleaned (Hangul `있습니다` → `あります`)                  |
| `cdn/grammar/rules/n5_legacy_fixes/rule_18_ka.json`           | W-11  | 1 rule cleaned (Hangul `인가` → `か`)                            |
| `cdn/grammar/rules/n5_legacy_fixes/rule_23_past_noun_na_adj.json` | W-11 | 1 rule cleaned (Hangul `사람` → `人`)                           |
| `cdn/phrases/data/p*.json`                                    | P-3   | 518 profanity phrases removed across 198 chunk files            |
| `cdn/phrases/data/p0134.json`                                 | P-7   | 1 phrase translation corrected (id `0000000000NTQET51VCTVJAB4Q`)|
| `cdn/phrases/audio/*.opus`                                    | P-3   | 518 audio files removed (matched the deleted phrases)           |
| `cdn/phrases/phrase_index.json`                               | P-3   | v14 → v15; 156749 → 156231 entries; `h` recomputed              |
| `cdn/well_known_set/well_known_sets_meta.json`                | S-3   | 384 set levels remapped (372 Duolingo + 12 Spy x Family)        |
| `cdn/dictionary/chunk_01.json`                                | L-4   | 114 entries: split `;`-joined translations into separate items  |
| `cdn/dictionary/chunk_02.json`                                | L-4   | 133 entries: split `;`-joined translations                       |
| `cdn/dictionary/chunk_03.json`                                | L-4   | 136 entries: split `;`-joined translations                       |
| `cdn/dictionary/chunk_04.json`                                | L-4   | 188 entries: split `;`-joined translations                       |
| `cdn/dictionary/chunk_05.json`                                | L-4   | 220 entries: split `;`-joined translations (incl. `意思`)        |
| `cdn/dictionary/chunk_06.json`                                | L-4   | 150 entries: split `;`-joined translations                       |
| `cdn/dictionary/chunk_07.json`                                | L-4   | 141 entries: split `;`-joined translations                       |
| `cdn/dictionary/chunk_08.json`                                | L-4   | 179 entries: split `;`-joined translations                       |
| `cdn/dictionary/chunk_09.json`                                | L-4   | 138 entries: split `;`-joined translations                       |
| `cdn/dictionary/chunk_10.json`                                | L-4   | 64 entries: split `;`-joined translations                        |
| `cdn/dictionary/chunk_11.json`                                | L-4   | 141 entries: split `;`-joined translations                       |

Unchanged but tracked in `manifest.json` (auto-regenerated by `deploy_cdn.py`):
all other `cdn/**` paths.

## Backup artifacts retained locally

| File                                                 | Purpose                                                |
|------------------------------------------------------|--------------------------------------------------------|
| `cdn/phrases/phrase_index.json.bak.v14`              | Pre-P-3 index (156749 entries, v14 hash) for rollback  |
| `cdn/phrases/phrase_index.json.bak.v13`              | Pre-existing local backup from earlier work            |

## Deployment

```powershell
# 1. Preview what will change against the live S3 manifest
python scripts/deploy_cdn.py --dry-run

# 2. After reviewing the dry-run output, deploy for real
python scripts/deploy_cdn.py
```

Both commands regenerate `cdn/manifest.json` from the current local state and
diff against the remote manifest. Phrase chunks themselves are not in the
manifest (only `phrase_index.json` is), which is by design — the index is the
content-addressed pointer to the chunks.

The deploy script uploads each changed file with
`Cache-Control: public, max-age=31536000, immutable`, except `manifest.json`
itself which uses `Cache-Control: no-cache` so clients can detect the new
version on the next poll.

## Verification after deploy

```powershell
# Smoke-check the deployed manifest (will fetch from S3)
python scripts/deploy_cdn.py --dry-run
# Expected: "0 changed, 32 unchanged" if everything landed

# In the running app, the phrase index version should bump to v=15 and the
# client will re-fetch phrase chunks for any user-localized card that
# referenced a deleted phrase id (handled by existing orphan-removal logic
# in origa::use_cases::seed_ready_phrases).
```

## Rollback

If anything goes wrong:

```powershell
# Restore the pre-P-3 phrase index and chunks
cp cdn/phrases/phrase_index.json.bak.v14 cdn/phrases/phrase_index.json
git checkout origa/  # revert any source-code changes
python scripts/deploy_cdn.py  # redeploy the previous state
```

Note: the audio files deleted under `cdn/phrases/audio/` are not recoverable
from git (gitignored). They can be re-extracted from the source corpus run
that originally produced them, or restored from the S3 version history.

## Related commits (in `master`)

```
f14e5abe fix(tokenizer): assign grammar_label to grammar nouns (べき, はず, etc.) (#178 P-6, P-8)
b10ad670 fix(phrases): harmonize sentence splitters, compact multi-line spacing (#178 P-1, P-2)
1941bbf9 fix(dictionary): split semicolon-joined translations (#178 L-4)
dad8d023 fix(markdown): enable furigana in grammar examples (#178 W-12)
68dd80e4 feat(furigana): unify hover tooltips, support multi-kanji segments (#178 L-3, L-7)
5b83de52 fix(lesson): hide correct answer when right, dedupe grammar labels (#178 L-1, L-2, L-5)
8807514d fix(vocab): cap EN translations to top 7 by sense priority (#178 W-9)
94e1d3bc fix(cdn): correct Duolingo/Spy set JLPT levels (N5→N3+) (#178 S-3)
81495890 fix(cdn): remove 518 profanity phrases, regenerate phrase_index (#178 P-3)
a3dfaf14 fix(phrases): normalize quote-wrapped translations, correct 危ない phrase (#178 P-5, P-7)
6d35a248 fix(cdn): remove Korean text from grammar rules (#178 W-11)
2d166275 fix(cdn): correct polysemic kanji descriptions (字 + others) (#178 W-10)
d5a5fcc0 fix(scripts): correct phrase_index hash algorithm (sort_keys=True) (#178 pre-work)
```

## Scripts added for audit/replay

| Script                                              | Purpose                                                     |
|-----------------------------------------------------|-------------------------------------------------------------|
| `scripts/fix_polysemic_kanji.py`                    | Apply W-10 kanji fixes idempotently (`--dry-run` supported) |
| `scripts/remove_korean_from_grammar.py`             | Apply W-11 Korean removal idempotently                       |
| `scripts/fix_phrase_translations.py`                | Apply P-7 (and future) per-phrase corrections by id         |
| `scripts/detect_profanity_phrases.py`               | Re-scan phrase corpus; emits `--report` for remove script   |
| `scripts/remap_duolingo_spy_levels.py`              | Apply S-3 JLPT level remapping                              |
| `scripts/fix_dict_semicolon_translations.py`        | Apply L-4: split `;`-joined vocabulary translations         |

All scripts are idempotent — running them twice produces zero changes on the
second run.

## Phase 4 notes

### W-12 (Markdown furigana in grammar examples)

ADR-8 (provisional): code-side fix chosen over data fix.

- **Mini-investigation result**: all 3253 code-fence blocks across 332 grammar
  rule files contain Japanese characters (kana/kanji). 0 blocks contain
  literal code. Every grammar `examples` block is therefore language content.
- **Decision**: removed `"pre"` from `SKIP_TAGS` in `origa_ui/src/ui_components/markdown.rs`.
  The pulldown-cmark `<pre><code>...</code></pre>` rendering is preserved (the
  monospace block visually separates examples from explanation), but the
  furigana walker now applies ruby markup to text nodes inside `<pre>`.
- **Why not data fix**: the runtime parser already splits the example text
  correctly via furigana once `<pre>` is un-skipped. Removing the
  ` ``` ` fences from 332 files would change the visual rendering for no
  functional gain and force a CDN deploy of every grammar-rule file.
- **No CDN changes** required for W-12.

### L-4 (Dictionary semicolon-joined translations)

Both logic-side and data-side fixes applied:

- **Logic fix**: `origa::dictionary::vocabulary::split_semicolon_joined_translations`
  now splits each `t[]` array entry on `;` and drops empty fragments. The
  runtime renders clean arrays even without deploying the data fix.
- **Data fix**: `scripts/fix_dict_semicolon_translations.py` normalizes the
  stored form across `cdn/dictionary/chunk_*.json`. 1604 entries changed
  (across 11 chunk files, 96684 total entries). Example: `意思` had
  `["намерение; Воля; Цель", "смысл; Значение; Суть"]` and now carries
  `["намерение", "Воля", "Цель", "смысл", "Значение", "Суть"]`.
- Idempotent — re-running the script is a no-op.
