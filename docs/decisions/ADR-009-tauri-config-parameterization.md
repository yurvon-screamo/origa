# ADR-009: Tauri config parameterization via `TAURI_CONFIG` env var

## Status

Accepted

## Date

2026-06-18

## Context

ADR-008 (Mobile OAuth login flow on Tauri Android) introduced the production
host `app.origa.uwuwu.net` in four separate locations and tracked this as
explicit TECH-DEBT in `tauri/build.rs:1-8`:

> ```text
> // TODO(TECH-DEBT): parameterize the production host (`app.origa.uwuwu.net`)
> // via a build-time env var so it is declared in a single place. Currently the
> // same literal appears in:
> //   * tauri/tauri.conf.json  -> `security.csp` (`connect-src` + `img-src`)
> //   * tauri/capabilities/default.json -> opener allow-list
> //   * origa_ui/src/repository/trailbase_client.rs -> `env!("TRAILBASE_URL")`
> //   * origa_ui/src/pages/login/oauth_buttons.rs -> `redirect_uri` builder
> ```

A fresh audit against the current `master` (commit `ac8c22c7`) shows that the
"four locations" claim is **partially stale**: the Rust side
(`trailbase_client.rs` and `oauth_buttons.rs`) already reads `env!("TRAILBASE_URL")`
at compile time. However, that macro itself has no Rust-side fallback — the env
var must be present (or be supplied via a separate default elsewhere). This
means `app.origa.uwuwu.net` is still independently encoded in:

1. `tauri/tauri.conf.json:24` — CSP `connect-src`, `img-src`, `media-src` literals.
2. `tauri/capabilities/default.json:15-16` — opener allow-list URL globs.
3. The shell environment that supplies `TRAILBASE_URL` for `origa_ui`'s `env!()`.

The CI constraint (`.github/workflows/_build-tauri.yml:54-55`) explicitly
documents:

> `cargo tauri build` does NOT execute `origa_ui/build.rs` or `beforeBuildCommand`.
> It simply bundles the pre-built `frontendDist` from `origa_ui/dist/`.

None of the four CI build jobs (windows/linux/macos/android) propagate
`TRAILBASE_URL`, `ORIGA_CDN_BASE_URL`, or `ORIGA_LANDING_BASE_URL`. Any
build-time substitution must therefore work **without** env vars being set in
CI (falling back to production defaults compiled into Rust).

## Decision

Introduce `tauri/build_config.rs` as the parameterization root for the Tauri
side, and inject the resulting CSP at build time via Tauri's native
`TAURI_CONFIG` env var. `tauri/capabilities/default.json` is **not** mutated by
the build script — its shape is asserted by a template-based drift-detection
test instead (Cargo contract: build scripts must not modify committed source
files).

### 1. `build_config.rs` — pure module with three production defaults

```rust
pub(crate) const DEFAULT_CDN: &str      = "https://s3-proxy-production-52e3.up.railway.app";
pub(crate) const DEFAULT_TRAILBASE: &str = "https://app.origa.uwuwu.net";
pub(crate) const DEFAULT_LANDING: &str   = "https://origa.uwuwu.net";
```

Plus one pure function: `build_csp(cdn, landing, trailbase)`. It is
byte-identical to the CSP line in the committed `tauri.conf.json` when invoked
with production defaults — verified by a drift-detection test that compares
generator output against the committed file via `include_str!`.

### 2. `build.rs` — inject CSP via native `TAURI_CONFIG` env var

Tauri v2 reads the `TAURI_CONFIG` env var as a **RFC 7396 JSON Merge Patch**
and merges it on top of `tauri.conf.json` (source: `tauri-codegen/src/lib.rs`,
`tauri-build/src/lib.rs::try_build()`, `tauri-cli/src/helpers/config.rs`).

