# ADR-056: Flatpak for Linux distribution, AppImage retired

## Status

Accepted (2026-09-16)

## Context

The Linux AppImage bundles an Ubuntu-built WebKitGTK (linuxdeploy pulls
every shared library into the image). On rolling distributions with new
GPUs this combination breaks: on CachyOS + Intel Arc B390 (Panther Lake) +
Mesa 26.2 + Hyprland 0.56 the v0.7.19 AppImage opens a blank window — the
bundled WebKit's WebProcess dies before rendering (`Could not create
default EGL display: EGL_BAD_PARAMETER. Aborting...` in the log, with or
without the usual `WEBKIT_DISABLE_DMABUF_RENDERER` /
`WEBKIT_DISABLE_COMPOSITING_MODE` / `LIBGL_ALWAYS_SOFTWARE` mitigations).
The same binary renders fine where the system WebKitGTK is new enough:
verified by running the app against the system webkit2gtk-4.1 2.52.6, which
rendered the full UI on the same machine.

So the failure is not in the app — it is inherent to shipping a frozen
Ubuntu webview build to distributions that ship a newer graphics stack.
Meanwhile `.deb` (system WebKitGTK) works on the Debian family, and the
other platforms already use system-provided webviews (WebView2, WKWebView,
Android System WebView).

Linux has no distro-independent "evergreen" webview; the webview must come
either from the distribution (compatible there, absent elsewhere) or from a
sandboxed runtime that is kept current by its upstream.

## Decision

1. **Flatpak is the universal Linux channel.** Manifest:
   `packaging/flatpak/net.uwuwu.origa.yml` (app id `net.uwuwu.origa` —
   matches the Tauri identifier). WebKitGTK comes from the
   `org.gnome.Platform` runtime. The native shell is compiled in-sandbox
   with `cargo build --release --offline -p origa-app --features
   tauri/custom-protocol` (no tauri-cli in the sandbox; `-p` keeps the
   `origa` core crate — and with it ort/onnxruntime — out of the build).
   Crates are vendored via `packaging/flatpak/cargo-sources.json`
   (flatpak-cargo-generator, covers the rev-pinned git deps). Build is
   fully offline: every source is declared and pinned.
2. **The frontend is built OUTSIDE the sandbox** by the existing
   `build-frontend` job (rustup + trunk + wasm32 — the same pipeline the
   .deb/.exe bundles use) and handed to the flatpak module as a source
   (`origa_ui/dist`). Building the WASM in-sandbox would require vendoring
   trunk plus a wasm32 std for the extension toolchain; that work is
   deferred to a future Flathub submission slice.
3. **speech-dispatcher client is built in-sandbox** (modules
   `speech-dispatcher` 0.12.1 + `dotconf` 1.4.1 — its config-parser
   dependency; Debian pool tarballs, sha-pinned, autotools; audio
   backends / server / python disabled). At runtime the app talks to the
   HOST speech-dispatcher daemon over `$XDG_RUNTIME_DIR/speech-dispatcher`
   (granted via finish-args). `bindgen` needs libclang: the
   `org.freedesktop.Sdk.Extension.llvm20` extension is mounted at
   `/usr/lib/sdk/llvm20` and wired through `LIBCLANG_PATH` +
   `BINDGEN_EXTRA_CLANG_ARGS` (clang builtin headers are not on the
   default path).
4. **AppImage is retired.** build-linux pins `bundle.targets` to `deb`
   (config override), the AppImage fixed-name alias is dropped,
   `latest.json` no longer carries a `linux-x86_64` entry (a stale
   signature/URL entry would surface as updater TargetNotFound instead of
   a clean skip), and the release asset filter no longer picks up
   AppImages. `_build-tauri.yml` is a reusable workflow with two callers
   (tauri.yml release pipeline and ci.yml's `build-tauri` job); the change
   is consistent for both — ci.yml consumes only the job result.
5. **The Flatpak build reuses the `ORIGA_APP_STORE` gate (ADR-042).**
   Semantics are consciously widened from "Microsoft Store build" to
   "managed distribution channel without a self-updater": it patches the
   Tauri config to remove updater endpoints, compiles the updater
   commands out (`cfg(app_store)`) and hides the in-app update UI
   (`is_store_build`). Flatpak/Flathub forbid self-updating apps — updates
   arrive as a new published bundle. The build-flatpak CI job templates
   `SENTRY_DSN`, `SENTRY_ENVIRONMENT` and `ORIGA_APP_STORE=1` into a
   generated copy of the manifest (flatpak-builder sandboxes do not inherit job env). The
   tauri deep-link runtime registration comment in `tauri/src/lib.rs` no
   longer cites AppImage.
6. **`.deb` stays** as the lightweight native channel for the Debian
   family; it costs nothing (built by the same `cargo tauri build`) and is
   the most robust path where the Flathub CDN is unreliable.
7. **Distribution form**: a single-file `Origa_amd64.flatpak` bundle
   (fixed-name alias per ADR-025 addendum) attached to GitHub Releases,
   built by a dedicated `build-flatpak` job. Flathub submission is a
   separate follow-up (needs an in-sandbox frontend build,
   `<screenshots>`, `update_contact` and the Flathub review); the manifest
   structure is Flathub-ready otherwise.
8. **Data migration for users moving from .deb/old channels** is
   documented, not automated. All app state (auth, learning progress,
   caches) lives under `~/.local/share/net.uwuwu.origa/`; the Flatpak
   redirect maps it to `~/.var/app/net.uwuwu.origa/data/net.uwuwu.origa/`:

   ```sh
   mkdir -p ~/.var/app/net.uwuwu.origa/data
   cp -a ~/.local/share/net.uwuwu.origa ~/.var/app/net.uwuwu.origa/data/
   ```

## Alternatives Considered

- **Unbundled AppImage** (keep AppImage, strip WebKitGTK from it): no
  official Tauri option as of the AppImage guide (May 2026); requires
  post-processing the AppDir and repacking in CI, and still ships the
  frozen-Ubuntu GTK stack. Flatpak covers the same "any distribution"
  audience with a maintained webview.
- **CEF / bundled Chromium**: not supported by Tauri; a 100+ MB engine per
  app contradicts the project's size budget.
- **Verso/Servo as the wry webview**: experimental (tauri-runtime-verso
  PoC, 2025); not production-ready.
- **AUR package**: covers only Arch-family, shifts maintenance to the
  community.

## Consequences

- The Flatpak sandbox build is fully offline: sources are declared and
  pinned (Debian pool + crates.io via cargo-sources.json); no build-time
  network use.
- Linux users have no in-app updater by design: updates = install the new
  bundle from the release (or Flathub once published). The Windows updater
  (latest.json, windows-x86_64 entry) is unaffected.
- First Flatpak install pulls the GNOME runtime (~1.5 GB), shared across
  all Flatpak apps on the machine.
- TTS works through the host speech-dispatcher daemon; machines without a
  running speechd daemon degrade to no system-voice output (phrase audio
  from the CDN is unaffected).
- `update-version` re-points the manifest `tag:` and the appstream
  `<releases>` entry on every release; the manifest is otherwise static.
- Users migrating from the retired AppImage follow the copy commands above
  (release notes embed the copy commands); their progress/auth are
   preserved.

## Addendum (2026-09-20): .deb channel is updater-enabled

Decision 4 above (dropping the `linux-x86_64` entry so no client would
chase a stale signature/URL into `TargetNotFound`) remains historically
correct for the AppImage it described. The `.deb` channel now provides
what AppImage no longer could:

- `tauri-plugin-updater` 2.10.0 (pinned in `Cargo.lock`) ships
  `install_deb` with a privilege ladder: `pkexec` → `zenity`/`kdialog`
  graphical sudo. Machines without a polkit agent (headless, minimal
  installs) cannot complete the in-app upgrade and fall back to manual
  bundle installation.
- `tauri-cli` (≥ 2.2.0) emits the `.deb.sig` updater signature because
  `bundle.createUpdaterArtifacts` is `true` — no config change was needed.
- `_build-tauri.yml` re-exposes the `linux-signature` output and
  `generate-latest-json` publishes `linux-x86_64` → the fixed-name
  `Origa_amd64.deb` asset (content-based Minisign signature survives the
  versioned → alias rename, same as the Windows channel). A fail-loud
  guard rejects a manifest with an empty signature for either platform.

Flatpak keeps the managed-channel posture described above: no
self-updater (`ORIGA_APP_STORE=1`), updates arrive from the Flatpak
repository once published. The Consequences bullet "Linux users have no
in-app updater by design" is hereby superseded for the `.deb` channel
only.
