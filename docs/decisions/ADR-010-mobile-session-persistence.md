# ADR-010: Mobile session persistence via `tauri-plugin-store`

## Status

Accepted

## Date

2026-06-19

## Context

After PR #159 fixed the mobile OAuth login flow on Tauri Android (external
browser opens and returns to the app via deep-link), users reported a new
symptom: the browser opens and returns to the app, but a **white screen** is
shown briefly before redirecting back to `/login`. The session is not preserved
after returning from the external browser.

### Root cause analysis (3 factors)

**RC-1 (PRIMARY):** Android kills the Origa process in the background (memory
pressure) when the user navigates to the external browser. The WebView
`localStorage` — the session storage used by the web build — relies on the
Chromium DOMStorage commit queue, which has a known bug where queued writes are
**lost on hard process kill** (Chromium issue 479767). After the process is
killed and cold-started by the deep-link, the session that was written to
`localStorage` during the OAuth exchange is gone.

**RC-2:** `window.location().set_href("/home")` in
`oauth_listeners.rs:103/263` caused a full WebView reload, discarding all
reactive state. The redirect Effect in `login/mod.rs:42-49` only covered the
email/password path (where `user.set` is called explicitly), not the OAuth path
(where the session is written asynchronously).

**RC-3:** `App()` initialization ordering race — `check_session()` was spawned
first, fast-exited when no session was found, and caused `ProtectedRoute` to
render `Login`. The OAuth deep-link callback arrived later via IPC and set the
session, but the user had already seen the Login page flash.

### Prior art

- **ADR-008** established the mobile OAuth flow on Tauri Android (deep-link
  scheme `origa://`, PKCE, `tauri-plugin-opener` for external browser).
- **ADR-009** parameterized the Tauri config (CSP, capabilities) via
  `TAURI_CONFIG` env var.

Neither addressed session persistence under process kills.

## Decision

### AD-1: `tauri-plugin-store` with explicit `Store::save()` fsync

Replace WebView `localStorage` as the authoritative session store on Tauri with
`tauri-plugin-store = "2"`, persisting to a single file `auth.json` in the
app's native data directory (`app_data_dir`). This directory survives process
kills on Android (internal storage).

The plugin's default `auto_save: 100ms` debounce is **insufficient** for
reliability under process kills — a write can be in the debounce window when the
OS kills the process. Every write command therefore calls `Store::save()`
explicitly to fsync to disk before returning:

```rust
fn write_value(app: &AppHandle, key: &str, value: &str) -> Result<(), String> {
    let store = app.store(STORE_FILE)?;
    store.set(key, value.to_string());
    store.save()?;  // EXPLICIT fsync — critical for process-kill durability
    Ok(())
}
```

Three generic IPC commands (`auth_store_get`, `auth_store_set`,
`auth_store_delete`) operate on arbitrary string keys in `auth.json`, serving
both the session and the PKCE verifier.

### AD-2: In-memory cache as `OnceLock<Arc<RwLock<Option<TrailBaseSession>>>>`

A process-global cache (`SESSION_CACHE`) acts as the single source of truth for
the current run. It is populated by `get_session_async()` on the first cold-path
access (Tauri: from the store; web: from localStorage) and kept in sync by
`set_session_async()` / `clear_session_async()`.

- **Hot-path reads** (every authenticated API request in
  `_request_with_auth_impl`) use the sync `get_session()`, which reads the cache
  first. This avoids an IPC round-trip per request.
- **Cold-path writes** (login, OAuth exchange, refresh, logout) use the async
  `*_async()` variants, which write to the store (Tauri) or localStorage (web),
  then update the cache.

The cache uses `std::sync::RwLock` (not `tokio::sync::RwLock`) because
`origa_ui` is a CSR-only WASM crate with no `tokio` dependency, and WASM is
single-threaded so std locks are safe. The lock is never held across an `.await`
point.

### AD-3: SPA navigate via `App()` Effect

The redirect Effect is moved from `Login` (which can unmount) to `App()` (always
mounted). It watches `auth_store.is_authenticated()` and navigates to `/home`
via `use_navigate()` (SPA, no reload) when the user becomes authenticated. This
covers **both** paths:

- Email/password: `login()` calls `self.user.set(Some(user))` → Effect fires.
- OAuth: `set_oauth_session()` writes the session asynchronously and calls
  `self.user.set(Some(user))` → Effect fires.

The old `window.location().set_href("/home")` calls in `handle_oauth_result()`
and `redirect_to_home()` are removed entirely — the Effect replaces them.