The `TAURI_CONFIG` env var itself is an internal codegen detail, not directly
documented in user-facing docs. The closest public-facing documentation is the
[`--config` flag](https://v2.tauri.app/develop/configuration-files/) which uses
the same RFC 7396 merge patch mechanism. Direct verification is via Tauri
source code (see References below).

`tauri/build.rs` now:

- Resolves `cdn` / `trailbase` / `landing` from env vars with `DEFAULT_*` fallback.
- Builds the CSP string.
- Wraps it in a JSON merge patch containing only `app.security.csp`.
- Merges the CSP patch INTO any pre-existing `TAURI_CONFIG` (set by the Tauri
  CLI via `--config`) via `build_config::apply_merge_patch` (a local RFC 7396
  implementation — serde_json 1.0.150 has NO public `merge` API on `Value`;
  verified against docs.rs/serde_json/1.0.150), then exposes the result
  in-process (`set_var`, for `tauri_build::build()`) and via
  `cargo:rustc-env=TAURI_CONFIG=...` (for `tauri::generate_context!()` in
  `origa-app`, which runs in a separate `rustc` invocation).

The hard-coded CSP in `tauri.conf.json:24` is **intentionally retained** as
defense-in-depth: if `TAURI_CONFIG` ever fails to propagate (e.g., a regression
in `build.rs`), the webview still loads with the production CSP. Setting
`csp: null` in `tauri.conf.json` would cause a silent security regression
(discussed in `tauri-apps/tauri#7881`) — never do this.

### 3. `tests/build_config.rs` — drift-detection for capabilities (no source mutation)

The opener allow-list in `tauri/capabilities/default.json` carries
environment-dependent hosts, but a build script MUST NOT mutate committed
source files (Cargo contract — modifying `tauri/capabilities/default.json` from
`build.rs` would silently corrupt the opener allow-list whenever a developer's
shell env differs from production defaults, breaking OAuth flows).

Instead, a reference template `build_capabilities_content(landing, trailbase)`
lives in `tauri/tests/build_config.rs` (test-only code). When invoked with
production defaults it is byte-identical to the committed file, asserted by
`capabilities_template_with_production_defaults_matches_committed_file`. This
turns any manual edit of the committed file into a failing test — drift is
detected without source-tree mutation.

## Alternatives Considered

### A1: Tauri native `${VAR}` substitution in `tauri.conf.json`

- **Pros:** No build-script logic; declarative.
- **Cons:** Not supported by Tauri v2 (verified against `tauri-codegen` and the
  configuration-files documentation). Tauri parses the JSON literally.
- **Rejected.**

### A2: `beforeBuildCommand` + Node script to template the JSON

- **Pros:** Cross-language, well-trodden Node-based templating.
- **Cons:** CI constraint — `_build-tauri.yml:54-55` explicitly states that
  `cargo tauri build` does NOT run `beforeBuildCommand` or `origa_ui/build.rs`.
  The substitution would silently not happen in production builds.
- **Rejected.**

### A3: `build.rs` regenerates `capabilities/default.json` in-place

- **Pros:** Eliminates the duplication of hosts in the capabilities file at
  build time.
- **Cons:** **Violates the Cargo contract** — build scripts must not mutate
  committed source files. Empirically rejected during code review: when a
  developer's shell carries `TRAILBASE_URL=https://origa.uwuwu.net` (a stale
  value that happens to equal the landing host), the regenerated file silently
  drops `https://app.origa.uwuwu.net/*` from the opener allow-list, breaking
  OAuth on the actual backend host. Idempotency holds only when env equals
  production defaults; any deviation corrupts a tracked file.
- **Rejected.** Drift detection via a template in `tests/` is the safe
  substitute (Decision §3).

### A4: `build.rs` writes capabilities to `OUT_DIR` + Tauri reads from there

- **Pros:** Honors the Cargo contract (writes go to `OUT_DIR`, not source tree).
- **Cons:** Tauri v2 reads capabilities from the static path
  `tauri/capabilities/*.json` relative to the crate root; there is no supported
  override pointing it at `OUT_DIR`. Would require patching Tauri or generating
  into source tree anyway.
- **Rejected.**

### A5: Runtime CSP override via `tauri::WebviewWindowBuilder::on_navigation`

- **Pros:** No build-time coupling; can read hosts from a runtime config.
- **Cons:** CSP is **immutable** after webview creation (WRY/WebView2/WebKitGTK
  contract). It must be set at build time; runtime edits are silently ignored.
- **Rejected.**

