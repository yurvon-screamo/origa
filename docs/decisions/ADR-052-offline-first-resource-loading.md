# ADR-052: Offline-First Resource Loading

## Status

Accepted (2026-09-13)

## Context

A user on iOS with a **warm dictionary cache** could not open the app without
internet: the startup overlay spun on «Загружаем словари и учебные материалы •
0 из 8» for minutes. The counter staying at zero meant the loader futures
never resolved at all — network requests were hanging, not failing.

Investigation found five hang points and one policy defect:

1. **No timeouts anywhere on the network paths.** CDN fetches
   (`cdn_provider.rs`), the startup manifest check (`cache_manager.rs`) and
   the TrailBase client (`gloo_net`) all used bare `fetch` promises. On
   "connected Wi-Fi, no uplink" every request hangs for the OS-level
   DNS/TCP timeouts (tens of seconds to minutes); the staged pipeline
   (`Phase A → Stage 1 → Stage 2 → Phase C`) serialized them, and
   `load_with_retry` doubled the stalls for kanji/radicals/JLPT.
2. **The pipeline ignored offline state.** Phase A fetched the manifest even
   with a full cache; a cache miss went to the network regardless of
   `navigator.onLine`.
3. **Silent failure.** `tracked()` flips readiness flags on errors, so a
   fast failure opened the app with empty dictionaries and no explanation.
4. Login with no network hung on the TrailBase request (`gloo_net` exposes
   no `AbortSignal`).
5. A saved session without a local profile hung in `check_session`'s merge
   branch (`remote.find_current_raw`).
6. No e2e coverage of offline starts; `failure_paths.feature` codified the
   eternal overlay as "sanctioned degradation".

## Decision

### 1. Idle-deadline fetch primitive (`utils/net_timeout.rs`)

Every network request goes through one primitive: an `AbortController`-backed
fetch whose watchdog aborts the request when **no data has arrived for N ms**
(default 10 s). Progress — a received body chunk — **restarts** the timer, so
slow-but-alive transfers finish while silent stalls abort. An aggregate
deadline was rejected: the first launch legitimately downloads tens of
megabytes over slow mobile networks.

Timeout errors carry a textual marker recognized by `is_idle_timeout()`;
HTTP statuses and instant refusals are distinguishable without touching the
shared domain error enum.

The primitive powers the CDN provider (text/bytes/prefetch), the manifest
fetch, and the TrailBase transport (`send_request_idle`; `gloo_net` was
replaced by `web_sys::fetch` + `RequestInit::signal` for API calls — the
alternative of racing and abandoning gloo requests would leak browser
per-host connection slots on retry loops). TrailBase call sites consume an
`ApiResponse` envelope with the accessor surface they had before, minus the
async: the transport has already read the body.

**Memory invariant (iOS jetsam, staged plan in `routes.rs`):** the
text/bytes paths already buffered whole bodies in JS (`text()` /
`array_buffer()`), so routing them through the stream reader does not change
the peak-heap profile. Only `prefetch_to_cache` switched from streaming the
body into `cache.put` to JS-side reading — those bodies are small (audio
files, card assets). Responses put into the Cache API are rebuilt from the
consumed bytes preserving status and content type.

### 2. Offline gate: probe, never trust `navigator.onLine`

`navigator.onLine` lies on WKWebView custom schemes (`tauri://localhost`) in
both directions, so it only decides **whether to probe**, never the verdict:

- When the browser claims offline, a **HEAD probe of the manifest URL**
  (~2 s idle budget; any HTTP status — even 405 — counts as "alive") settles
  the truth. A live CDN still gets full manifest validation, keeping cache
  invalidation working on iOS; a dead one sets `CdnUnreachable` and skips
  Phase A.
- When the browser claims online, the manifest fetch itself runs under the
  idle deadline; a transport-level failure (refusal or timeout — not an HTTP
  status) sets `CdnUnreachable`.
- `CacheFirstCdnProvider` consults `fetch_decision(cache_hit,
  cdn_unreachable)`: a cache miss under the flag **fails instantly** instead
  of issuing a doomed request. This is what keeps the honest-offline budget
  at seconds.
- The flag is cleared by the browser `online` event and by Retry. A false
  positive costs one extra attempt after the clear; a false negative hangs
  the app — which is why the flag is never set from `onLine` alone.