### AD-4: `App()` initialization ordering

The init sequence is reordered so that OAuth callback checks run before the
session check, minimizing the window where Login is visible on a cold-start
OAuth flow:

1. `migrate_session_to_store_if_needed()` — one-time migration for users
   upgrading from pre-ADR-010 builds (localStorage → store).
2. `check_session()` — reads from the store (populated by migration if needed).
3. `check_url_oauth_callback()` — web-build URL-fragment OAuth callback.
4. `setup_oauth_listener()` — Tauri deep-link listener + pending link check.

Steps 2–4 run concurrently (all use `spawn_local`), but the migration in step 1
is `await`ed before step 2 within the same async task, ensuring the store is
populated before `check_session` reads it.

### Web fallback

All three core functions (`set_session_async`, `get_session_async`,
`clear_session_async`) branch on `tauri::is_tauri()`:

```rust
if tauri::is_tauri() {
    // IPC → tauri-plugin-store (auth.json, explicit save)
} else {
    // localStorage (web build, E2E tests)
}
```

The sync functions (`get_session`, `set_session`, `clear_session`) continue to
read/write localStorage + cache and serve as the web-build primary path and
Tauri hot-path cache fallback.

### PKCE verifier persistence

The PKCE verifier is critical: it must survive the process kill between the
OAuth redirect (external browser) and the deep-link callback (app cold start).
It is migrated to the same store (`auth.json`, key `pkce_verifier`) using the
generic IPC commands. The write happens **before** opening the external browser
(inside `open_oauth_url`, now async) to ensure the IPC `Store::save()` completes
before the app goes to the background.

## Alternatives Considered

### A1: Increase localStorage flush frequency

- **Pros:** Minimal code change.
- **Cons:** Chromium DOMStorage bug 479767 is a WebView-level issue with no
  reliable workaround. There is no `localStorage.flush()` API. The commit queue
  is opaque and can be lost under hard kills regardless of write frequency.
- **Rejected.**

### A2: Custom JSON file in `app_data_dir` via `std::fs`

- **Pros:** Full control over fsync semantics.
- **Cons:** Reimplements what `tauri-plugin-store` already provides (atomic
  write, cross-platform path resolution, thread-safe access). More code to
  maintain and test.
- **Rejected** in favor of the maintained plugin.

### A3: `tokio::sync::RwLock` for the cache

- **Pros:** Matches the plan's original design.
- **Cons:** `origa_ui` has no `tokio` dependency (it is a WASM crate using
  `wasm-bindgen-futures` / `spawn_local`). Adding `tokio` to a CSR-only WASM
  crate is architecturally wrong and increases bundle size. WASM is
  single-threaded, so `std::sync::RwLock` is equivalent and allows both sync and
  async access patterns.
- **Rejected.** `std::sync::RwLock` is the correct choice for WASM.

### A4: Keep redirect in `Login` Effect, add OAuth-specific Effect there

- **Pros:** Smaller diff.
- **Cons:** The Login page can unmount (e.g., if `ProtectedRoute` switches to a
  loading state during the OAuth callback). An unmounted Effect stops watching
  signals, so the redirect would never fire. App() is always mounted.
- **Rejected.**

### A5: Block `check_session` until deep-link resolves (500ms timeout)

- **Pros:** Eliminates the Login flash on cold-start OAuth entirely.
- **Cons:** Adds artificial latency to every cold start (even non-OAuth starts).
  The concurrent execution of `check_session` and deep-link processing already
  minimizes the flash to a few milliseconds. The `is_oauth_loading` overlay
  covers the remaining window.
- **Rejected** as unnecessary complexity; can be revisited if the flash is
  noticeable in user testing.

## Consequences

### Positive

- **Session survives process kills on Android.** The session and PKCE verifier
  are persisted to the native filesystem (`app_data_dir/auth.json`) with explicit
  `Store::save()` fsync, independent of WebView `localStorage` reliability.
- **SPA navigation preserves reactive state.** Replacing `set_href` with
  `use_navigate()` avoids a full WebView reload after OAuth success.
- **Both auth paths unified.** A single Effect in App() handles navigation for
  email/password and OAuth, eliminating the previous gap where OAuth callbacks
  had no redirect trigger.
- **Web build unaffected.** All async functions branch on `tauri::is_tauri()`;
  the web build continues to use localStorage with no behavior change. E2E tests
  pass without modification.
- **Existing users migrated transparently.** `migrate_session_to_store_if_needed()`
  runs once on app start and moves any existing localStorage session to the
  store.

