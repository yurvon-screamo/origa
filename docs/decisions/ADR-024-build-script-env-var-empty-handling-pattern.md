# ADR-024: Build-script env var reads must handle the empty-string case

## Status

Accepted

## Date

2026-07-02

## Context

ADR-020 (`TRAILBASE_URL` in `origa_ui`, merged) fixed one instance of this bug
class. A sibling effort, PR #213 (ADR-023: `ORIGA_CDN_BASE_URL` in `origa_ui`),
targets a second instance but is **open and not yet merged** at the time of
writing — so on `master`, `origa_ui`'s CDN read is still the vulnerable
`env!()` (see inventory). This ADR promotes the fix to a project-wide
**principle** and closes the two instances neither landed effort reaches:
`tauri/build.rs` (3 vars) and `origa_landing/build.rs` (1 var).

### Root cause (generalized)

Two Rust mechanisms for reading an env var at build time are **both** blind to
the empty-string case:

- `env!("VAR")` panics only when the var is **unset**; for a var **set to `""`**
  it silently compiles to `""`.
- `env::var("VAR").unwrap_or_else(|_| default)` catches only the **unset** case
  (the `Err`); for a var **set to `""`** it returns `Ok("")` and the fallback is
  skipped, yielding `""`.

A shell carrying `VAR=""` (an easy mistake — an unterminated assignment, a CI
step that exports an empty secret, a `.env` with a blank value) therefore
produces an empty value downstream, with no warning. For a host-URL var the
consequence is severe: an empty host is substituted into a CSP (dropping that
host — the WebView then blocks requests to it) or into `BASE_URL` (every
canonical URL, Open Graph tag, and JSON-LD `url` becomes relative).

### The two remaining instances

1. **`tauri/build.rs`** — three reads affected the CSP, on **all** desktop
   platforms (Win/Linux/macOS/iOS/Android):

   ```rust
   let cdn      = env::var("ORIGA_CDN_BASE_URL")   .unwrap_or_else(|_| build_config::DEFAULT_CDN.to_string());
   let trailbase = env::var("TRAILBASE_URL")        .unwrap_or_else(|_| build_config::DEFAULT_TRAILBASE.to_string());
   let landing  = env::var("ORIGA_LANDING_BASE_URL").unwrap_or_else(|_| build_config::DEFAULT_LANDING.to_string());
   ```

   An empty value for any one assembled the CSP without that host, so the
   WebView blocked requests to the CDN, TrailBase, or the landing. Symptom on
   desktop: CSP violation errors in devtools.

2. **`origa_landing/build.rs`** — `ORIGA_LANDING_BASE_URL`:

   ```rust
   std::env::var("ORIGA_LANDING_BASE_URL").unwrap_or_else(|_| "https://origa.app".to_string())
   ```

   An empty value emitted `cargo:rustc-env=ORIGA_LANDING_BASE_URL=`, so
   `env!("ORIGA_LANDING_BASE_URL")` in `seo.rs:6` compiled to `""`. Every
   canonical URL, OG tag, and JSON-LD field became relative — an SEO
   catastrophe. (Production CI was unaffected by the empty case: `ci.yml` and
   `docker.yml` always pass a non-empty value. The bug bit local `cargo`
   builds of the landing with an empty shell var.)

   The same line also carried a **second, independent defect**: its fallback
   literal was `https://origa.app`, which is **not** the canonical landing
   domain — see "Host-value drift fix" below.

### Relationship to ADR-020

ADR-020 fixed `origa_ui`'s `TRAILBASE_URL` (the crate that lacked any
fallback). This ADR fixes the two build scripts ADR-020 did not touch
(`tauri/build.rs` already had a fallback but the wrong kind; `origa_landing/build.rs`
had a fallback with the wrong value). The fix mechanism — a `resolve_env`
helper that treats empty as "use default" — is identical; this ADR simply
declares it the rule rather than the exception. PR #213 (ADR-023, open) applies
the same mechanism to `origa_ui`'s CDN read independently; the two efforts do
not overlap in the files they edit (see "Conflict / sequencing with PR #213").

## Decision

