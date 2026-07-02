# ADR-020: Propagate `TRAILBASE_URL` fallback to the `origa_ui` build script

## Status

Accepted

## Date

2026-07-02

## Context

In the Tauri desktop app (`cargo tauri dev`, Windows), all auth API requests
went to `http://tauri.localhost/api/auth/v1/login` instead of
`https://app.origa.uwuwu.net/api/auth/v1/login`. The WebView resolved the
relative path `/api/auth/v1/login` against its own origin.

### Root cause

Two sites in `origa_ui` read the compile-time macro `env!("TRAILBASE_URL")`:

1. `origa_ui/src/repository/trailbase_client.rs:43` — base URL for every fetch:
   `format!("{}{}", self.base_url, path)` where
   `self.base_url = trailbase_url().to_string()` and `trailbase_url()` returns
   `env!("TRAILBASE_URL")`.
2. `origa_ui/src/repository/trailbase_auth.rs:63` — JWT issuer validation:
   `iss == "trailbase" || iss == env!("TRAILBASE_URL")`.

`origa_ui/build.rs` did **not** emit `cargo:rustc-env=TRAILBASE_URL=...`. It
handled `ORIGA_CDN_BASE_URL` (strict `env!()` with a panic message),
`ORIGA_VERSION`, `ORIGA_COMMIT`, `ORIGA_BUILD_DATE`, `ORIGA_PUBLIC_BASE_URL`,
`ORIGA_CDN_REGION` — but `TRAILBASE_URL` was missing entirely.

The `env!()` macro panics only when an env var is **unset**; for a var **set to
an empty string** it silently returns `""`. So when a developer ran
`cargo tauri dev` with `TRAILBASE_URL=""` in the shell, the macro compiled to
`""`, producing two distinct symptoms:

1. **Primary:** `format!("{}{}", "", "/api/auth/v1/login")` = a relative URL,
   which the WebView resolved against its own origin `tauri.localhost`.
2. **Secondary:** in `trailbase_auth.rs:63`, `iss == ""` never matched the real
   issuer, so every token triggered `tracing::warn!("Untrusted JWT issuer: ...")`
   (`trailbase_auth.rs:65`).

This was an asymmetry with `tauri/build.rs:31`, which already resolved the same
var with a fallback:
`env::var("TRAILBASE_URL").unwrap_or_else(|_| build_config::DEFAULT_TRAILBASE.to_string())`
where `DEFAULT_TRAILBASE = "https://app.origa.uwuwu.net"` (in
`tauri/build_config.rs:22`). The `tauri` crate had a fallback; `origa_ui` did
not.

### Relationship to ADR-009

ADR-009 ("Tauri config parameterization via `TAURI_CONFIG` env var")
parameterized the `tauri`-side CSP and explicitly tracked the `origa_ui`
`env!()` macro as a **residual sync point it did not close**:

- *"Negative — `origa_ui` uses `env!("TRAILBASE_URL")` (compile-time macro) —
  this means the env var MUST be set when compiling `origa_ui` (the macro has
  no default inside `origa_ui`)."* (ADR-009, "Consequences → Negative")
- *"This is a deliberate scoping decision: the `origa_ui` `env!()` macro is
  pre-existing and changing it is out of scope for this ADR."* (ADR-009,
  "Consequences → Negative")

This ADR closes exactly that residual sync point. The omission from ADR-009 was
an oversight in scope, not a deliberate long-term design: ADR-009 reduced
duplication on the `tauri` side and deferred the `origa_ui` side, but never
circled back. The bug above is the direct consequence of that deferral.

## Decision

### 1. `origa_ui/build.rs` emits `cargo:rustc-env=TRAILBASE_URL=<value>` with a fallback

`origa_ui/build.rs` now resolves the URL via a pure helper and emits it:

```rust
let trailbase = build_config::resolve_trailbase(env::var("TRAILBASE_URL").ok().as_deref());
println!("cargo:rustc-env=TRAILBASE_URL={trailbase}");
println!("cargo:rerun-if-env-changed=TRAILBASE_URL");
```

The fallback treats **both unset and empty** as "use default". Empty handling is
the crux: a naive `unwrap_or_else` (fallback only on `Err`) would still emit
`cargo:rustc-env=TRAILBASE_URL=` (empty) for the `Ok("")` case, leaving the bug
in place. The `resolve_trailbase` helper explicitly falls through to the default
on empty:

```rust
pub(crate) fn resolve_trailbase(env_value: Option<&str>) -> String {
    match env_value {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => defaults::DEFAULT_TRAILBASE.to_string(),
    }
}
```

### 2. `cargo:rustc-env` precedence — the fix is crate-wide, no source edits

Per The Cargo Book ("Outputs of the Build Script"), `cargo:rustc-env=VAR=VALUE`
**unconditionally sets** the variable for the rustc invocation that compiles the
package. This is a **different mechanism** from `.cargo/config.toml [env]`,
which does not override an existing host value without `force = true`. There is
no such caveat for `cargo:rustc-env`: it sets the variable regardless.