### A6: Minimal DRY — replace literal occurrences with a single `const` in Rust

- **Pros:** Trivial diff; no new mechanism.
- **Cons:** A `const` cannot flow into static JSON files. The CSP in
  `tauri.conf.json` and the opener allow-list in `capabilities/default.json`
  would still carry duplicated literals; only the Rust side would de-duplicate.
  Does not solve staging/dev environments where the host differs.
- **Rejected.**

### A7: Build the CSP from typed fragments via `concat!` (shorter diff lines)

- **Pros:** Each CSP directive becomes its own string literal, so editing one
  directive produces a small focused diff (vs. the current ~520-character
  single-line literal that any CSP tweak replaces wholesale). Readability in
  editors without word-wrap improves.
- **Cons:** Byte-equality with `tauri.conf.json:24` still has to be enforced by
  a test — `concat!` does not change the byte-equality requirement, it only
  changes how the literal is authored in source. The single-line form is chosen
  deliberately because `rustfmt` does not reflow string-literal contents, which
  makes the literal visually identical to the committed JSON line and lets a
  reviewer eyeball-byte-equality without running the test. Splitting via
  `concat!` would lose that visual equivalence.
- **Rejected** as the default authoring style; the option remains open if CSP
  grows to a size where single-line authoring becomes unworkable.

## Consequences

### Positive

- **Reduced duplication** for the CSP: env vars → `build_config.rs` → CSP. Any
  rename of a host now requires editing only one Rust constant (vs. hand-patching
  `tauri.conf.json`).
- **Defense-in-depth retained**: the hard-coded CSP in `tauri.conf.json` is kept
  as a fallback; if `TAURI_CONFIG` ever fails to propagate, the webview still
  loads with the production CSP.
- **Byte-equality drift detection** for both the CSP (via `build_csp` template)
  and the capabilities opener allow-list (via the test-local template). Any
  manual edit to either committed JSON that forgets to update the matching
  template will fail CI.
- **Cargo contract honored**: the build script performs no source-tree writes.
- Staging/dev environments work locally by exporting the three env vars before
  `cargo tauri dev` / `cargo tauri build` (CSP only — capabilities stay static).
- `TODO(TECH-DEBT)` removed from `build.rs`; ADR-008 negative consequence
  explicitly resolved.
- **`TAURI_CONFIG` merge semantics:** `tauri/build.rs` does NOT replace
  `TAURI_CONFIG` if it is already set (e.g., by `cargo tauri build/dev --config
  <merge>` — the standard Tauri CLI mechanism for flavor/beta configs). The CSP
  patch is merged INTO the existing value via `build_config::apply_merge_patch`
  (a local RFC 7396 implementation — serde_json 1.0.150 exposes no public
  `merge` API on `Value`, so the merge is implemented in-repo and unit-tested in
  `tauri/tests/build_config.rs`). This preserves any `--config` overrides
  (productName, identifier, bundle, plugins, devUrl, etc.). Verified against
  `tauri-cli/src/helpers/config.rs::load_config()` which sets `TAURI_CONFIG`
  via `set_var()` internally.

### Negative

- **`unsafe { std::env::set_var("TAURI_CONFIG", ...) }`** — sanctioned exception
  to the project-wide "no unsafe" rule (`AGENTS.md`). Since Rust
  edition 2024 `set_var` is `unsafe` because of potential data races in
  multi-threaded contexts. SAFETY: Cargo guarantees that exactly one `main()`
  runs per build script invocation with no spawned threads, and
  `tauri_build::build()` is a synchronous API (`pub fn build()`, no async, no
  `std::thread::spawn`) that reads `TAURI_CONFIG` via `env::var()` on the same
  thread. There is no safe alternative: the only way to feed the merge patch to
  `tauri_build::build()` is via the `TAURI_CONFIG` process env var, which
  edition 2024 forces through `set_var`. The narrow scope (single statement,
  immediately before the documented consumer, single-threaded context) keeps
  the risk surface minimal.