### 1. Principle: empty is equivalent to unset for host-URL vars

Every build-script env-var read that has a production default MUST resolve via
a helper that treats **both** unset and empty as "use default":

```rust
pub(crate) fn resolve_env(env_value: Option<&str>, default: &str) -> String {
    match env_value {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => default.to_string(),
    }
}
```

Call sites pass `env::var("VAR").ok().as_deref()` (so unset → `None`, empty →
`Some("")`, both fall through). `env!()` and `env::var(...).unwrap_or_else(...)`
are **insufficient** for host-URL vars and must not be used for them.

`resolve_env` is the canonical primitive for new code. The pre-existing
specific wrapper in `origa_ui` (`resolve_trailbase`, ADR-020) is an equivalent
specific-form expression of the same principle. PR #213 (open) plans to add a
matching `resolve_cdn` in `origa_ui`; once it lands, both could delegate to a
shared `resolve_env` — tracked as a follow-up, out of scope here to avoid
coupling with that PR.

### 2. `tauri/build.rs` — three reads normalized

All three reads now go through `build_config::resolve_env(...,
build_config::DEFAULT_X)`. `resolve_env` is appended to the pure
`tauri/build_config.rs` module (testable via `#[path]` from
`tauri/tests/build_config.rs`, alongside the existing `build_csp` /
`apply_merge_patch` tests).

### 3. `origa_landing` — new `build_config.rs` module + tests

`origa_landing` gains `build_config.rs` (`DEFAULT_LANDING` + `resolve_env`),
`#[path]`-included by both `build.rs` and `tests/build_config.rs`, mirroring the
`tauri` / `origa_ui` layout. Its `[[test]]` is declared **without**
`required-features` (unlike the SSR integration tests) because the module is
pure and compiles with empty default features.

### 4. Why `DEFAULT_LANDING` is not in the shared `build_defaults.rs`

`build_defaults.rs` (root) is `#[path]`-included by **both** `tauri` and
`origa_ui`. `origa_ui` never references the landing host, so a shared
`DEFAULT_LANDING` would be dead code in the `origa_ui` build binary — and the
project forbids `#[allow(dead_code)]` (`AGENTS.md`). The three crates consume
**divergent subsets** of the build-time defaults (tauri needs all three;
origa_ui needs TRAILBASE + CDN; origa_landing needs only LANDING), so no single
shared constants file fits all three without dead code.

Consequence: `DEFAULT_LANDING` is **local** in both `tauri/build_config.rs` and
`origa_landing/build_config.rs`, and `resolve_env` is duplicated across the two
(the dead-code constraint prevents sharing it either). Both are pinned to the
canonical `https://origa.uwuwu.net` and protected by drift-detection tests
(see Verification). This is the same "reduced duplication with drift guard"
trade-off ADR-009 accepted for the CSP and ADR-020 accepted for
`DEFAULT_CDN`/`DEFAULT_LANDING` staying tauri-local.

### Host-value drift fix (independent defect)

