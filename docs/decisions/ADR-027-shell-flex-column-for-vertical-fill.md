# ADR-027: Shell `<main>` as a flex column for vertical fill / centering

## Status

Accepted — supersedes the rejection rationale in [ADR-026](./ADR-026-touch-aware-safe-area-layout.md) §A1 for pages that must fill the viewport vertically.

## Date

2026-07-03

## Context

ADR-026 established that viewport-height (`100dvh`) and safe-area
(`pt-safe-t`) live ONLY on the shell (`<body>`, `<main>`); page wrappers were
stripped of viewport `min-height` and size to content. That breaks the overflow
chain on notched devices (the original bug cluster ADR-026 fixed) and is the
correct default — most pages do not need to fill the viewport.

Bug #6 surfaced a page that DOES need vertical fill: the lesson view centers its
content card vertically in the space between a pinned header and the page
bottom. Centering (`flex-1` content area + `justify-content: safe center`)
requires the page wrapper to have a viewport-derived height so `flex-1` has free
space to distribute. With page wrappers sized to content (ADR-026), the only way
to propagate the shell's height to the page is to make the shell itself a flex
column so the page can `flex-1` it.

ADR-026 §A1 explicitly considered and rejected making `<main>` a flex container,
citing the fragile coupling to `leptos_router`'s `<Routes>` not wrapping the
matched page view (if `<Routes>` ever inserts a wrapper DOM node, the page is no
longer a direct flex child of `<main>` and `flex-1` stops filling). That
coupling is real and remains the key risk — but it is the ONLY viable mechanism
that does not reintroduce a nested viewport unit (ADR-026 §1) or re-derive
`env(safe-area-inset-*)` per page (ADR-026 §2). The trade-off is accepted for
pages that need vertical fill.

## Decision

The non-sidebar `<main>` shell branch is a flex column:

```rust
"paper-texture pt-safe-t min-h-[100dvh] flex flex-col"
```

This branch is used only where the sidebar is hidden — `/lesson`, `/onboarding`,
`/login`, and pre-auth states (`sidebar_visible = false`). The sidebar branch
(`.main-with-sidebar`) is unchanged.

A page fills the shell by being a flex child with `flex-1`:

```rust
<div class="flex-1 flex flex-col py-4" data-testid="lesson-page">
```

No viewport unit or `env(safe-area-inset-*)` appears on page wrappers — they
fill via flexbox only, keeping ADR-026's height/safe-area ownership on the shell.

### Hard dependency: `<Routes>` must not wrap the matched view

This contract holds because `leptos_router` 0.8 `<Routes>` renders the matched
route's view inline (no wrapper DOM node), so the page component is a direct
flex child of `<main>`. **If a `leptos_router` upgrade (or a manual edit) inserts
a wrapper between `<main>` and the page, every `flex-1` page silently collapses
to content height and vertical fill/centering breaks without a compile error.**

The E2E suite guards this for `/lesson`: a test asserts the lesson content area
has free vertical space on a tall viewport (i.e. `flex-1` is filling the shell,
not collapsed). Pages added in the future that rely on vertical fill must either
add an analogous guard or be covered by a shell-level layout test.

## Alternatives Considered

### B1: Reintroduce `min-h-screen` on the page wrapper

- **Pros:** No shell change; page gets viewport height directly.
- **Cons:** Directly violates ADR-026 §1 and reintroduces the exact overflow bug
  it fixed: a nested `min-height: 100vh` child overflows the shell's
  `pt-safe-t` padding by `env(safe-area-inset-top)` on notched devices →
  spurious scroll.
- **Rejected.**

### B2: `min-h-[calc(100dvh-env(safe-area-inset-top))]` on the page wrapper

- **Pros:** Stays off the shell; technically avoids the §1 overflow.
- **Cons:** Re-derives `env(safe-area-inset-*)` per page — exactly what ADR-026
  §2 centralized onto the shell to avoid. Brittle if the shell's padding
  changes; sets a bad precedent (every centering page re-derives safe-area).
- **Rejected.**

### B3: `margin: auto` (`my-auto`) on the card wrapper instead of `justify-content: safe center`

- **Pros:** Identical centering semantics without touching the shell.
- **Cons:** Requires adding a `class` (+ a `data-testid` for E2E) to a previously
  bare element in `LessonCardContainer`. tachys encodes each attribute as a
  generic type parameter, so adding attributes to a deep element increases the
  monomorphized view-type depth. The `origa_ui_bin` crate renders the entire
  `<App/>` tree under its default `recursion_limit` (128; the lib crate has 512
  but that is crate-scoped) and was already at the ceiling — the two new
  attribute type-params tipped it over, producing `error: queries overflow the
  depth limit!` during layout/monomorphization (platform-independent; would fail
  CI). Raising the bin's limit to 512 let it compile but produced 2479 linker
  errors from over-monomorphization. See the recursion-limit landmine note in
  `origa_ui/AGENTS.md`.
- **Rejected** in favor of `justify-content: safe center` on the existing
  `lesson-content` div (a class-string change only — `Class<&str>` stays
  `Class<&str>`, zero new type params; verified by byte-identical `origa_ui_bin`
  artifact hash vs. `master`).

`justify-content: safe center` is spec-defined to fall back to `flex-start` when
centering would overflow (preventing data loss), so it is behaviorally equivalent
to `my-auto`: center when the card fits, top-aligned + scrollable when it
overflows, with no top-clipping.

## Consequences

### Positive

- The lesson card (and any future page opting into vertical fill) can center /
  fill without reintroducing nested viewport units or per-page safe-area math.
- ADR-026's shell ownership of height + safe-area is preserved.

### Negative

- **Coupling to `<Routes>` non-wrapping** (see Decision). A `leptos_router`
  change that wraps the matched view silently breaks vertical fill across all
  `flex-1` pages. Mitigated by the `/lesson` E2E guard.
- **The non-sidebar shell branch now differs structurally** (flex column) from
  the sidebar branch (block). Pages in the non-sidebar branch are flex children;
  this is a no-op for single content-sized pages (`/login`, `/onboarding`) but is
  a behavior change reviewers must account for.

## References

- [ADR-026](./ADR-026-touch-aware-safe-area-layout.md) — shell owns viewport
  height + safe-area; §A1 rejected `<main>` as flex (superseded here for
  vertical-fill pages).
- `origa_ui/src/routes.rs` — non-sidebar `<main>` `flex flex-col`.
- `origa_ui/src/pages/lesson/mod.rs` — `lesson-page` `flex-1`.
- `origa_ui/src/pages/lesson/content.rs` — `lesson-content`
  `flex-1 min-h-0 overflow-y-auto flex flex-col justify-safe-center` (the
  `justify-safe-center` class is a **literal rule** in `origa_ui/input.css`, not a
  Tailwind utility — Tailwind v3 cannot express `safe center`; see §B3 and the
  recursion_limit landmine note in `origa_ui/AGENTS.md`).
- `origa_ui/AGENTS.md` — recursion-limit landmine note (why `my-auto` on a new
  attribute was infeasible).
- CSS Flexbox §8.4.1 (auto margins) and the `safe` alignment keyword
  (CSS Box Alignment §3.x).
