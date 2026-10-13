# ADR-062: In-App Store Review Request Program

## Status

Accepted (issue #729)

## Date

2026-10-13

## Context

ASO slices across KR/VN/US storefronts (PR #724, #725, #726, guide
`docs/marketing/guides/2026-10-12-aso-appstore-toolkit.md`) converged on one
blocker: the app has **0 ratings** on every storefront. Apple weighs rating
velocity heavily, so keyword mechanics cannot work until a base pool of
ratings exists — the top of the US query results sits on 1.4k–5.5M ratings.
Brand queries convert poorly for the same reason.

The agreed lever is an in-app review request program: ask for a review after
a successful study session, slowly enough to respect Apple's system limits
(max 3 prompts per 365 days — excess calls are silently dropped by the OS).

## Decision

After each completed study session the lesson completion screen calls the
gate in `origa_ui/src/core/review_request.rs`; when the gate allows, the WASM
side invokes the `request_app_review` Tauri command
(`tauri/src/src/store_review.rs`), which forwards to the platform in-app
review API.

Gate rules (per device, `localStorage`):

- Sessions with zero processed cards never count and never prompt (guard is
  defense in depth — the completion screen requires ≥ 1 processed card, see
  the `lesson_state` reset in the lesson reload task).
- The first 3 completed sessions never prompt — an early prompt to a
  confused beginner risks a low rating, and the program exists to collect
  positive ratings.
- After a prompt is requested, the next one waits at least 120 days. With
  Apple's 3/365 ceiling this yields exactly 3 prompts/year maximum.
- The cooldown timestamp is recorded only when the native side reports the
  prompt was actually handed to the OS; platforms without a wired review API
  return `false` and never burn the cooldown.

Platform scope: **Apple only** (macOS `+requestReviewInWindow:` with the
main window, iOS `+requestReviewInScene:` with the foreground-active
scene). Windows, Linux, Android and the web build are no-ops today.

## Alternatives Considered

- **Deep link to the store review page** — App Review policy risk; the
  in-app API is the canonical, system-rate-limited mechanism.
- **Community Tauri review plugins** — none official/maintained; the call
  surface is two class methods, not worth a third-party dependency.
- **`objc2-store-kit` typed bindings** — pulls a large generated crate for
  two calls; raw `objc2` msg_send (version pinned by the workspace to the
  wry/tao generation) is sufficient and compiles under a standalone
  `rustc --target aarch64-apple-{ios,darwin}` check.
- **Swift bridge to StoreKit 2 `AppStore.requestReview(in:)`** — the
  non-deprecated replacement is Swift-only; a Swift bridge into
  `tauri/gen/apple` would add a toolchain build step for one call. Rejected;
  revisit if Apple removes the ObjC entry points.
- **Per-account gating** — rejected: per-device gating is simpler, requires
  no server state, and errs toward fewer interruptions (the ASO goal only
  needs a steady stream of prompted users).

## Consequences and Known Trade-offs

- Both ObjC entry points used here are **deprecated** (the whole
  `SKStoreReviewController` class since macOS 15 / iOS 18). There is no ObjC
  replacement — StoreKit 2's `AppStore.requestReview(in:)` is Swift-only —
  so the deprecation is accepted deliberately; both APIs remain functional
  on the project's minimums (macOS 14.0 / iOS 26.0).
- On non-App-Store macOS builds the OS silently never shows the dialog; the
  cooldown still ticks. macOS distribution is App Store-centric, so this is
  acceptable.
- Gate state lives in `localStorage`: clearing site data resets the
  counters. Apple's own 3/365 limit remains the hard backstop.
- The native Apple code is not compiled by CI (ubuntu lint). It is checked
  by a standalone `rustc --emit=metadata` compile against the real objc2
  rlibs for both Apple targets; full compilation happens at release builds.
- Release checkpoint (not in this PR): verify the dialog appears once on a
  TestFlight/App Store build after the third completed session.

## Not Done Here (future work)

- Windows MSIX: `Windows.Services.Store.StoreContext.RequestRateAndReviewAppAsync`.
- Android: Google Play / RuStore review APIs (RuStore has no in-app API).
- Storefront metadata changes (KR/VN/US) — per issue #729, applied through
  ASC only **after** the ratings pool exists.