While fixing the empty-handling in `origa_landing/build.rs`, the fallback
literal was corrected from the non-canonical `https://origa.app` to
`https://origa.uwuwu.net`. `origa.uwuwu.net` is the canonical landing domain
(ADR-007, ADR-011 "landing-url-canonicalization", ADR-013/017/021, the CSP in
`tauri.conf.json:24`, and `tauri/build_config.rs`'s `DEFAULT_LANDING`).
`origa.app` appeared in exactly three places — `origa_landing/build.rs:5`,
`origa_landing/Dockerfile:11`, and the `AGENTS.md` "Landing dev" example — all
now corrected. No integration depends on `origa.app` as a distinct host; it was
a stale literal. Production CI was unaffected (it always passes
`ORIGA_LANDING_BASE_URL=https://${{ vars.ORIGA_BASE_URI }}` = the canonical
host); the stale default only affected local `docker build` without the
build-arg. This is a behavior change (the local fallback host) and is called
out separately from the empty-handling normalization.

## Env-var inventory

| Build script | Var | Status |
| --- | --- | --- |
| `tauri/build.rs` | `ORIGA_CDN_BASE_URL` | **PROTECTED** — `resolve_env` (this ADR) |
| `tauri/build.rs` | `TRAILBASE_URL` | **PROTECTED** — `resolve_env` (this ADR) |
| `tauri/build.rs` | `ORIGA_LANDING_BASE_URL` | **PROTECTED** — `resolve_env` (this ADR) |
| `origa_landing/build.rs` | `ORIGA_LANDING_BASE_URL` | **PROTECTED** — `resolve_env` (this ADR) |
| `origa_landing/build.rs` | `ORIGA_APP_BASE_URL` | **PROTECTED** — inline `.ok().filter(\|v\| !v.is_empty())` (fallback is derived from `ORIGA_APP_URI_PREFIX` + `ORIGA_BASE_URI`, not a constant, so `resolve_env` does not apply) |
| `origa_ui/build.rs` | `TRAILBASE_URL` | **PROTECTED** — `resolve_trailbase` (ADR-020) |
| `origa_ui/build.rs` | `ORIGA_CDN_BASE_URL` | **REMAINING INSTANCE** — strict `env!("ORIGA_CDN_BASE_URL", ...)`; empty compiles to `""` (vulnerable). PR #213 (ADR-023, open) replaces it with `resolve_cdn` but is not yet merged. Out of scope for this ADR. |
| `origa_ui/build.rs` | `ORIGA_VERSION` / `ORIGA_COMMIT` / `ORIGA_BUILD_DATE` | N/A — `option_env!().unwrap_or("dev"/"unknown")`; empty is the intentional sentinel, not a bug |
| `origa_ui/build.rs` | `ORIGA_PUBLIC_BASE_URL` | N/A — `option_env!().unwrap_or("")`; empty is the intentional default |
| `origa_ui/build.rs` | `ORIGA_CDN_REGION` | N/A — `option_env!().unwrap_or("auto")`; sentinel default |
| `origa_landing/build.rs` | `ORIGA_BUILD_DATE` | N/A — sitemap lastmod; falls back to git log then `1970-01-01` sentinel |
| `origa/build.rs` | `CARGO_MANIFEST_DIR` / `OUT_DIR` | N/A — Cargo-provided |

The principle's boundary: **empty-as-default applies to host-URL vars with a
production default.** Sentinel/metadata vars (`ORIGA_VERSION` → `"dev"`, etc.)
intentionally use empty or sentinel defaults and are out of scope.

## Verification / Coverage boundary

`resolve_env` is Prove-It-tested in both crates via `#[path]`-include, across
the three input shapes (unset / empty / set), asserting the **literal**
canonical host (not the constant — asserting `resolve_env(None, DEFAULT_CDN) ==
DEFAULT_CDN` would be a tautology):

- `cargo test -p origa-app --test build_config` (tauri)
- `cargo test -p origa_landing --test build_config` (landing; requires
  `ORIGA_APP_BASE_URL` to be set because `origa_landing/build.rs:16` panics
  without it — pre-existing, unrelated to this fix)

**Coverage boundary:** the build-script wiring itself
(`env::var().ok().as_deref()` → `resolve_env` → `cargo:rustc-env`) is **not**
covered by `cargo test` — build scripts are not unit-tested. Only the pure
`resolve_env` primitive is tested. This matches the pre-existing coverage gap
for `origa_ui` (same architecture); the drift-guard tests pin the constant
values so a wiring regression that changed which default is used would still be
caught.

## Conflict / sequencing with PR #213

PR #213 (`fix/android-edge-to-edge-and-cdn-fallback`, OPEN at the time of this
ADR) also modifies `tauri/build_config.rs`: it moves `DEFAULT_CDN` from a local
`const` to a `pub(crate) use defaults::{DEFAULT_CDN, DEFAULT_TRAILBASE};`
re-export from `build_defaults.rs`, and rewrites the module doc-header. This
ADR's addition (`resolve_env`) is appended at end-of-file with its own
fn-level doc-comment and does **not** touch the re-export line or doc-header
region, so the two PRs merge cleanly regardless of order: `resolve_env`'s call
sites reference `build_config::DEFAULT_CDN`, which resolves to the local `const`
on `master` and to the re-export after PR #213 — identical behavior either way.

