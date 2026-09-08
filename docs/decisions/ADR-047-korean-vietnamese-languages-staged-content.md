# ADR-047: Korean and Vietnamese interface languages with staged content rollout

## Status

Accepted

## Date

2026-09-07

## Context

Origa ships UI locales and dictionary translations for English and Russian.
To widen the audience we are adding Korean (KO) and Vietnamese (VI). The
kanji/radical dictionary content for KO/VI is hand-translated and reviewed,
but must **not** ship to the CDN yet — the user explicitly staged the rollout:
translations live in a separate, git-tracked location until the decision to
deploy is made.

Constraints:

- `cdn/` is gitignored (regenerable artifact), so translation data placed
  there would be unversioned and unreviewable.
- `NativeLanguage` is persisted as `i32` in TrailBase and IndexedDB and read
  by already-shipped binaries that map every unknown value to Russian.
- `leptos_i18n` 0.6 does not fail the build on missing locale keys
  (`emit_diagnostics` only prints warnings), so parity needs an external gate.
- The font pipeline subsets Noto Sans JP to the glyphs actually used; hangul
  has no coverage anywhere, so a KO UI would fall back to system fonts
  (tofu risk on Linux/WebKitGTK — ADR-30 class problem).
- `GrammarRule::content()` indexed a `HashMap<NativeLanguage, _>` directly —
  a KO/VI lookup with no CDN grammar content would panic in WASM.

## Decision

1. **Translation overlays** live in a new git-tracked top-level `translations/`
   directory (`kanji_ko_vi.json`, `radicals_ko_vi.json`), keyed by kanji/
   radical with `ko`/`vi` sub-objects. They are staging data: outside `cdn/`,
   invisible to `deploy_cdn.py` (which reads only `cdn/`). The merge runbook
   (including the font-deploy/manifest ordering) is `translations/README.md`.
2. **Wire format**: `NativeLanguage` gained `Korean = 2`, `Vietnamese = 3`
   (append-only; historical 0/1 unchanged; garbage still degrades to Russian).
3. **Fallback contract** until KO/VI dictionary data ships on the CDN: every
   language-projection accessor resolves KO/VI to the **English** content
   (kanji descriptions, radicals, vocabulary, phrases, well-known sets,
   grammar), never to Russian. Grammar resolution is a non-panicking chain
   `requested → English → any → static EMPTY` (empty text surfaces as the
   existing `GrammarContentNotFound` error).
4. **Locale parity gate**: `scripts/validate_translations.py` enforces
   bidirectional leaf-key parity across `en/ru/ko/vi`, placeholder parity,
   hangul presence in KO values, and an untranslated-string allowlist —
   `leptos_i18n` itself would only warn.
5. **Typography**: `scripts/subset_fonts.py` gained a pinned Noto Sans KR
   source subset to the hangul actually used by the locale and overlays
   (corpus sources are fail-loud), wired via a `HANGUL_RANGE` `@font-face`
   (`display: block`, ADR-30 rationale) and appended after "Noto Sans JP" in
   the font stacks. A cmap invariant fails the font build if any used hangul
   glyph is missing from the subset.
6. **e2e manifest ordering**: `subset_fonts.py` rewrites the `FONTS-*` block
   of `end2end/cdn-manifest.txt` with fresh hashes; that block stays reverted
   until the new fonts are actually deployed (e2e downloads every manifest
   line from production — committing undeployed hashes means silent 404s).

## Alternatives Considered

### Translate directly into `cdn/dictionary/*.json`

- Rejected: `cdn/` is gitignored — the hand-translated data would be
  unversioned, unreviewable and indistinguishable from regenerable
  artifacts; also violates the explicit "do not ship yet" staging.

### Fork `description_ko` fields into the parsers now, reading from a merged file

- Rejected for this slice: wires half-shipped data into Rust before the
  CDN decision; the overlay + runbook keeps the merge mechanical and
  reviewable when the decision is made.

### KO/VI → Russian fallback (mirror of the English → Russian legacy path)

- Rejected: a Korean/Vietnamese user seeing Russian is strictly worse than
  English; the EN projection is the canonical translation source.

## Consequences

- Selecting KO/VI in the UI works end-to-end today: fully translated
  interface, kanji/radical/vocab/grammar content in English until the CDN
  data lands (then the fallbacks are removed per the runbook).
- Mixed-version multi-device setups can silently downgrade a KO/VI selection
  to Russian when an **old** binary writes the profile back (old clients map
  unknown i32 → 1). We rely on Tauri updater adoption; documented in
  `translations/README.md`.
- The untranslated-string gate and the hangul cmap invariant are the quality
  backstops for autonomous translation work; both caught real defects during
  the initial rollout (Latin "Russian" in ru.json, unassigned jamo
  codepoints).
- Translation-only PRs have no CI wiring yet (path filter has no
  `translations` component) — known gap, tracked in `translations/README.md`.

## Note (2026-09-08): final delivery shape

The `translations/` staging directory was transitional and is gone: the
overlays were merged into the (gitignored) `cdn/` sources and deployed.
Translated content lives on the CDN only — the repository carries the
runtime code, the UI locales and the build/deploy gates. Two operational
caveats from the staging README outlive it:

- `NativeLanguage` persists as `i32` in TrailBase/IndexedDB; binaries
  older than the KO/VI release map unknown values to Russian, so in a
  mixed-version multi-device setup an old client can downgrade a KO/VI
  selection on profile write-back until the updater reaches it.
- The `FONTS-*` block of `end2end/cdn-manifest.txt` must only ever be
  committed together with the font files actually deployed (the e2e
  `download-cdn` step fetches every manifest line from production).
