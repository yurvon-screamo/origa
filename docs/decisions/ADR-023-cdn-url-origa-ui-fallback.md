# ADR-023: Propagate `ORIGA_CDN_BASE_URL` fallback to the `origa_ui` build script

## Status

Accepted

## Date

2026-07-02

## Context

On an Android tablet, CDN resources (e.g. `/grammar/grammar.json`) were fetched
from `blob:http://tauri.localhost/<uuid>` instead of
`https://s3.origa.uwuwu.net/grammar/grammar.json`. The user expected direct CDN
URLs.

### Root cause

Identical bug class to ADR-020 (TRAILBASE_URL). `origa_ui/build.rs` used the
strict `env!()` macro for `ORIGA_CDN_BASE_URL`:

```rust
let cdn_base_url = env!(
    "ORIGA_CDN_BASE_URL",
    "ORIGA_CDN_BASE_URL environment variable is required. ..."
);
```

Rust's `env!()` macro panics only when the env var is **unset**; for a var **set
to an empty string** it silently returns `""`. So with
`$env:ORIGA_CDN_BASE_URL = ""` in the developer's shell, the build script did
not panic — it emitted `cargo:rustc-env=ORIGA_CDN_BASE_URL=` (empty value).

At `origa_ui` compile time, `env!("ORIGA_CDN_BASE_URL")` returned `""` at both
use sites:

1. `origa_ui/src/core/config.rs:57` — `cdn_url(path)`:
   `let base = env!("ORIGA_CDN_BASE_URL").trim_end_matches('/');` → `base = ""`
2. `origa_ui/src/repository/cache_manager.rs:89` — `build_manifest_url()`:
   same pattern → `""`

`cdn_url("/grammar/grammar.json")` then built
`format!("{}{}", "", "/grammar/grammar.json")` = `/grammar/grammar.json` — a
**relative** URL. The WebView resolved it against its own origin
(`tauri.localhost`), fetching `http://tauri.localhost/grammar/grammar.json`. The
file existed locally in `frontendDist` (from `origa_ui/dist/`), so the fetch
succeeded — and `CacheFirstCdnProvider` wrapped the response in a blob URL via
`URL::createObjectURL()`, producing the `blob:http://tauri.localhost/<uuid>`
symptom.

### Asymmetry with `tauri/build.rs`

`tauri/build.rs:29` already had the correct pattern:

```rust
let cdn = env::var("ORIGA_CDN_BASE_URL").unwrap_or_else(|_| build_config::DEFAULT_CDN.to_string());
```

with `DEFAULT_CDN = "https://s3.origa.uwuwu.net"`. But `origa_ui/build.rs` used
strict `env!()` — no fallback, no empty-handling. This was the exact asymmetry
ADR-020 resolved for `TRAILBASE_URL`.

### Relationship to ADR-020

ADR-020 introduced `build_defaults.rs` (root) with `DEFAULT_TRAILBASE` and
`origa_ui/build_config.rs` with `resolve_trailbase()`. At that time, ADR-020
§3 explicitly stated:

> *"Only `DEFAULT_TRAILBASE` is shared. `DEFAULT_CDN` and `DEFAULT_LANDING`
> stay in `tauri/build_config.rs`. Rationale: `origa_ui/build.rs` uses a
> **strict** `env!()` for the CDN (panics if unset) ... so it needs neither
> `DEFAULT_CDN` nor `DEFAULT_LANDING`."*

That rationale was correct **at the time** — the strict `env!()` did not need a
fallback constant. This ADR supersedes that statement: now that the strict
`env!()` is being replaced with `resolve_cdn()` (the same fallback pattern),
`DEFAULT_CDN` **is** needed by both crates and moves to the shared file.

## Decision

### 1. `resolve_cdn()` helper with empty-as-default fallback

`origa_ui/build_config.rs` gains `resolve_cdn()`, mirroring `resolve_trailbase()`:

```rust
pub(crate) fn resolve_cdn(env_value: Option<&str>) -> String {
    match env_value {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => defaults::DEFAULT_CDN.to_string(),
    }
}
```

`origa_ui/build.rs` replaces the strict `env!()` with:

```rust
let cdn_base_url = build_config::resolve_cdn(env::var("ORIGA_CDN_BASE_URL").ok().as_deref());
```

Empty handling is the crux (same as ADR-020): `env::var("ORIGA_CDN_BASE_URL")`
returns `Ok("")` for an empty-set var; `.as_deref()` yields `Some("")`;
`resolve_cdn` falls through to `DEFAULT_CDN` on empty.

### 2. `DEFAULT_CDN` moves to `build_defaults.rs`

`DEFAULT_CDN` is now shared by both crates (`tauri/build_config.rs` for CSP
injection, `origa_ui/build_config.rs` for the CDN fetch URL). It moves from a
tauri-local const to the root `build_defaults.rs`, re-exported via the existing
`#[path]`-include pattern:

- `build_defaults.rs`: `pub(crate) const DEFAULT_CDN: &str = "https://s3.origa.uwuwu.net";`
- `tauri/build_config.rs`: `pub(crate) use defaults::{DEFAULT_CDN, DEFAULT_TRAILBASE};`
  (was `pub(crate) use defaults::DEFAULT_TRAILBASE;` + local const).
