# ADR-031: Lesson card stable minimum height via `svh` on the Card component

## Status

Accepted — supersedes the fill-via-`grow`-only mechanism in [ADR-029](./ADR-029-lesson-card-fill-over-vertical-centering.md). ADR-029's `grow` on the Card stays in force as a secondary stretch mechanism; the **stable minimum height** introduced here is the primary fix for the residual card-resizing bug. ADR-027's height-chain (`<main>` flex-col → page `flex-1` → `lesson-content` `flex-1`) remains unchanged.

## Date

2026-07-07

## Context

ADR-029 made each card view's `<Card>` `grow` to fill the `lesson-content` rectangle, with the card top-anchored. The E2E suite in Chromium (820×1180) asserted `cardBox.height >= 80% of padded area` and passed. Yet users in the **Tauri** build (WebView2 / WKWebView / Android system WebView) still reported the original symptom: the card visibly resized between card types and between question/answer states, occupying ~30–40% of the viewport for short content.

Root cause analysis after the 4th iteration:

1. `flex-grow: 1` is **non-deterministic across WebViews**. Whether the card stretches to fill the container depends on subtle flex-basis/overflow interactions that differ between Chromium (Playwright), WebView2, WKWebView and the Android WebView. In some Tauri WebViews `grow` did not stretch a short-content card past its `min-h-[250px]`/`sm:min-h-[300px]` floor, leaving it at ~30% of the viewport.
2. The `p-[12dvh] px-[15dvw]` padding added in PR #243 interacted badly with the `max-w-4xl` (896px) wrapper on wide desktops: `px-[15dvw]` consumed ~576px on a 1920px viewport, leaving the card ~320px wide — a narrow strip rather than the intended "large card".
3. A `dvh`-based minimum would itself introduce a new source of jumping in the dev browser (`trunk serve`), where `dvh` floats as the mobile URL bar collapses/expands.

The lesson here is that **`grow` alone is not a reliable size guarantee across WebViews**. A deterministic minimum-height mechanism is needed that does not depend on flex-basis resolution quirks.

## Decision

The card view's `<Card>` gets a **stable minimum height in `svh`** (small viewport height), applied via a shared constant so every card type renders identically:

```rust
// origa_ui/src/pages/lesson/mod.rs
pub(in crate::pages::lesson) const LESSON_CARD_CLASS: &str =
    "p-4 sm:p-6 min-h-[60svh] sm:min-h-[70svh] flex flex-col grow";
```

All five card views (`LessonCard`, `QuizCardView`, `PhraseCardView`, `WritingCard`, `YesNoCardView`) reference this constant. Four of the five wrap it in `Signal::derive(|| super::LESSON_CARD_CLASS.to_string())` to preserve the pre-existing `Signal<String>` prop shape; `WritingCard` passes it as a literal `&'static str` (as it did before). In the same edit, the four card views' `shadow` and `test_id` props were also simplified from `Signal::derive(|| true)` / `Signal::derive(|| "lesson-card-root".to_string())` to the literals `true` / `"lesson-card-root"` (matching the pre-existing `WritingCard` shape — type-neutral via `#[prop(into)]`, removing redundant `Signal::derive` for static values).

The mix of `Signal::derive` and literal for `class` is forced by the `origa_ui_bin` recursion-limit landmine (ADR-027 §B3). The bin crate inherits the default `recursion_limit` of 128 and is near that ceiling. Converting all four `Signal::derive(|| ...)` `class` sites to literal `&'static str` tipped monomorphization over the limit during implementation (`error: queries overflow the depth limit!`). Keeping the four `Signal::derive` wrappers and the one pre-existing literal compiles cleanly. **Note:** the same four views did convert `test_id` (also `Signal<String>`) to literal without overflow — so the constraint is not "`&str` props on `<Card>` overflow", it is specifically the `class` prop's position in the tachys view-tree tuple at a critical monomorphization depth. The empirical workaround (leave `class` as `Signal::derive`, literals are fine for `shadow`/`test_id`) is what shipped.

