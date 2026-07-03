# ADR-026: Touch-aware & safe-area layout conventions

## Status

Accepted

## Date

2026-07-03

## Context

PR #213 (ADR-022) introduced Android edge-to-edge rendering
(`viewport-fit=cover`) and the `env(safe-area-inset-*)` CSS custom
properties + Tailwind utilities (`pt-safe-t`, `pb-safe-b`, …). User testing
on an Android tablet (Chromium WebView ≥ 140) surfaced a cluster of layout
and touch-UX bugs that share two underlying gaps:

1. **No single layout-height contract.** Several CSS classes pinned
   `min-height: 100vh` independently — `body` (`min-h-screen`), the app
   shell `<main>` (`.main-with-sidebar`), and the per-page wrappers
   (`.page-layout-full`, `.page-layout-centered`, `.page-layout-compact`),
   plus inline `min-h-screen` on the lesson and home-loading views. On a
   mobile browser `100vh` is the *large* viewport (it includes the region
   behind the URL bar), so the stacked min-heights produced a body taller
   than the visible area → spurious scroll and an empty band at the bottom.
   Worse, once `padding-top: env(safe-area-inset-top)` is added to the shell
   (to clear the status bar), a nested `min-height: 100vh` child overflows
   its padded parent by exactly `safe-area-inset-top` on notched devices —
   reintroducing the very scroll the fix targets.

2. **Device class detected by width, not pointer.** Keyboard-shortcut hints
   (`[1]`, `[Space]`, `[Esc]`) were gated on the `sm:` width breakpoint
   (`hidden sm:inline`) — but a tablet in portrait is wide (≥ 640 px) yet
   touch-driven, so the hints stayed visible where no keyboard exists. Other
   hints were ungated entirely.

3. **No native-app selection discipline.** With no global `user-select`
   rule, long-pressing a button or nav label on touch fired the browser's
   native text-selection / copy menu — behaviour native apps do not exhibit.

This ADR promotes the four conventions applied to fix that cluster into
project-wide principles so the next page, component, or fixed element follows
them by default.

## Decision

### 1. Dynamic viewport height is owned by exactly one element

The app shell's full-viewport height uses `100dvh` (dynamic viewport height),
which excludes the browser chrome and so does not overflow the visible area.
It is applied **only** to `body` and the `<main>` shell — never to nested
layout wrappers.

