# ADR-060: Post-push sync bookkeeping derived from the pushed payload

## Status

Accepted (2026-09-21)

Amends ADR-045: supersedes its "re-fetch the server-authoritative
fingerprint after every push" clause (the rest of ADR-045 — the sync
orchestration, the dirty flag CAS discipline, the delta probe — stands).

## Context

ADR-045's full sync path ended with `record_sync_fingerprint`: after
pushing the merged user, the client re-fetched the entire row
(`find_current_raw`, a multi-megabyte GET) to compute the
"server-authoritative" fingerprint and the probe stamp. Two problems:

1. **The longest abort window on mobile.** The re-fetch ran immediately
   after the multi-second PATCH — on iOS, backgrounding the app at that
   moment kills the request (HTTP 499 server-side), the sync surfaces
   `Err`, and the user sees a "sync failed" error toast for data that
   had, in fact, landed (the false-positive class behind the 2026-09
   RC reports; Sentry stayed empty because the suspended WebView also
   dropped the queued telemetry).
2. **The authority was already client-owned.** Verified empirically
   against the production TrailBase (v0.33.x, STRICT SQLite tables):
   - the PATCH response body is **empty** — nothing server-rendered to
     fingerprint;
   - `updated_at` is **not** server-managed: the column is a plain
     `TEXT DEFAULT (datetime('now'))`, and the value actually stored is
     the client's own wire-bump (`user_to_json` stamps every push with
     `Utc::now().to_rfc3339()`);
   - the stored row is the pushed JSON verbatim (plus `id`, plus
     PATCH-partial survival of unpushed columns).

The "server-authoritative re-fetch" therefore fetched bytes the client
itself had just written. The ADR-045 concern it defended against — a
server that normalizes pushes — remains real but bounded (below).

## Decision

1. **`RemoteUserSource::save_with_record_id` / `create` return
   `SavedWireRow { record_id, wire }`** — the record the server actually
   holds (the update path can re-resolve to a different id; the failure
   path can fall through to `create`) and the exact wire payload pushed.
2. **`record_pushed_fingerprint` replaces `record_sync_fingerprint`.**
   The fingerprint is computed over `skeleton ∪ wire ∪ {id}` (union with
   the pushed payload winning on key collisions; `updated_at` excluded
   by the hash as before). The probe stamp is the pushed `updated_at`
   **verbatim** — the same renderer the probe's lexicographic `$gt`
   compares against. **Zero network operations after a push.**
3. **The skeleton.** `RemoteRow` retains a lightweight copy of the
   fetched row minus the fat blob columns (`knowledge_set`,
   `jlpt_progress`, `imported_sets` — every push fully overwrites
   them). Columns the client does not push survive server-side via
   PATCH-partial semantics, and the reconstruction must remember their
   values. ~1 KB instead of a megabyte-scale JSON tree; the peak memory
   profile improves (the re-fetch's full-row `Value` no longer exists).
4. **Checkpoint pushes record inline.** `save_sync_core` (login,
   onboarding, import checkpoints) now records the bookkeeping from its
   own pushed payload instead of leaving the meta dirty for the next
   merge's full fetch. The next merge collapses to a ~30-byte probe.
5. **Divergence canary.** On any full check, an unchanged stamp (nobody
   wrote since our push — the stamp is client-owned) combined with a
   fingerprint mismatch emits a `warn` with both hashes: that signature
   is reconstruction drift, not a legitimate remote edit (those bump
   the stamp).

## Known costs (accepted)

- **Server-side normalization divergence.** If the server ever rewrites
  pushed content (new columns, value coercion), the reconstruction
  misses it until the delta-probe valve (`PROBE_SKIP_FULL_CHECK_INTERVAL`
  full check) detects the mismatch — one extra full merge+push cycle,
  after which the fetched row's skeleton carries the normalized columns
  and the state settles. The canary makes the drift visible in logs.
- **Create-path asymmetry.** No pre-push row exists on `create`, so the
  first reconstruction has no skeleton; a server holding extra columns
  diverges until the first valve full check settles it (one-time cost
  per fresh account). The inline recording in checkpoint pushes
  (`save_sync_core`) has the same skeleton-less shape by construction —
  the checkpoint push does not fetch first (the fast path PATCHes by the
  session-cached record id).
- **Out-of-band server edits** (admin SQL) do not bump `updated_at` and
  are invisible to the probe — bounded by the same valve, unchanged
  from the probe design of #639.

## Field verification gate

The release channel must be healthy (v0.7.27/v0.7.28 tag builds failed)
before this ships in an RC. Device criteria: no sync-error toasts when
backgrounding during a sync on iOS; server HTTP logs show no
`GET /api/records/v1/domain_user` immediately after a `PATCH`.

## Consequences

- The false "sync failed" class disappears by construction: a genuine
  `Err` from the sync now means the fetch, merge, or push itself failed.
- Every push cycle costs one PATCH and zero follow-up GETs (was: PATCH +
  a full-row GET); steady-state syncs remain a ~30-byte probe.
- `recorded_fingerprint_is_server_authoritative_not_request_body`
  (the executable specification of the old clause) is superseded by
  `recorded_fingerprint_matches_the_server_read_back` (echo fidelity)
  and `server_side_normalization_settles_through_the_valve` (drift
  self-healing).