Three coordinated choices:

1. **`min-h-[60svh] sm:min-h-[70svh]`** — a deterministic floor. Even if `grow` does not stretch the card in a given WebView, the card is at least 60% of the small viewport height on mobile and 70% on tablet/desktop. `grow` remains as a secondary mechanism that stretches the card to fill the `lesson-content` rectangle when the rectangle is taller than the floor.
2. **`svh`, not `dvh`** — `svh` is the small viewport height (the viewport size with all browser chrome expanded). It is **stable in every environment**: in Tauri WebView (no URL bar) it equals `dvh`/`lvh` and the full viewport; in the dev browser (`trunk serve`) it does not float when the URL bar collapses. `dvh` would have reintroduced jumping in dev mode.
3. **`lesson-content` padding reduced** from `p-[12dvh] px-[15dvw]` to `p-4 sm:p-6`. The proportional padding consumed ~24dvh of vertical space and, on wide desktops, conflicted with the `max-w-4xl` (896px) wrapper to leave the card ~320px wide. A fixed `p-4 sm:p-6` gives a consistent visual gap between the card and the viewport edge without starving the card of space.

### Vertical budget (mobile, 568px viewport, no notch)

| Element              | Height |
| -------------------- | ------ |
| LessonHeader         | ~50px  |
| lesson-page `py-4`   | 32px   |
| lesson-content `p-4` | 32px   |
| Card `min-h-[60svh]` | ~341px |
| rating buttons       | ~80px  |
| **Total**            | ~535px |

535px ≤ 568px viewport — fits with ~33px margin. With a notch (+44px safe-area) the page overflows by ~11px, which `lesson-content`'s `overflow-y-auto` handles by scrolling — acceptable, and no worse than the previous design.

### Why this is safe under ADR-026 §1

ADR-026 §1 bans viewport units on **page wrappers** (`page-layout-*`, `lesson-content`, `lesson-page`) because a nested `min-height: 100vh` overflows the shell's `pt-safe-t` padding by `env(safe-area-inset-top)` on notched devices. The `svh` minimum here is on the **`<Card>` content component**, not on a page wrapper. The card sits inside `lesson-content`, which has `overflow-y-auto`: if the card's `min-h` ever exceeds the available rectangle, the parent scrolls rather than overflowing the shell. The height-chain (ADR-027) and shell ownership of viewport height + safe-area (ADR-026) are preserved.

### E2E guard

The fragile layout assertions in `end2end/tests/lesson.spec.ts` (padding-top proportion, `cardBox.height >= 80% of padded area`) are removed — they were brittle (broke on any padding change, ±20px CI tolerance hacks) and asserted exact proportions the user explicitly does not want tested in E2E. The **height-chain guard** (`containerHeight > 700`) stays, because it is the automated protection for ADR-027's contract: if a future `leptos_router` upgrade wraps the matched view in a DOM node, `flex-1` collapses to content height and this assertion catches the regression. The `lessonCardRoot` locator is removed from the page object (dead code after the simplification); the `data-testid="lesson-card-root"` prop on `<Card>` stays as a semantic anchor for debugging and future tests.

## Alternatives Considered

### D1: Literal `h-[80svh]` on the Card (true fixed size)

- **Pros:** Matches the user's literal "80%×80%" request; fully deterministic.
- **Cons:** A fixed height cannot accommodate content taller than the available rectangle without an inner scroll region (the card would need its own `overflow-y-auto`, splitting scroll responsibility with `lesson-content`). On small viewports with a notch, `h-[80svh]` plus the header plus padding overflows the shell and forces shell-level scroll, which ADR-026 §1 was written to prevent.
- **Rejected** in favour of `min-h` + `grow` + parent-scroll.

### D2: Keep ADR-029's `grow`-only approach, debug why it fails in Tauri