Therefore, once `origa_ui/build.rs` emits `cargo:rustc-env=TRAILBASE_URL=<non-empty>`,
`env!("TRAILBASE_URL")` resolves to that non-empty value **even if the host had
`TRAILBASE_URL=""`**. Because `cargo:rustc-env` is crate-wide, both call sites
(`trailbase_client.rs:43` and `trailbase_auth.rs:63`) resolve to the emitted
value simultaneously. **No change is required in either source file** — keeping
`env!("TRAILBASE_URL")` exactly as is. This strengthens the fix: a single
build-script line closes both symptoms.

### 3. Single source of truth — share only `DEFAULT_TRAILBASE`

A root file `build_defaults.rs` (workspace root, alongside `Cargo.toml`) holds
the one constant needed by **both** crates:

```rust
pub(crate) const DEFAULT_TRAILBASE: &str = "https://app.origa.uwuwu.net";
```

Both build scripts reach it via the existing `#[path]`-include pattern (build
scripts cannot `use` workspace crates — they compile with `std` +
`[build-dependencies]` only; the pattern is established in `tauri/build.rs:20-21`):

- `tauri/build_config.rs`: `#[path = "../build_defaults.rs"] mod defaults; pub(crate) use defaults::DEFAULT_TRAILBASE;`
- `origa_ui/build_config.rs`: `#[path = "../build_defaults.rs"] mod defaults;`

Only `DEFAULT_TRAILBASE` is shared. `DEFAULT_CDN` and `DEFAULT_LANDING` stay in
`tauri/build_config.rs`. Rationale: `origa_ui/build.rs` uses a **strict**
`env!()` for the CDN (panics if unset — intentional, see `AGENTS.md`:
"`ORIGA_CDN_BASE_URL` установлена перед сборкой") and never references the
landing host, so it needs neither `DEFAULT_CDN` nor `DEFAULT_LANDING`. Moving
all three into the shared root file would make `DEFAULT_CDN`/`DEFAULT_LANDING`
unused constants inside the `origa_ui` build binary, and the project forbids
`#[allow(dead_code)]` (`AGENTS.md` "🚫 НИКОГДА ... `#[allow(dead_code)]`").
Sharing only the genuinely-shared constant is the minimal correct DRY.

### 4. Prove-It regression tests

`origa_ui/tests/build_config.rs` unit-tests `resolve_trailbase` across the three
input shapes via `#[path]`-include (mirroring `tauri/tests/build_config.rs`):

- `None` → production default.
- `Some("")` → production default (the bug case).
- `Some("https://staging.example.com")` → that value (pass-through).

The `None` case doubles as a drift guard for `DEFAULT_TRAILBASE`: the fallback
branch returns exactly that constant, so any change to its value breaks the
test.

## Alternatives Considered

### A1: Duplicate `DEFAULT_TRAILBASE` locally in `origa_ui`

- **Pros:** Trivial diff; no `#[path]` chaining across packages.
- **Cons:** Reintroduces the duplication ADR-009 set out to eliminate. A rename
  of the production host would require touching two Rust constants again — the
  exact tech-debt the shared file exists to remove.
- **Rejected.**

### A2: `#[path = "../tauri/build_config.rs"]` from `origa_ui` (cross-crate reference)

- **Pros:** No new file; reuses the existing module wholesale.
- **Cons:** Directionally wrong — `origa_ui` would depend on the `tauri/`
  directory, inverting the natural dependency direction. Worse, the whole
  `tauri/build_config.rs` includes CSP-specific functions (`build_csp`,
  `apply_merge_patch`) that `origa_ui` does not use, which would be dead code
  in the `origa_ui` build binary (and the project forbids `#[allow(dead_code)]`).
- **Rejected.**

### A3: Move the whole `tauri/build_config.rs` to the root, share all three constants

- **Pros:** Maximally DRY; one file for all build-time defaults.
- **Cons:** Same dead-code problem as A2 — `origa_ui` does not use
  `DEFAULT_CDN`/`DEFAULT_LANDING`, so including the whole module produces unused
  constants. Would require `#[allow(dead_code)]`, which is forbidden.
- **Rejected.** The split (shared `DEFAULT_TRAILBASE` at root; tauri-local
  `DEFAULT_CDN`/`DEFAULT_LANDING`) is the only shape that is both DRY and
  lint-clean.

### A4: Patch the call sites (`trailbase_client.rs`, `trailbase_auth.rs`) to fall back at runtime

- **Pros:** Defense-in-depth; survives any build-script regression.
- **Cons:** Duplicates the fallback logic in two source files, and the
  `env!("TRAILBASE_URL")` value is baked at compile time anyway. The build-script
  fix is the correct single point of control; runtime fallback would mask future
  build misconfiguration instead of failing loudly.
- **Rejected** as the primary fix. (The existing `env!()` sites are kept
  unchanged per Decision §2.)

### A5: `.cargo/config.toml [env]` with `force = true`

