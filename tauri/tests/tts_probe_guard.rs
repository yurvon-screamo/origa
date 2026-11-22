#![cfg(any(windows, target_os = "linux"))]
//! Crash-class pins for the TTS probe-guard (Sentry RUST-S, task #1442).
//!
//! `tauri/src/lib.rs` registers `tauri-plugin-tts` only after a successful
//! engine probe (`tts::Tts::default()` — the exact engine init the plugin
//! repeats inside its own setup, `TtsEngine::default()`). A failed engine
//! init inside the plugin setup kills the whole app with
//! `PluginInitialization("tts", …)` before any window appears — the Sentry
//! RUST-S crash on Windows (Win10, release 0.7.37: "TTS error: WinRT error").
//!
//! On a Linux host the same failure mode is reproduced deterministically by
//! pointing the libspeechd client at a dead unix socket (`SPEECHD_ADDRESS`)
//! — the same engine stack both the probe and the plugin init use there.
//! There is no equivalent injection for a Windows engine on a non-Windows
//! host: the Windows decision cell is pinned by the `platform_routing`
//! rstest matrix, the cross-compilation by the CI Build Windows job, and the
//! production fact by the post-release Sentry watch (task #1442 Проверка).

#[cfg(target_os = "linux")]
#[test]
fn broken_speech_engine_probe_fails_and_unconditional_registration_dies() {
    const DEAD_SOCKET: &str = "unix:/nonexistent/speechd.sock";
    // SAFETY: single test in this binary (no parallel access to the process
    // env); libspeechd reads the var when the client connects below.
    unsafe { std::env::set_var("SPEECHD_ADDRESS", DEAD_SOCKET) };

    // The guard rests on probe ⇔ plugin-init equivalence: `tts::Tts::default()`
    // (the probe) and the plugin's `TtsEngine::default()` are the same engine
    // init. If a version bump ever breaks that equivalence (probe passes where
    // the plugin setup dies), the guard would register a plugin that still
    // kills the app — this assert is the tripwire for that drift.
    let err = tts::Tts::default()
        .err()
        .expect("probe must fail with a dead speech-dispatcher address");
    assert!(
        err.to_string().contains("Speech Dispatcher"),
        "unexpected probe error: {err}"
    );

    // The crash class itself (Sentry RUST-S): an unconditional tts-plugin
    // registration on an engine-broken machine dies with
    // `PluginInitialization("tts", …)` during builder setup — exactly what
    // killed the app before the fix. The guard in `run()` avoids precisely
    // this by probing first and skipping the registration on Err; this pin
    // keeps the crash reproducible in-test, so a regression that reverts to
    // unconditional registration cannot pass unnoticed.
    let result = tauri::test::mock_builder()
        .plugin(tauri_plugin_tts::init())
        .build(tauri::test::mock_context(tauri::test::noop_assets()));
    match result {
        Err(tauri::Error::PluginInitialization(name, msg)) => {
            assert_eq!(name, "tts");
            assert!(
                msg.contains("Speech Dispatcher"),
                "unexpected plugin init error: {msg}"
            );
        },
        Ok(_) => panic!(
            "tts plugin registration must fail on an engine-broken machine \
             (PluginInitialization) — if it succeeded, the broken-engine \
             injection no longer exercises the fatal path and this pin is \
             meaningless"
        ),
        Err(other) => panic!("unexpected error kind: {other}"),
    }
}