- `body` → `min-height: 100vh; min-height: 100dvh;` (progressive enhancement,
  declared in `input.css`'s `body` rule).
- `<main>` → the sidebar branch uses `.main-with-sidebar`'s
  `min-height: 100vh; min-height: 100dvh;` (progressive); the non-sidebar
  branch uses the Tailwind arbitrary value `min-h-[100dvh]` (dvh-only —
  acceptable because the target is Chromium WebView ≥ 140 and desktop, both
  of which support `dvh`; on an unsupported engine the declaration is dropped
  and `<main>` sizes to content while `body`'s background fills the viewport).
- `.page-layout-full`, `.page-layout-centered`, `.page-layout-compact` have
  **no viewport `min-height`** — they size to content and let the shell's
  background fill the rest. This breaks the overflow chain: there is no
  nested viewport min-height to conflict with the shell's safe-area padding.

`body` and `.main-with-sidebar` carry a `vh`→`dvh` progressive fallback for
older WebViews; the non-sidebar `<main>` branch is `dvh`-only (declared above).
On desktop `dvh ≈ vh` and `env(safe-area-inset-*) = 0`, so the change is a
no-op there.

### 2. The content shell clears the top inset; fixed bottom elements clear the bottom inset

- The shell `<main>` receives `padding-top: env(safe-area-inset-top)` via the
  `pt-safe-t` utility, so every page's content starts below the status bar
  without each page repeating the rule.
- Fixed bottom elements use `env(safe-area-inset-bottom)` **with a floor**:
  `.bottom-tab-bar` uses `padding-bottom: max(env(safe-area-inset-bottom, 0px), 8px)`
  so labels lift above the gesture pill even when the reported inset is 0 or
  small. `.toast-container` is raised above the bottom tab bar on viewports
  where the bar is visible (`@media (max-width: 1023px)`). That media query is
  width-based, not route-based: on `/lesson` and `/onboarding` the bar is
  hidden (`BottomTabBar`'s `is_visible`) yet the toast still sits ~88 px
  higher — an accepted cosmetic trade-off, since CSS cannot be route-aware
  without body-class plumbing disproportionate to the benefit.

This requires `viewport-fit=cover` in `index.html` (already present);
otherwise `env(safe-area-inset-*)` returns `0`. On Android WebView < 140
(crbug 441253216) `env()` returns `0` regardless — the floor and the
content-padding-when-short design tolerate this (the shell's background still
fills; only the precise notch inset is lost).

### 3. Keyboard-only affordances hide on `pointer: coarse`, not on width

A single `.kbd-hint` class hides keyboard-shortcut hints under
`@media (pointer: coarse)`:

```css
.kbd-hint { display: inline; }
@media (pointer: coarse) {
    .kbd-hint { display: none; }
}
```

`(pointer: coarse)` matches touch-primary devices regardless of width, so a
wide touch tablet correctly hides the hints. It is preferred over:

- **Width breakpoints** (`sm:inline`) — a tablet is wide yet touch-driven.
- **`navigator.keyboard`** — limited support and does not reflect the actual
  input modality.
- **`maxTouchPoints`** — a touch laptop reports touch but still has a
  keyboard; the pointer media query is the better signal.

The keyboard *behaviour* (handlers in `keyboard_handler.rs`) is unaffected —
only the visual hint is hidden. A keyboard user on a touch device loses the
hint but not the function; this is the accepted trade-off.

Every keyboard hint in the app uses `.kbd-hint` (29 call sites across the
lesson, grammar-practice, and onboarding views).

### 4. Global `user-select: none` with a whitelist

`body` sets `user-select: none` (plus `-webkit-touch-callout: none` to
suppress the iOS long-press menu), so long-pressing UI chrome no longer
triggers native selection. Selection is **opt back in** for content the user
should be able to copy:

```css
input,
textarea,
[contenteditable="true"],
.markdown-text,
.translator-text,
.furigana-text,
.furigana-plain,
.word-translations,
.kanji-detail-hero-meaning,
.kanji-detail-hero-reading,
.kanji-card-answer,
.grammar-detail-hero-meaning {
    -webkit-user-select: text;
    user-select: text;
    -webkit-touch-callout: default;
}
```

This is a **whitelist**: any future container that must remain selectable
opts in by class. `user-select` is inherited, so the top-level content
container suffices (e.g. `.word-translations` covers its items and
description). The pre-existing `rt` rule (furigana readings excluded
from selection, fix for #178) is preserved — the base text of
`.furigana-text` / `.furigana-plain` stays selectable while its `<rt>`
readings do not.

## Alternatives Considered

### A1: `min-height: 100%` on nested wrappers instead of removing their viewport height

- **Pros:** Keeps a "fill the parent" intent explicit.
- **Cons:** Percentage `min-height` resolves against the parent's *height*,
  not `min-height`; with a `min-height`-only parent it collapses. It would
  also require making `<main>` an explicit flex container and depending on
  `leptos_router`'s `<Routes>` not wrapping the page view — a fragile
  coupling.
- **Rejected.** Letting wrappers size to content (with the shell background
  filling) is simpler and robust.

### A2: Reactive signal (`use_media_query("(pointer: fine)")`) for keyboard hints

- **Pros:** The visibility logic becomes unit-testable as a signal.
- **Cons:** More Rust code, WASM/SSR caveats, and re-renders for what is a
  static, declarative concern. The CSS media query is the idiomatic,
  cheaper, and better-supported mechanism.
- **Rejected.** The CSS class is preferred; it also handles dynamic input
  changes (e.g. a 2-in-1 detaching its keyboard) without reactivity.

### A3: `user-select: auto` (contextual) instead of `none` + whitelist

- **Pros:** Less prescriptive.
- **Cons:** `auto` defers to the parent/user-agent, which on touch WebViews
  still selects button labels — exactly the reported bug. The explicit
  `none` + whitelist makes the app-shell discipline unambiguous and is the
  pattern native-app-like web UIs adopt.
- **Rejected.**

## Consequences

### Positive

- **One declared height contract** (`100dvh` on shell only); no nested
  overflow on notched devices; no spurious mobile scroll.
- **Safe-area insets handled once** on the shell and fixed elements, not
  re-derived per page.
- **Keyboard hints consistent** — hidden on every touch device, visible on
  every pointer-fine device, independent of screen width.
- **Native-app-like selection** — UI chrome is not long-press-selectable;
  copyable content explicitly opts in.

### Negative

- **`user-select` is whitelist-managed** — a new selectable container that
  forgets to opt in silently becomes non-selectable. Mitigated by documenting
  the whitelist here and reviewing new content components against it.
- **Touch devices with an attached keyboard lose the visual hint** (function
  preserved). Accepted trade-off; `(pointer: coarse)` is the best available
  signal and avoids the false-positive of `maxTouchPoints` on touch laptops.
- **`dvh` is ignored by very old WebViews**; the `vh` fallback line keeps
  them working, only without the dynamic-toolbar benefit.

## References

- PR #213 / ADR-022: Android edge-to-edge + `viewport-fit=cover`
- ADR-024: build-script env-var empty-handling (the `env!()` discipline that
  makes the CDN URL reliable, relevant context for the `env()` safe-area
  values working on ≥ 140 WebViews)
- `origa_ui/index.html` — `viewport-fit=cover`, body `min-h-[100dvh]`
- `origa_ui/tailwind.config.js` — `pt-safe-t` / `pb-safe-b` / `safe` utilities
- `origa_ui/input.css` — `.kbd-hint`, `.page-layout-*`, `.bottom-tab-bar`,
  `.toast-container`, the `user-select` whitelist
- `origa_ui/src/routes.rs` — app shell `<main>` classes
- MDN: [dynamic viewport units (`dvh`)](https://developer.mozilla.org/en-US/docs/Web/CSS/length),
  [`@media (pointer)`](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/pointer),
  [`env(safe-area-inset-*)`](https://developer.mozilla.org/en-US/docs/Web/CSS/env)