- **Pros:** No new CSS; smallest possible change.
- **Cons:** The 4th iteration of this bug shows `grow`'s behaviour varies across WebViews in ways that are not practical to debug without device farms for every Tauri target. A deterministic `min-h` removes the dependency on flex-basis resolution quirks entirely.
- **Rejected.**

### D3: `dvh` instead of `svh`

- **Pros:** "Dynamic" viewport sounds more adaptive.
- **Cons:** `dvh` floats in the dev browser when the URL bar collapses/expands, reintroducing the jumping the user complained about — but only in `trunk serve`, masking the bug during local development while it persists in production Tauri. `svh` is stable everywhere; in Tauri (no URL bar) `svh` equals `dvh`.
- **Rejected.**

### D4: Add `max-h` cap on large desktops

- **Pros:** Prevents an overly tall card on 4K monitors.
- **Cons:** The `max-w-4xl` (896px) wrapper already bounds the width; at `sm:min-h-[70svh]` on a 2160px viewport the card is 896×1512 ≈ 1:1.69 aspect — a vertical rectangle, not an extreme strip. The user explicitly asked for a "large" card. Adding `max-h` would re-introduce an upper bound that fights `grow`.
- **Rejected** (consciously); revisit if smoke testing on 4K shows a real problem.

## Consequences

### Positive

- The card's minimum height is **deterministic across all Tauri WebViews** — no more reliance on `grow` resolving consistently between WebView2/WKWebView/Android.
- The shared `LESSON_CARD_CLASS` constant (DRY after the rule-of-three threshold of 5 call sites) guarantees all card types render with identical geometry.
- `lesson-content` padding is simpler (`p-4 sm:p-6` vs `p-[12dvh] px-[15dvw]`) and no longer starves the card of width on wide desktops.
- The E2E suite keeps the meaningful height-chain guard and drops the fragile proportion assertions that were hacked to ±20px CI tolerance.

### Negative

- `svh` support requires Chromium ≥ 108 (Nov 2022), Safari ≥ 15.4 (Mar 2022), Firefox ≥ 101 (May 2022). All current Tauri targets meet this (WebView2 auto-updates; Android System WebView ≥ 140 per `origa/AGENTS.md`), but a user on an ancient Android System WebView could see `min-h-[60svh]` ignored, falling back to `grow`-only behaviour (the ADR-029 state). Acceptable — `svh` is widely supported as of 2022.
- The `lessonCardRoot` page-object locator is removed; any future test that wants to assert on the card must re-add it (or use the still-present `data-testid="lesson-card-root"` directly).
- Card height still varies slightly on normal/reversed text cards when `show_answer` reveals rating buttons as flex-siblings of the card (the card shrinks by ~80px to make room). This is **pre-existing behaviour from ADR-029**, not introduced here, and affects only the question→answer transition on text cards — not the cross-type jumping the user reported.

## References

- [ADR-026](./ADR-026-touch-aware-safe-area-layout.md) — shell owns viewport height + safe-area; §1 bans nested viewport units on page wrappers (the Card is not a page wrapper).
- [ADR-027](./ADR-027-shell-flex-column-for-vertical-fill.md) — shell flex column / height chain (in force); E2E height-chain guard preserved here.
- [ADR-029](./ADR-029-lesson-card-fill-over-vertical-centering.md) — fill-via-`grow` (superseded as the primary mechanism; `grow` stays as secondary).
- `origa_ui/src/pages/lesson/mod.rs` — `LESSON_CARD_CLASS` constant.
- `origa_ui/src/pages/lesson/content.rs` — `lesson-content` class.
- `origa_ui/src/pages/lesson/{lesson_card,quiz_card,phrase_card,writing_card,yesno_card_view}.rs` — five call sites.
- `end2end/tests/lesson.spec.ts` — simplified height-chain guard.
- CSS Viewport Units Level 4 (W3C) — `svh`/`lvh`/`dvh` definitions.
