//! Tauri IPC commands for the auto-updater (`tauri-plugin-updater`).
//!
//! The frontend (WASM) calls these commands through `invoke` to check for
//! available updates and install them. Progress events are emitted via
//! `app.emit("origa-update-progress", DownloadEvent)` so the WASM side can
//! subscribe through the existing `core::tauri::event_listen_fn()` helper
//! without any new JS-reflection code.
//!
//! The `Update` returned by `check` is stored in `PendingUpdate` state and
//! consumed by `install_update` — this avoids a second network round-trip and
//! the race condition of the manifest changing between check and install.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::platform_routing::HostPlatform;
use tauri_plugin_updater::{Update, UpdaterExt};

/// Holds the `Update` returned by `check_for_update` until `install_update`
/// consumes it. Without this state, `install_update` would have to re-run the
/// check (extra network round-trip + manifest race).
pub struct PendingUpdate(pub Mutex<Option<Update>>);

impl PendingUpdate {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
}

impl Default for PendingUpdate {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize)]
pub struct UpdateMetadata {
    pub version: String,
    pub current_version: String,
}

/// Event emitted on the `"origa-update-progress"` channel during
/// `install_update`. Per-variant `rename_all = "camelCase"` so the WASM side
/// receives canonical JS-style field names (`contentLength`, `chunkLength`).
#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum DownloadEvent {
    #[serde(rename_all = "camelCase")]
    Started {
        content_length: Option<u64>,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        chunk_length: usize,
    },
    Finished,
}

const PROGRESS_EVENT: &str = "origa-update-progress";

