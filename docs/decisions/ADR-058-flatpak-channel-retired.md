# ADR-058: Flatpak channel retired, Linux = .deb + .rpm + AUR

## Status

Accepted (2026-09-20)

Supersedes the Flatpak half of ADR-056 (the AppImage retirement made there
stands). Follows #627 (deb in-app updater restored).

## Context

ADR-056 made Flatpak the primary universal Linux channel: its WebKitGTK
comes from the GNOME runtime and works on every distribution, while the
bundled-Ubuntu-WebKitGTK AppImage died on rolling setups. The natural end
state of that decision was a Flathub submission.

In May 2026 Flathub tightened its Generative AI policy: submission pull
requests must not be generated or automated with AI tools, and the policy
now covers the submission itself (manifest, metadata, patches, build
scripts) as well as applications containing AI-generated or AI-assisted
content. Origa is developed with heavy AI assistance; the carve-out for
"mature, well-maintained projects" is discretionary and repeatedly
violating the policy means a permanent submission ban. Maintaining a
Flathub submission therefore carries an unacceptable policy risk.

At the same time the value of the Flatpak bundle dropped: the in-app
self-updater is back for the .deb channel (#627), the .rpm channel now
adds RHEL/Fedora with the same in-app updater, and Arch users are covered
by an AUR package updated via their regular AUR helper (`yay -Syu`). The
sandboxed Flatpak build (GNOME runtime pinning, cargo vendor manifest
regeneration, SENTRY templating into a generated manifest copy) was the
most expensive Linux artifact to build and support, for a channel whose
users had no auto-update at all.

## Decision

1. **The Flatpak channel is retired.** `build-flatpak` is removed from
   the release workflow, `packaging/flatpak/` and
   `scripts/flatpak-cargo-generator.py` are removed, and
   `.flatpak` is no longer a release asset.
2. **Linux distribution = .deb + .rpm + AUR.**
   - `.deb` (Debian/Ubuntu) — in-app updater, `pkexec dpkg` (#627).
   - `.rpm` (RHEL/Fedora) — in-app updater, `pkexec rpm -U`. Built in the
     same `cargo tauri build` run as the .deb: tauri-bundler patches the
     bundle type into the binary per-bundle, so deb-installed clients pick
     the `linux-x86_64-deb` manifest key and rpm-installed clients the
     `linux-x86_64-rpm` key (tauri-cli ≥ 2.2.0 signs both package types).
   - AUR `origa-bin` (Arch) — no in-app updater by design: the AUR helper
     is the update channel. `check_for_update` skips the endpoint check on
     systems with neither dpkg nor rpm, so AUR users never see a dead-end
     update banner.
3. The updater manifest carries three Linux keys:
   `linux-x86_64-deb`, `linux-x86_64-rpm`, and `linux-x86_64` (fallback,
   pointing at the .deb) — see the generate-latest-json step.
4. Flatpak remains blocked on the Flathub policy, not on engineering:
   if the policy ever changes, ADR-056 describes the bundle pipeline and
   this ADR can be reverted.

## Consequences

- Linux releases publish exactly two native bundles plus the updater
  manifest: `Origa_amd64.deb`, `Origa_x86_64.rpm`, `latest.json`.
- Release checklist additions: bump `pkgver`/`sha256sums` in
  `packaging/aur/origa-bin/PKGBUILD`, regenerate `.SRCINFO`, and push the
  AUR package before or together with the stable tag (the AUR package is
  not automated in release CI yet — follow-up).
- AUR registration is paused as of 2026-09-20 (automated-account wave,
  no manual queue). Until the `origa-bin` listing is live, the landing
  download card links the in-repo PKGBUILD (`makepkg -si`) instead of the
  AUR page; switch it to https://aur.archlinux.org/packages/origa-bin in
  a follow-up commit right after publishing. Watch aur-general / Arch
  news for the reopening.
- Fedora post-release checkpoint: install the .rpm from tag N-1 in a
  Fedora environment, run the in-app update to N, verify the version
  bumped and the desktop entry/icons survived the `rpm -U` path.
- The Flatpak sandbox data of existing users lives outside the standard
  XDG dir; the release notes carry a `cp -a` migration command
  (`~/.var/app/net.uwuwu.origa/data/net.uwuwu.origa` →
  `~/.local/share/net.uwuwu.origa`).
- Speech-dispatcher TTS keeps working through the native packages the same
  way it worked through the sandbox (client library + host daemon).