### Negative

- **Additional IPC round-trip** for session writes on Tauri (one `invoke` per
  write). Mitigated by the in-memory cache: hot-path reads never touch the store.
- **`parse_tokens_from_url` no longer persists the session** (SRP change). The
  caller (`handle_oauth_callback`) now explicitly calls `set_session_async`.
  This is cleaner but is a behavior change that any future caller must be aware
  of.
- **`tauri-plugin-store = "2"` added to workspace deps.** The plugin is
  well-maintained (Tauri core ecosystem) and adds minimal binary size.
- **PKCE verifier write is now async** (`open_oauth_url` is async). The OAuth
  button click handler wraps the call in `spawn_local`, ensuring the UI thread is
  not blocked. The `await` before opening the browser guarantees the store write
  completes, which is the whole point.
- **IPC store round-trip not covered by unit tests.** The `extract_string`
  function (the site of the H1 double-encoding bug) is tested in isolation
  with a regression test, but the full IPC path (`store_write` → Tauri command →
  `Store::save()` → `store_read` → deserialization) requires a Tauri runtime and
  cannot be exercised in `cargo test`. Integration testing on a physical device
  or Tauri mock is needed for full confidence. Tracked as TODO(TECH-DEBT).

## Verification

| Check | Command | Result |
|---|---|---|
| Format | `cargo fmt --check` | PASS |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | 0 warnings |
| Tests | `cargo test --workspace` | 1488 passed, 0 failed |
| Tests (auth_store) | `cargo test -p origa-app` | 4 passed (JsonValue extraction + double-encode regression) |
| Tests (session cache) | `cargo test -p origa_ui --lib` | 135 passed (includes cache_round_trip) |
| Build (WASM) | `cargo build -p origa_ui` | PASS (0 warnings) |
| Build (Tauri) | `cargo build -p origa-app` | PASS |
| Store plugin registered | `tauri/src/lib.rs` `.plugin(tauri_plugin_store::Builder::default().build())` | PASS |
| Explicit `Store::save()` | `tauri/src/auth_store.rs` every write/delete command | PASS |
| No double JSON encoding | `extract_string_does_not_double_encode` test | PASS |
| Web fallback | `tauri::is_tauri()` branch in all 3 async functions | PASS |
| Grep audit | `rg "get_session"` / `set_session` / `clear_session` in `origa_ui/src` | only legacy wrappers + sync hot-path reads |
| Idempotency | `git status --porcelain tauri/tauri.conf.json tauri/capabilities/default.json` after build | empty |

## Tauri v2 Android gotchas (reference)

These findings informed the decisions above:

1. **localStorage does not persist under process kill.** Chromium DOMStorage
   commit queue (bug 479767) loses queued writes on hard kill. The WebView
   `localStorage` is therefore unsuitable for critical session data on Android.
2. **`tauri-plugin-store` default `auto_save: 100ms` is insufficient.** The
   debounce window can straddle a process kill. Every write must call
   `Store::save()` explicitly to fsync.
3. **`tauri-plugin-store` persists to `app_data_dir` (native FS).** This is the
   correct path for critical persistence on Android — it survives process kills
   and app restarts.
4. **`app.deep_link().get_current()` works on Android cold start.** The pending
   deep-link URL is available in the Tauri `setup` callback at app launch.
5. **`deep-link://new-url` event is emitted on app load.** As of
   `tauri-plugin-deep-link` v2.0.0-rc.5+ (PR
   tauri-apps/plugins-workspace#1770), the event fires both for live deep-links
   and for the cold-start link.
6. **`AndroidManifest` `launchMode="singleTask"` is correct for deep-link.**
   `onNewIntent` delivery works as expected for deep-link callbacks when the app
   is already running.
7. **`window.location().set_href` causes a full WebView reload** with loss of all
   reactive state. SPA navigation via `use_navigate()` preserves the Leptos
   reactive graph.

## References

- ADR-008: Mobile OAuth login flow on Tauri Android (`docs/decisions/ADR-008-mobile-oauth-tauri-android.md`)
- ADR-009: Tauri config parameterization (`docs/decisions/ADR-009-tauri-config-parameterization.md`)
- `tauri-plugin-store` docs: <https://docs.rs/tauri-plugin-store/2/>
- Tauri Store plugin guide: <https://v2.tauri.app/plugin/store/>
- Chromium DOMStorage bug: <https://issues.chromium.org/issues/479767>
- PR #159: Mobile OAuth login flow fix
- PR #160: CSP parameterization