/// The decision the auto-update check has taken for one platform.
#[derive(Debug, PartialEq, Eq)]
pub enum UpdatePolicy {
    /// Proceed with the updater endpoint check.
    Check,
    /// Skip the check entirely; the payload is the logged reason.
    Skip(&'static str),
}

/// Pure routing of the auto-update check per platform (task 20261104060604,
/// GH #854). App-store builds never reach this function: the whole module is
/// compiled out by `#[cfg(all(any(windows, linux), not(app_store)))]`, so the
/// signature deliberately carries no store flag — that gate is enforced by
/// the module cfg and verified by the CI step `ORIGA_APP_STORE=1 cargo check`.
///
/// The macOS/Android/iOS arms exist only to make the decision matrix visible
/// to the host test table: the module is not compiled on those targets, so
/// their policy is a documented `Skip`, not reachable code.
///
/// AUR (Linux without dpkg/rpm) is the #701-class case this table pins: the
/// check itself would succeed and surface a banner whose install is
/// guaranteed to fail.
pub fn update_policy(
    platform: HostPlatform,
    is_dev: bool,
    dpkg_available: bool,
    rpm_available: bool,
) -> UpdatePolicy {
    if is_dev {
        return UpdatePolicy::Skip("dev build: version is always behind the latest release");
    }
    match platform {
        HostPlatform::Windows => UpdatePolicy::Check,
        // pkexec dpkg/rpm is the only install path the updater drives.
        HostPlatform::Linux => {
            if has_system_package_tool(dpkg_available, rpm_available) {
                UpdatePolicy::Check
            } else {
                UpdatePolicy::Skip("no dpkg/rpm on this system (e.g. AUR): install would fail")
            }
        },
        // No updater plugin is registered on these platforms: macOS updates
        // via the App Store / intentionally excluded builds, mobile has no
        // self-update path at all.
        HostPlatform::MacOS
        | HostPlatform::Android
        | HostPlatform::Ios => UpdatePolicy::Skip("no self-update path on this platform"),
    }
}


/// Checks the configured updater endpoint for a newer release.
///
/// Returns `Ok(None)` when the current build is already up to date. When an
/// update exists, its metadata is returned AND the `Update` object is stashed
/// in `PendingUpdate` so `install_update` can consume it without re-checking.
///
/// Dev builds skip the check entirely: their version is always behind the
/// latest release, so the updater banner would offer to overwrite the dev
/// binary with a released bundle. `tauri::is_dev()` is a compile-time signal
/// (`!cfg!(feature = "custom-protocol")`, enabled only by `cargo tauri
/// build`) — any binary not built through the Tauri CLI counts as dev.
#[tauri::command]
#[cfg(any(windows, target_os = "linux"))]
pub async fn check_for_update(
    app: AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<Option<UpdateMetadata>, String> {
    // The platform arm is decided at compile time — this command only exists
    // on Windows and Linux (the module cfg); `update_policy` carries the
    // full cross-platform table for the host test suite.
    let platform = if cfg!(windows) {
        HostPlatform::Windows
    } else {
        HostPlatform::Linux
    };
    #[cfg(target_os = "linux")]
    let (dpkg_available, rpm_available) = system_package_tools();
    #[cfg(not(target_os = "linux"))]
    let (dpkg_available, rpm_available) = (false, false);

    // Dev builds and platforms without a working install path (AUR: no
    // dpkg/rpm) skip here — the reasons live in `update_policy`'s table.
    match update_policy(platform, tauri::is_dev(), dpkg_available, rpm_available) {
        UpdatePolicy::Skip(reason) => {
            tracing::debug!("updater: skipping check: {reason}");
            return Ok(None);
        },
        UpdatePolicy::Check => {},
    }

    let update = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;

    let metadata = update.as_ref().map(|u| UpdateMetadata {
        version: u.version.clone(),
        current_version: u.current_version.clone(),
    });

    tracing::info!(?metadata, "updater check result");

    *pending.0.lock().unwrap_or_else(|e| e.into_inner()) = update;

    Ok(metadata)
}

/// Pure decision function for [`system_package_tool_available`]: at least one
/// of the two package managers the updater drives must be present. Injected
/// flags keep this unit-testable without spawning real processes.
fn has_system_package_tool(dpkg_available: bool, rpm_available: bool) -> bool {
    dpkg_available || rpm_available
}

/// Detects whether this system can install updates at all (Linux only; the
/// Windows NSIS path never reaches the caller). Probed once per process —
/// the answer cannot change while the app runs. Returns the per-tool
/// availability pair so [`update_policy`]'s table stays the single decision
/// point.
#[cfg(target_os = "linux")]
fn system_package_tools() -> (bool, bool) {
    use std::sync::OnceLock;

    static PROBE: OnceLock<(bool, bool)> = OnceLock::new();
    *PROBE.get_or_init(|| {
        let dpkg_ok = tool_runs("dpkg");
        let rpm_ok = tool_runs("rpm");
        tracing::debug!(
            dpkg_ok,
            rpm_ok,
            "system package tool probe: available={}",
            dpkg_ok || rpm_ok
        );
        (dpkg_ok, rpm_ok)
    })
}

/// Cheap liveness check for a package-manager binary on PATH.
#[cfg(target_os = "linux")]
fn tool_runs(tool: &str) -> bool {
    std::process::Command::new(tool)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Downloads and installs the update previously stashed by `check_for_update`.
///
/// Emits `Started` (once, on the first chunk), `Progress` (per chunk), and
/// `Finished` on the `"origa-update-progress"` channel, then restarts the app.
///
/// On Windows the NSIS installer terminates the process during
/// `download_and_install`, so `app.restart()` is unreachable there — it
/// remains as the restart path for Linux and macOS.
#[tauri::command]
#[cfg(any(windows, target_os = "linux"))]
pub async fn install_update(
    app: AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<(), String> {
    let update_opt = pending.0.lock().unwrap_or_else(|e| e.into_inner()).take();
    let update = update_opt.ok_or_else(|| "no pending update".to_string())?;

    let started = AtomicBool::new(false);
    let app_for_progress = app.clone();
    let app_for_finish = app.clone();

    update
        .download_and_install(
            move |chunk_length, content_length| {
                if !started.swap(true, Ordering::Relaxed) {
                    let _ = app_for_progress
                        .emit(PROGRESS_EVENT, DownloadEvent::Started { content_length });
                }
                let _ =
                    app_for_progress.emit(PROGRESS_EVENT, DownloadEvent::Progress { chunk_length });
            },
            move || {
                let _ = app_for_finish.emit(PROGRESS_EVENT, DownloadEvent::Finished);
            },
        )
        .await
        .map_err(|e| e.to_string())?;

    app.restart();
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    /// Host-side decision table for the auto-update check (task
    /// 20261104060604, GH #854): every platform's policy is pinned so a
    /// policy change without a matching row fails the plain `cargo test`
    /// run on any Linux CI runner — no device needed. The macOS/Android/iOS
    /// rows document that the module is compiled out there (never reachable
    /// code); the store-build gate is a module-level cfg, verified by the
    /// CI step `ORIGA_APP_STORE=1 cargo check`, not by this table.
    #[rstest]
    #[case::windows_release_checks(HostPlatform::Windows, false, false, false, UpdatePolicy::Check)]
    #[case::windows_dev_skips(
        HostPlatform::Windows,
        true,
        true,
        true,
        UpdatePolicy::Skip("dev build: version is always behind the latest release")
    )]
    #[case::linux_deb_checks(HostPlatform::Linux, false, true, false, UpdatePolicy::Check)]
    #[case::linux_rpm_checks(HostPlatform::Linux, false, false, true, UpdatePolicy::Check)]
    #[case::linux_deb_and_rpm_checks(HostPlatform::Linux, false, true, true, UpdatePolicy::Check)]
    #[case::linux_aur_without_tools_skips(
        HostPlatform::Linux,
        false,
        false,
        false,
        UpdatePolicy::Skip("no dpkg/rpm on this system (e.g. AUR): install would fail")
    )]
    #[case::linux_dev_skips_before_the_probe(
        HostPlatform::Linux,
        true,
        false,
        false,
        UpdatePolicy::Skip("dev build: version is always behind the latest release")
    )]
    #[case::macos_module_compiled_out(
        HostPlatform::MacOS,
        false,
        false,
        false,
        UpdatePolicy::Skip("no self-update path on this platform")
    )]
    #[case::android_module_compiled_out(
        HostPlatform::Android,
        false,
        true,
        true,
        UpdatePolicy::Skip("no self-update path on this platform")
    )]
    #[case::ios_module_compiled_out(
        HostPlatform::Ios,
        false,
        false,
        false,
        UpdatePolicy::Skip("no self-update path on this platform")
    )]
    fn update_policy_routes_every_platform(
        #[case] platform: HostPlatform,
        #[case] is_dev: bool,
        #[case] dpkg: bool,
        #[case] rpm: bool,
        #[case] expected: UpdatePolicy,
    ) {
        assert_eq!(update_policy(platform, is_dev, dpkg, rpm), expected);
    }

    /// The dpkg/rpm probe decision: at least one tool must be present.
    #[test]
    fn has_system_package_tool_requires_any_tool() {
        assert!(has_system_package_tool(true, true));
        assert!(has_system_package_tool(true, false));
        assert!(has_system_package_tool(false, true));
        assert!(!has_system_package_tool(false, false));
    }

    /// Wire-format contract: the WASM side (`origa_ui/src/core/updater.rs`)
    /// mirrors `DownloadEvent` with the same serde tags and per-variant
    /// `rename_all = "camelCase"`. This round-trip test pins the canonical
    /// JSON shape; if either side drifts, deserialization on the WASM side
    /// fails silently (progress events stop arriving) — the same class of
    /// bug this updater rewrite was meant to eliminate.
    #[test]
    fn download_event_started_serializes_camel_case() {
        let event = DownloadEvent::Started {
            content_length: Some(12345),
        };
        let json = serde_json::to_string(&event).expect("serialize Started");
        assert_eq!(
            json,
            r#"{"event":"Started","data":{"contentLength":12345}}"#
        );
    }

    #[test]
    fn download_event_started_with_null_content_length() {
        let event = DownloadEvent::Started {
            content_length: None,
        };
        let json = serde_json::to_string(&event).expect("serialize Started None");
        assert_eq!(json, r#"{"event":"Started","data":{"contentLength":null}}"#);
    }

    #[test]
    fn download_event_progress_serializes_camel_case() {
        let event = DownloadEvent::Progress { chunk_length: 1024 };
        let json = serde_json::to_string(&event).expect("serialize Progress");
        assert_eq!(json, r#"{"event":"Progress","data":{"chunkLength":1024}}"#);
    }

    #[test]
    fn download_event_finished_serializes_with_event_tag() {
        let event = DownloadEvent::Finished;
        let json = serde_json::to_string(&event).expect("serialize Finished");
        // serde tag/content for a unit variant may omit the `data` slot — the
        // event tag alone identifies the variant. Assert only the tag.
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("Finished JSON must parse");
        assert_eq!(parsed["event"], "Finished");
    }

    #[test]
    fn update_metadata_serializes_with_both_version_fields() {
        let metadata = UpdateMetadata {
            version: "0.4.2".to_string(),
            current_version: "0.4.1-rc3".to_string(),
        };
        let json = serde_json::to_string(&metadata).expect("serialize UpdateMetadata");
        assert_eq!(json, r#"{"version":"0.4.2","current_version":"0.4.1-rc3"}"#);
    }

    #[test]
    fn pending_update_new_starts_empty() {
        let pending = PendingUpdate::new();
        assert!(pending.0.lock().unwrap().is_none());
    }
}
