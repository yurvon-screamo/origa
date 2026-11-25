//! Integration tests for the Android smoke script's device boundary (#1486).
//!
//! The freezer knob is applied through `adb shell device_config` before the
//! APK is reinstalled, and the script must fail closed when the device does
//! not accept it. A scripted `adb` stands in for the emulator: the script is
//! executed as a black box and only its observable behaviour (exit status,
//! diagnostics, whether `install -r` was reached) is asserted. No device or
//! network is needed, so these tests run in the normal host test contour.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_RIG: AtomicUsize = AtomicUsize::new(0);

const ADB_STUB: &str = r#"#!/usr/bin/env bash
set -u
STATE="${STUB_STATE_DIR:?}"
case "$*" in
    devices) printf 'List of devices attached\nemulator-5554\tdevice\n' ;;
    "shell getprop sys.boot_completed") printf '1\n' ;;
    "shell device_config put activity_manager max_cached_processes 0")
        printf '%s' "${STUB_FREEZER_VALUE:-0}" > "$STATE/freezer" ;;
    "shell device_config get activity_manager max_cached_processes")
        cat "$STATE/freezer" 2>/dev/null || printf 'null\n' ;;
    "install -r "*)
        printf 'ok' > "$STATE/installed" ;;
    "logcat -c")
        printf 'I origa : [android-context] published\nI origa : [rustls] platform-verifier initialized for Android\n' > "$STATE/log" ;;
    "shell cmd package resolve-activity --brief -c android.intent.category.LAUNCHER net.uwuwu.origa")
        printf 'net.uwuwu.origa/.MainActivity\n' ;;
    "shell am start -n net.uwuwu.origa/.MainActivity") : ;;
    "shell pidof net.uwuwu.origa") printf '1234\n' ;;
    "shell am start -a android.intent.action.VIEW -d origa://selftest-ocr")
        printf 'I origa : [selftest] ocr ok: 日本語\n' >> "$STATE/log" ;;
    "logcat -d") cat "$STATE/log" 2>/dev/null || true ;;
    *) : ;;
esac
"#;

struct SmokeRig {
    dir: PathBuf,
    freezer_value: String,
}

impl SmokeRig {
    /// `freezer_value` models what `device_config get` reports after the
    /// script's `put`: "0" on an emulator that accepts the knob, anything
    /// else on a device where it is unavailable or ignored.
    fn new(freezer_value: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "origa-smoke-gate-{}-{}",
            std::process::id(),
            NEXT_RIG.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).expect("create smoke rig dir");
        fs::write(dir.join("adb-stub"), ADB_STUB).expect("write adb stub");
        fs::set_permissions(dir.join("adb-stub"), fs::Permissions::from_mode(0o755))
            .expect("make adb stub executable");
        fs::write(dir.join("test.apk"), b"fake apk").expect("write fake apk");

        Self {
            dir,
            freezer_value: freezer_value.to_string(),
        }
    }

    fn run(&self) -> Output {
        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/android_smoke.sh");
        Command::new("bash")
            .arg(script)
            .arg(self.dir.join("test.apk"))
            .env("ADB", self.dir.join("adb-stub"))
            .env("STUB_STATE_DIR", &self.dir)
            .env("STUB_FREEZER_VALUE", &self.freezer_value)
            .env("EMU_LOG", self.dir.join("emulator.log"))
            .env("BOOT_TIMEOUT_S", "5")
            .env("SETTLE_S", "0")
            .env("SELFTEST_TIMEOUT_S", "10")
            .output()
            .expect("run android_smoke.sh")
    }

    fn reached_install(&self) -> bool {
        self.dir.join("installed").exists()
    }
}

impl Drop for SmokeRig {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn smoke_script_runs_the_warm_path_when_the_freezer_knob_applies() {
    // Arrange
    let rig = SmokeRig::new("0");

    // Act
    let output = rig.run();

    // Assert
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "smoke script must pass with the freezer knob applied; stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.contains("[selftest] ocr ok"),
        "the warm selftest marker must be accepted; stdout={stdout}"
    );
    assert!(
        rig.reached_install(),
        "the script must reach `install -r` after the knob is verified"
    );
}

#[test]
fn smoke_script_refuses_to_install_when_the_freezer_knob_is_not_applied() {
    // Arrange — the device ignores `device_config put` and still reports a
    // non-zero value (the user-build/physical-device case from the review).
    let rig = SmokeRig::new("null");

    // Act
    let output = rig.run();

    // Assert
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "an unverified freezer knob must fail the smoke run"
    );
    assert!(
        stderr.contains("max_cached_processes knob did not apply"),
        "the failure must name the knob, not hide behind a later timeout; stderr={stderr}"
    );
    assert!(
        !rig.reached_install(),
        "install must not start until the knob is verified"
    );
}
