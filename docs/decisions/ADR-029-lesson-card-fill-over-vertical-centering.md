# ADR-029: Lesson card fills a fixed rectangle instead of vertical centering

## Status

Accepted — supersedes the vertical-centering mechanism in [ADR-027](./ADR-027-shell-flex-column-for-vertical-fill.md). Only ADR-027's `justify-content: safe center` centering (§Decision) and its §B3 `my-auto`-vs-`justify-safe-center` rationale are superseded. The **height-chain** half of ADR-027 (`<main>` flex-col → page `flex-1` → `lesson-content` `flex-1`) remains in force.

## Date

2026-07-06

## Context

ADR-027 centered the lesson card vertically inside `lesson-content` via
`justify-content: safe center` (a literal `.justify-safe-center` CSS rule, chosen
over `my-auto` to avoid adding attributes to a deep bare element — the
`origa_ui_bin` recursion-limit landmine, ADR-027 §B3).

Users reported that the card visibly JUMPS / RESIZES between card types (quiz,
text, writing, phrase, yesno): "то микроквадрат по центру экрана, то вертикально
растягивает, то фулл вью." PR #236 (`max-w-4xl` width cap) did not fix it because
the root cause is vertical, not horizontal:

1. `justify-content: safe center` shifts the card's vertical position depending
   on its height. Different card types have different intrinsic heights → the
   card's top edge moves → perceived "jumping."
2. The card does not FILL the container, so its visible bordered box changes
   height with content (short text card looks like a "micro-square"; a tall quiz
   card looks "stretched").

The `lesson-content` container itself is already a stable rectangle: it is
`flex-1` of the shell's flex column (ADR-027 height chain) ≈ 80-85% of the
viewport, and its size does NOT depend on card content. The bug is the centering

+ non-fill, not the container size.

## Decision

The lesson card FILLS the `lesson-content` rectangle and is top-anchored, so its
visible box is constant across all card types:

+ `lesson-content` keeps `flex-1 ... flex flex-col` (ADR-027 height chain,
  unchanged) but drops `justify-safe-center` — children are top-aligned by
  default.
+ The bare wrapper `<div>` in `LessonCardContainer` is removed, so the active
  card's root becomes a DIRECT flex child of `lesson-content` (Leptos 0.8
  `<Show>` renders no DOM wrapper; the component returns a view fragment).
+ Each card view's `<Card>` adds `grow` (`flex-grow: 1`, basis auto): the card
  grows to fill the container when its content is short, and expands by content
  when tall (then `lesson-content`'s `overflow-y-auto` scrolls).

This is implemented entirely with type-neutral edits (class-string changes + a
component-prop value + a node removal) — no new attribute is added to any deep
bare element, so the `origa_ui_bin` recursion limit is unaffected (verified by
`cargo test --workspace`).

A literal `h-[80dvh]` on the page wrapper was rejected: it would decouple the
container from the shell flex chain and risk overflow on notched devices
(ADR-026 §1 bans nested viewport units). `flex-1` already yields a stable
≈80-85% rectangle and satisfies the user's "≈80%" intent.

### testid convention

To enable an E2E guard without a CSS-class selector, the card component root
carries `data-testid="lesson-card-root"` (passed via the `Card` component's
existing `test_id` prop — type-neutral; the `data-testid` attribute node already
exists in `Card`'s view type). This is distinct from
`data-testid="lesson-card"` (the `mod.rs` page-content wrapper). The two must
not collide.

## Alternatives Considered

### C1: Keep centering, fix only the non-fill

+ **Pros:** Minimal change to ADR-027.
+ **Cons:** A centered, filled card still shifts when its height crosses the
  container height (centered-when-fits vs top-aligned-when-overflows). Does not
  fully eliminate the jump.
+ **Rejected.**

### C2: Literal `h-[80dvh]` on `lesson-content`

+ **Pros:** Matches the user's literal "80%" request.
+ **Cons:** Violates ADR-026 §1 (nested viewport unit on a page wrapper →
  notched-device overflow); decouples the container from the header/shell.
  `flex-1` already provides a stable ≈80-85% rectangle.
+ **Rejected.**

## Consequences

### Positive

+ The card's visible box is constant across all card types — no jumping.
+ `flex-1` height chain (ADR-027) preserved; no viewport units reintroduced.
+ A testid on the card enables robust E2E layout assertions without CSS-class
  selectors.

### Negative

+ The `.justify-safe-center` literal CSS rule and its comment are removed (dead
  code once the class is dropped from `lesson-content`).
+ ADR-027's centering-specific guidance is superseded (this ADR records that).

## References

+ [ADR-027](./ADR-027-shell-flex-column-for-vertical-fill.md) — shell flex column
  / height chain (still in force); its centering mechanism is superseded here.
+ [ADR-026](./ADR-026-touch-aware-safe-area-layout.md) — shell owns viewport
  height + safe-area (§1 bans nested viewport units).
+ `origa_ui/src/pages/lesson/content.rs` — `lesson-content` class.
+ `origa_ui/src/pages/lesson/lesson_card_container.rs` — wrapper removal.
+ `origa_ui/AGENTS.md` — recursion-limit landmine note.