### 3. Retry policy

`load_with_retry` re-runs a failed loader only when the failure is transient
(HTTP 5xx, an instant refusal on a live network). An idle-timeout stall or a
proven-unreachable CDN makes the retry pure latency and skips it (R3-C1).

### 4. Failure verdict and UX (product decision)

**The load-error screen (`app-load-error` + Retry) is reserved for a FULL
critical failure** — all five critical resources (vocabulary, phrases, kanji,
grammar, radicals) failed, meaning nothing is loadable and nothing is cached
(first offline launch, evicted cache). A **partial failure never blocks the
app**: it keeps running with what loaded (silent degradation; a partial-loss
banner and online-event auto-refetch are tracked follow-ups, not shipped).

Consequences accepted deliberately:

- A returning user whose entire cache was evicted sees the error screen —
  correct, there is nothing to show.
- A singleton failure of the heaviest resource (vocabulary) with the rest
  loaded still opens the app: user data is intact (IndexedDB), only
  definitions/search degrade until the next online start. Recovery today is
  an app restart or the Retry button on a full failure.
- `failure_paths.feature` was rewritten: a dead CDN on a cold cache now
  expects the error screen on both loads. The eternal overlay is no longer
  sanctioned behavior.

The failure branch in `ProtectedRoute` outranks the readiness check (after a
full failure every readiness flag is also `true` — `tracked` flips them on
errors), otherwise a hollow "successful" open would mask the error screen.

### 5. Retry mechanics: generations over cancellation

WASM `spawn_local` tasks cannot be cancelled, so Retry does not try.
Instead, each pipeline run captures a **generation counter**; only the
current generation may set terminal signals (`load_failure`, the final
`is_jlpt_content_loaded`). A stale run resolving after a Retry lands on
idempotent ground: domain `OnceLock` slots ignore double sets, readiness
`set(true)` is idempotent, and its late error cannot override the new run.

Retry resets the readiness flags of resources whose **domain slot** never
installed (the authoritative "did it actually load" signal —
`retry_plan(domain_loaded, readiness)`); successful loaders early-return on
their domain guards and are not re-fetched. Retry always clears
`CdnUnreachable` — it must attempt the network, never trusting a stale
probe. Duplicate in-flight traffic for one failed resource (the stale hang
plus the fresh attempt) is accepted: cache-first and idempotent slots make
it harmless.

## Verification

- WASM browser tests: stalled-stream abort with the marker; chunk-progress
  deadline reset; refused connection/POST without the marker; unreachable
  cache-miss refusal under 900 ms; flag round-trip; decision matrix.
- Native tests: verdict classification, retry plan, retry policy, response
  envelope, manifest staleness logic (pre-existing).
- E2E (`offline_startup.feature`, 7 scenarios): first offline start → error
  screen within 20 s (not an eternal overlay); Retry recovers after the
  network returns; offline login fails fast; saved session without a local
  profile does not hang; warm cache opens the app in ≤10 s with no
  successful CDN responses and at most one manifest probe; a lesson
  completes offline with cached audio; partial cache loss does not block.

## Budgets

- Honest offline, cold cache: probe ~2 s → refusal cascade → error screen
  in ~3–5 s.
- Worst case (WebKit lying `onLine=true`): manifest idle deadline ~10 s →
  refusal cascade → error screen in ~10–11 s.
- Warm cache offline: probe ≤2 s + cache parse — target ~2–6 s; the e2e
  ceiling of 10 s is CI-runner headroom, not the design goal.
- E2E runs against a static build only: `trunk serve`'s live-reload WS
  bridge force-reloads on offline disconnects and poisons the scenarios.

## Consequences

- No network path in the app can hang indefinitely.
- The app is offline-usable with a warm cache: dictionaries, lessons and
  cached audio work with zero successful CDN responses.
- `guard_expectation` already treats a missing remote manifest as
  `Unavailable` (trust the cache), so the offline-skip of Phase A is
  consistent with the blob freshness model.
- Follow-ups (tracked, out of scope here): partial-degradation banner,
  auto-refetch of failed resources on the `online` event, Sentry events for
  `load_failure`, a full offline contract for sync push (`save_sync`).