- **Pros:** Declarative; no build-script code.
- **Cons:** The repo's `.cargo/config.toml` has no `[env]` section by design
  (env vars come from the shell or build scripts). Adding `force = true` for
  `TRAILBASE_URL` would silently override a developer's local backend, and is
  inconsistent with how every other build-time var (`ORIGA_CDN_BASE_URL`, etc.)
  is handled — via build scripts. Also does not solve the empty-string case
  without `force`.
- **Rejected.**

## Consequences

### Positive

- **Both symptoms fixed** by a single build-script line: auth fetches go to the
  absolute backend URL, and JWT issuer validation matches again (no more
  spurious "Untrusted JWT issuer" warnings).
- **`cargo build -p origa_ui` works without `TRAILBASE_URL` in the shell** —
  previously this was a compile error (`env!()` panic on unset).
- **Single source of truth** for the production TrailBase host: root
  `build_defaults.rs`, consumed by both crates. A rename touches one constant.
- **Closes the residual sync point ADR-009 explicitly left open** (ADR-009
  "Consequences → Negative").
- **DRY without lint debt:** only the genuinely-shared constant is shared;
  `DEFAULT_CDN`/`DEFAULT_LANDING` stay tauri-local; no `#[allow(dead_code)]`
  anywhere.
- **CI unchanged:** `TRAILBASE_URL` is already propagated in `ci.yml` (lines
  55, 122, 175, 214, 247, 455), `docker.yml` (line 72), and `tauri.yml`
  (line 94). The fix is purely additive (a build-time fallback) and can only
  make CI more robust, never break it.

### Negative

- **Behavior change: compile error → silent production default.** Before this
  ADR, building `origa_ui` with `TRAILBASE_URL` **unset** failed loudly at
  compile time (`env!()` panic) — a clear signal. After it, such a build
  **silently** defaults to `https://app.origa.uwuwu.net`: a local developer who
  forgets `TRAILBASE_URL=http://localhost:4000` will get a WASM/Tauri binary
  that silently talks to production auth. This is the same trade-off ADR-009
  accepted for the `tauri` side (CSP defaults to production when env vars are
  absent), so it is consistent and defensible — but it must be known.
  **Mitigation:** local developers targeting a local backend MUST explicitly
  export `TRAILBASE_URL=http://localhost:4000` (or the relevant local host)
  before `cargo tauri dev` / `trunk serve`. (`AGENTS.md` lists `TRAILBASE_URL`
  as an optional build-time variable; the explicit local-backend requirement
  follows from the silent-default behavior introduced here.)
- **`DEFAULT_CDN`/`DEFAULT_LANDING` remain tauri-local.** A truly complete
  single source of truth for all three hosts is not achieved. This is
  deliberate (see Decision §3 / Alternative A3) and only matters if `origa_ui`
  ever needs those defaults — currently it does not.
- **Cross-package `#[path]` reference.** `origa_ui/build_config.rs` reaches
  `../build_defaults.rs` outside its own package. `#[path]` resolves relative to
  the file containing the `mod` declaration, so it is stable, but a reader must
  follow the chain. Mitigated by `cargo:rerun-if-changed=../build_defaults.rs`
  in `origa_ui/build.rs` and by the module doc-comments pointing at this ADR.
- **`tauri/build.rs` reads `TRAILBASE_URL` for the CSP via `env::var()`** (line
  31), while `origa_ui/build.rs` now reads it for the fetch URL via the build
  script. Both honor the same env var with the same fallback, but through two
  `#[path]`-include sites. Drift between the two resolutions is prevented by the
  shared `DEFAULT_TRAILBASE` constant.

## Verification

| Check | Command | Result |
| --- | --- | --- |
| Format | `cargo fmt --check` | PASS |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | 0 warnings (no dead code) |
| Tests (new) | `cargo test -p origa_ui --test build_config` | 3 passed |
| Tests (existing, unchanged) | `cargo test -p origa-app --test build_config` | all passed (`#[path]` re-export preserves `build_config::DEFAULT_TRAILBASE`) |
| Build without env var | `TRAILBASE_URL` unset → `cargo build -p origa_ui` | PASS (previously compile error) |
| Reasoning (end-to-end) | `env!("TRAILBASE_URL")` resolves to non-empty via `cargo:rustc-env` at both sites | fetch URL is absolute; issuer validation matches |

## References

- ADR-009: Tauri config parameterization via `TAURI_CONFIG` env var (`docs/decisions/ADR-009-tauri-config-parameterization.md`)
- The Cargo Book — Build Scripts, "Outputs of the Build Script" (`cargo::rustc-env`): <https://doc.rust-lang.org/cargo/reference/build-scripts.html>
- The Cargo Book — Configuration (`[env]` section, `force` flag): <https://doc.rust-lang.org/cargo/reference/config.html>
- `origa_ui/src/repository/trailbase_client.rs:43` — fetch base URL
- `origa_ui/src/repository/trailbase_auth.rs:63` — JWT issuer validation