- `origa_ui/build_config.rs`: accesses `defaults::DEFAULT_CDN` via the existing
  `#[path = "../build_defaults.rs"] mod defaults;`.

`DEFAULT_LANDING` remains tauri-local (only `tauri/build.rs` references it for
CSP injection; `origa_ui` never uses the landing host).

### 3. Build-time scheme assertion

Defense-in-depth against typoed env values (e.g.
`ORIGA_CDN_BASE_URL="s3.origa.uwuwu.net"` missing the scheme):

```rust
assert!(
    cdn_base_url.starts_with("https://") || cdn_base_url.starts_with("http://"),
    "ORIGA_CDN_BASE_URL must be an http(s) URL, got: {cdn_base_url}"
);
```

Both `https://` and `http://` are accepted unconditionally. `http://` is a
legitimate scheme for local test/dev backends — e.g. E2E CI runs
`trunk build --release` with `ORIGA_CDN_BASE_URL=http://localhost:8080` pointing
at a local mock backend. This is safe because `resolve_cdn()` (§1) returns
`DEFAULT_CDN` (`https://s3.origa.uwuwu.net`) whenever the env var is unset OR
empty, so `http://` can only reach the assertion through an **explicit**
non-empty override — which is exactly the intent. The assertion still catches
the failure mode it was added for: a schemeless value that would reintroduce
the relative-URL bug this ADR fixes.

An earlier revision gated `http://` behind `cfg!(debug_assertions)`:

```rust
// ... || cfg!(debug_assertions) && cdn_base_url.starts_with("http://")
```

This was **rejected**: `cfg!(debug_assertions)` reflects the build *profile*
(dev vs release), not the runtime *environment*. E2E CI legitimately combines
`--release` with a `http://localhost` backend, which the debug-only gate
blocked — panicking in the "E2E Build" CI job. HTTPS enforcement in production
is the deploy environment's responsibility (Dockerfile `build_args`, CI GitHub
vars), not the build script's; the build script only guards against
*schemeless* values.

### 4. Prove-It regression tests

`origa_ui/tests/build_config.rs` gains three tests for `resolve_cdn`,
mirroring the `resolve_trailbase` tests:

- `None` → production default.
- `Some("")` → production default (the bug case).
- `Some("https://cdn.staging.example.com")` → that value (pass-through).

The `None` case doubles as a drift guard for `DEFAULT_CDN`.

## Alternatives Considered

### Inline the fallback in `build.rs` without a shared constant

```rust
let cdn_base_url = env::var("ORIGA_CDN_BASE_URL")
    .ok().filter(|v| !v.is_empty())
    .unwrap_or_else(|| "https://s3.origa.uwuwu.net".to_string());
```

- **Pros:** Trivial diff; no `build_defaults.rs` change.
- **Cons:** Duplicates the production CDN host literal. It already lives in
  `tauri/build_config.rs` as `DEFAULT_CDN` — a rename would require touching two
  places. Moving it to `build_defaults.rs` (the shared file ADR-020 established
  for exactly this purpose) is the minimal correct DRY.
- **Rejected.**

## Consequences

### Positive

- **CDN resources fetched from the correct origin** (`https://s3.origa.uwuwu.net`)
  instead of `blob:http://tauri.localhost`. Both `cdn_url()` and
  `build_manifest_url()` resolve to absolute URLs.
- **`cargo build -p origa_ui` works without `ORIGA_CDN_BASE_URL` in the shell** —
  previously this was a compile error (`env!()` panic on unset).
- **Build-time scheme validation** catches typoed URLs early.
- **Single source of truth** for `DEFAULT_CDN`: root `build_defaults.rs`,
  consumed by both crates. Supersedes ADR-020 §3's statement that `DEFAULT_CDN`
  stays tauri-local.

### Negative

- **Behavior change: compile error → silent production default.** Before this
  ADR, building `origa_ui` with `ORIGA_CDN_BASE_URL` **unset** failed loudly.
  After it, such a build silently defaults to the production CDN. Same trade-off
  as ADR-020 (TRAILBASE) and ADR-009 (CSP). **Mitigation:** the scheme assertion
  catches malformed (schemeless) values; local developers targeting a local CDN
  must explicitly `export ORIGA_CDN_BASE_URL=http://localhost:8080`.

## Verification

| Check | Command | Result |
| --- | --- | --- |
| Format | `cargo fmt --check` | PASS |
| Lint | `cargo clippy -p origa_ui -p origa-app --all-targets -- -D warnings` | 0 warnings |
| Tests (new, CDN) | `cargo test -p origa_ui --test build_config` | 6 passed (3 trailbase + 3 cdn) |
| Tests (existing, re-export) | `cargo test -p origa-app --test build_config` | 9 passed (`DEFAULT_CDN` re-export preserves value) |

## References

- ADR-020: Propagate `TRAILBASE_URL` fallback to the `origa_ui` build script (`docs/decisions/ADR-020-trailbase-url-origa-ui-fallback.md`)
- ADR-009: Tauri config parameterization via `TAURI_CONFIG` env var (`docs/decisions/ADR-009-tauri-config-parameterization.md`)
- `origa_ui/src/core/config.rs:57` — `cdn_url()` use site
- `origa_ui/src/repository/cache_manager.rs:89` — `build_manifest_url()` use site