- **"Single source of truth" is partial, not absolute.** `app.origa.uwuwu.net`
  still independently lives in `origa_ui` via `env!("TRAILBASE_URL")` (a
  compile-time macro with no Rust-side default), and in the shell environment
  that supplies that macro. A complete rename therefore requires touching:
  1. `tauri/build_config.rs::DEFAULT_TRAILBASE` (this PR).
  2. The shell/CI env that supplies `TRAILBASE_URL` for `origa_ui` (out of scope).
  3. The hard-coded CSP in `tauri.conf.json:24` (defense-in-depth, kept).
  4. The opener allow-list in `capabilities/default.json` (kept static, asserted
     by template drift detection).
  This is a deliberate scoping decision: the `origa_ui` `env!()` macro is
  pre-existing and changing it is out of scope for this ADR.
- **CI staging does not work without updating `_build-tauri.yml`** — the four
  CI build jobs do not propagate env vars, so `DEFAULT_*` constants (= production
  values) are always used. Acceptable for production releases, but a staging
  build would need a separate CI change. Tracked as a follow-up ticket (NOT in
  this ADR's scope).
- **Capabilities file is NOT env-parameterized** — by design (Decision §3). A
  staging deployment that needs a different opener allow-list must hand-edit
  `tauri/capabilities/default.json` and update the test-local template. This is
  accepted as a smaller tech-debt than silently mutating a tracked file.
- **`DEFAULT_*` constants are tech-debt shifted**, not eliminated — production
  host values now live in Rust source instead of JSON. Accepted as a trade-off:
  Rust constants are greppable, type-checked, and covered by unit tests, whereas
  JSON literals were opaque.
- **`origa_ui` uses `env!("TRAILBASE_URL")` (compile-time macro)** — this means
  the env var MUST be set when compiling `origa_ui` (the macro has no default
  inside `origa_ui`). If a developer has a stale `TRAILBASE_URL` in their shell,
  both `origa_ui` and `tauri/build.rs` will pick it up; the injected CSP will
  then disagree with the committed capabilities file. Workaround: either
  `unset TRAILBASE_URL` (so the `build.rs` fallback kicks in, though `origa_ui`
  will then fail to compile until the macro is given a value) or
  `export TRAILBASE_URL=https://app.origa.uwuwu.net` (production value, both
  sides agree).

## Verification

| Check | Command | Result |
|---|---|---|
| Format | `cargo fmt --check` | PASS |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | 0 warnings |
| Tests | `cargo test --workspace` | 1478 passed (1469 baseline + 9 new) |
| Tests (targeted) | `cargo test -p origa-app --test build_config` | 9 passed |
| Build (WASM) | `cargo build -p origa_ui` | PASS |
| Build (Tauri) | `cargo build -p origa-app` | PASS |
| No source mutation | `git status --porcelain tauri/capabilities/default.json tauri/tauri.conf.json` after `cargo build` | empty |
| Byte-equality CSP | `build_csp_with_production_defaults_matches_committed_tauri_conf` | PASS |
| Byte-equality capabilities | `capabilities_template_with_production_defaults_matches_committed_file` | PASS |
| RFC 7396 merge unit-tests | `apply_merge_patch_*` (5 tests in `tauri/tests/build_config.rs`) | PASS |
| Merge-into-existing (mock) | `TAURI_CONFIG='{...}' cargo build -p origa-app` + inspect `target/debug/build/origa-app-*/output` | final `TAURI_CONFIG` carries BOTH mock fields (`productName`, `identifier`, `bundle`) AND `app.security.csp` — merge confirmed |

## References

- ADR-008: Mobile OAuth login flow on Tauri Android (`docs/decisions/ADR-008-mobile-oauth-tauri-android.md`)
- Tauri v2 configuration files (`--config` flag, CLI-facing): <https://v2.tauri.app/develop/configuration-files/>
- RFC 7396 JSON Merge Patch: <https://www.rfc-editor.org/rfc/rfc7396>
- Tauri codegen source: `tauri-codegen/src/lib.rs` (commit `4794a6ba`)
- Tauri CLI config loader (`TAURI_CONFIG` `set_var` call site): `tauri-cli/src/helpers/config.rs::load_config()`
- Tauri silent CSP regression discussion: <https://github.com/tauri-apps/tauri/issues/7881>
- CI constraint: `.github/workflows/_build-tauri.yml:54-55`