## Alternatives Considered

### A1: Inline `.ok().filter(|v| !v.is_empty()).unwrap_or_else(...)` at each site

- **Pros:** No new helper; no `#[path]` chain.
- **Cons:** Duplicates the empty-filter idiom at every site (four in scope);
  `origa_landing/build.rs` already uses it inline for `ORIGA_APP_BASE_URL`, but
  extracting a named, testable helper is what makes Prove-It possible (build
  scripts are not unit-tested, so only an extracted pure function can be
  tested). ADR-020 established the helper pattern; deviating here would
  split the codebase across two styles.
- **Rejected.**

### A2: A dedicated shared `landing_defaults.rs` (included by tauri + origa_landing only) for a true single source of `DEFAULT_LANDING`

- **Pros:** Eliminates the `DEFAULT_LANDING` drift class entirely (the bug this
  ADR fixes *was* a drift — `origa.app` vs `origa.uwuwu.net`).
- **Cons:** Adds a second shared file for one constant; `resolve_env` would
  still be duplicated (it cannot live in `build_defaults.rs` without dead code
  in `origa_landing`, nor in a landing-only file without misleading naming for
  a generic helper). The drift is already guarded by literal-asserting tests in
  both crates, so the marginal benefit is low.
- **Rejected** in favor of local constants + drift-guard tests, consistent with
  ADR-009/020's accepted "reduced duplication with drift guard" philosophy.

### A3: Specific per-var wrappers (e.g. `resolve_landing`) instead of a generic `resolve_env`

- **Pros:** Matches `origa_ui`'s existing specific-form wrapper (`resolve_trailbase`,
  ADR-020) exactly.
- **Cons:** Three near-identical wrappers in `tauri` for one shared match arm;
  the generic form expresses the actual invariant ("empty or unset → default")
  with no loss of clarity.
- **Rejected** for new code. (`origa_ui`'s `resolve_trailbase` is kept as-is —
  refactoring it to delegate to `resolve_env` is a follow-up, out of scope to
  avoid coupling with PR #213.)

## Consequences

### Positive

- **Empty shell var can no longer drop a host from the CSP** (tauri) or empty
  `BASE_URL` (landing). All four vulnerable reads are protected.
- **One declared pattern** (`resolve_env`) for all host-URL build-script reads;
  the principle is documented so future vars follow it by default.
- **`origa.app` drift eliminated** — the landing fallback is canonical
  everywhere (build.rs, Dockerfile, AGENTS.md, tauri CSP).
- **Drift guards** pin the canonical hosts via literal-asserting tests in both
  crates.

### Negative

- **`DEFAULT_LANDING` is duplicated** across `tauri/build_config.rs` and
  `origa_landing/build_config.rs`, and `resolve_env` is duplicated across the
  two. Forced by the dead-code constraint (divergent constant subsets across
  three crates that cannot share a workspace crate). Mitigated by drift-guard
  tests and this ADR.
- **Behavior change:** local fallback for the landing host changed from
  `origa.app` to `origa.uwuwu.net` (see "Host-value drift fix"). Production
  unaffected; local `docker build` without the build-arg now matches canonical.
- **Build-script wiring remains untested** by `cargo test` (architectural limit;
  matches the existing `origa_ui` gap).

## References

- ADR-009: Tauri config parameterization via `TAURI_CONFIG` env var
- ADR-020: Propagate `TRAILBASE_URL` fallback to the `origa_ui` build script
- ADR-023 / PR #213: Propagate `ORIGA_CDN_BASE_URL` fallback to the `origa_ui` build script (**open, not yet merged** at the time of this ADR — references its planned `resolve_cdn` are forward-looking)
- ADR-011: URL Canonicalization Policy for `origa_landing` (canonical landing domain)
- The Cargo Book — Build Scripts, "Outputs of the Build Script" (`cargo:rustc-env`, `cargo:rerun-if-env-changed`)
- `origa_landing/src/components/seo.rs:6` — `env!("ORIGA_LANDING_BASE_URL")` consumer
